//! LLM-guided clustering: structured extraction (concepts + relations) becomes
//! thought nodes and weighted edges. Embeddings modulate edge weight and decide
//! clusters; no keyword heuristics.

use std::collections::BTreeMap;

use noema_core::ThoughtId;
use noema_core::vector::cosine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default = "half")]
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub relation: String,
    #[serde(default = "half")]
    pub weight: f32,
}

/// Shape the extraction model must return. causal_links are accepted as a
/// separate list and folded into relations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Extraction {
    #[serde(default)]
    pub concepts: Vec<Concept>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub causal_links: Vec<Relation>,
}

impl Extraction {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn all_relations(&self) -> Vec<&Relation> {
        self.relations.iter().chain(self.causal_links.iter()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtNode {
    pub id: ThoughtId,
    pub title: String,
    pub body: String,
    pub concepts: Vec<String>,
    pub cluster_id: Option<String>,
    pub importance: f32,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtEdge {
    pub id: ThoughtId,
    pub from: ThoughtId,
    pub to: ThoughtId,
    pub weight: f32,
    pub relation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThoughtGraph {
    pub nodes: Vec<ThoughtNode>,
    pub edges: Vec<ThoughtEdge>,
}

fn half() -> f32 {
    0.5
}

/// Build the graph deterministically from one extraction. Embedding lookup is
/// keyed by concept name; missing vectors leave the node unembedded.
pub fn build(extraction: &Extraction, embeddings: &BTreeMap<String, Vec<f32>>) -> ThoughtGraph {
    let mut nodes: Vec<ThoughtNode> = extraction
        .concepts
        .iter()
        .map(|c| ThoughtNode {
            id: ThoughtId::new(),
            title: c.name.clone(),
            body: c.summary.clone(),
            concepts: vec![c.name.clone()],
            cluster_id: None,
            importance: c.weight.clamp(0.0, 1.0),
            embedding: embeddings.get(&c.name).cloned().unwrap_or_default(),
        })
        .collect();
    let index: BTreeMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.title.as_str(), i))
        .collect();

    let mut edges: BTreeMap<(usize, usize), ThoughtEdge> = BTreeMap::new();
    for rel in extraction.all_relations() {
        let (Some(&from), Some(&to)) = (index.get(rel.from.as_str()), index.get(rel.to.as_str()))
        else {
            continue;
        };
        if from == to {
            continue;
        }
        let similarity = semantic_similarity(&nodes[from].embedding, &nodes[to].embedding);
        let weight = (rel.weight.clamp(0.0, 1.0) * (0.5 + 0.5 * similarity)).clamp(0.0, 1.0);
        let key = (from.min(to), from.max(to));
        let edge = ThoughtEdge {
            id: ThoughtId::new(),
            from: nodes[from].id.clone(),
            to: nodes[to].id.clone(),
            weight,
            relation: rel.relation.clone(),
        };
        match edges.get(&key) {
            Some(old) if old.weight >= weight => {}
            _ => {
                edges.insert(key, edge);
            }
        }
    }

    // Single-link clusters over edges at or above the link threshold.
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for (from, to) in edges.keys() {
        if edges[&(*from, *to)].weight >= LINK_THRESHOLD {
            union(&mut parent, *from, *to);
        }
    }
    for (i, node) in nodes.iter_mut().enumerate() {
        let root = find(&mut parent, i);
        node.cluster_id = Some(format!("cluster-{root}"));
    }

    ThoughtGraph {
        nodes,
        edges: edges.into_values().collect(),
    }
}

const LINK_THRESHOLD: f32 = 0.6;

/// Map cosine -1..1 to 0..1; missing vectors are neutral (0.5).
fn semantic_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.5;
    }
    (cosine(a, b) + 1.0) / 2.0
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (find(parent, a), find(parent, b));
    if ra != rb {
        let (lo, hi) = (ra.min(rb), ra.max(rb));
        parent[hi] = lo;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraction_becomes_weighted_graph() {
        let json = r#"{
            "concepts": [
                {"name": "attention", "summary": "focus mechanism", "weight": 0.9},
                {"name": "memory", "summary": "stored traces", "weight": 0.8},
                {"name": "sleep", "summary": "consolidation", "weight": 0.4}
            ],
            "relations": [
                {"from": "attention", "to": "memory", "relation": "gates", "weight": 0.8}
            ],
            "causal_links": [
                {"from": "sleep", "to": "memory", "relation": "consolidates", "weight": 0.7}
            ]
        }"#;
        let extraction = Extraction::parse(json).unwrap();
        let mut embeddings = BTreeMap::new();
        embeddings.insert("attention".to_string(), vec![1.0, 0.0]);
        embeddings.insert("memory".to_string(), vec![1.0, 0.0]);
        embeddings.insert("sleep".to_string(), vec![0.0, 1.0]);
        let graph = build(&extraction, &embeddings);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        // attention/memory identical vectors: 0.8 * (0.5 + 0.5 * 1.0) = 0.8
        let strong = graph
            .edges
            .iter()
            .find(|e| e.relation == "gates")
            .unwrap();
        assert!((strong.weight - 0.8).abs() < 1e-6);
        // sleep/memory orthogonal: 0.7 * (0.5 + 0.5 * 0.5) = 0.525
        let weak = graph
            .edges
            .iter()
            .find(|e| e.relation == "consolidates")
            .unwrap();
        assert!((weak.weight - 0.525).abs() < 1e-6);

        let attention = &graph.nodes[0];
        let memory = &graph.nodes[1];
        let sleep = &graph.nodes[2];
        assert_eq!(attention.cluster_id, memory.cluster_id);
        assert_ne!(attention.cluster_id, sleep.cluster_id);
        assert_eq!(attention.importance, 0.9);
    }

    #[test]
    fn unknown_relation_endpoints_are_ignored() {
        let extraction = Extraction::parse(
            r#"{"concepts":[{"name":"a"}],"relations":[{"from":"a","to":"ghost"}]}"#,
        )
        .unwrap();
        let graph = build(&extraction, &BTreeMap::new());
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.edges.is_empty());
        assert_eq!(graph.nodes[0].importance, 0.5);
    }

    #[test]
    fn duplicate_relations_keep_the_heaviest_edge() {
        let extraction = Extraction::parse(
            r#"{"concepts":[{"name":"a"},{"name":"b"}],
                "relations":[
                  {"from":"a","to":"b","relation":"weak","weight":0.2},
                  {"from":"b","to":"a","relation":"strong","weight":0.9}
                ]}"#,
        )
        .unwrap();
        let graph = build(&extraction, &BTreeMap::new());
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].relation, "strong");
    }
}
