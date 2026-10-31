//! Incremental 3D projection of embeddings into the brain volume.
//! UMAP-style core: a cosine kNN graph is relaxed with SGD attraction,
//! pairwise repulsion and an anchor pull to the functional region.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Prefrontal,
    Motor,
    Broca,
    Wernicke,
    Hippocampus,
}

impl Region {
    pub const ALL: [Region; 5] = [
        Region::Prefrontal,
        Region::Motor,
        Region::Broca,
        Region::Wernicke,
        Region::Hippocampus,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Region::Prefrontal => "prefrontal",
            Region::Motor => "motor",
            Region::Broca => "broca",
            Region::Wernicke => "wernicke",
            Region::Hippocampus => "hippocampus",
        }
    }

    /// Functional anchor inside the unit brain volume.
    pub fn anchor(self) -> [f32; 3] {
        match self {
            Region::Prefrontal => [0.0, 0.30, 0.70],
            Region::Motor => [0.0, 0.62, 0.05],
            Region::Broca => [-0.62, -0.05, 0.45],
            Region::Wernicke => [0.66, -0.10, 0.30],
            Region::Hippocampus => [0.30, -0.48, -0.12],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub id: String,
    pub embedding: Vec<f32>,
    #[serde(default = "default_region")]
    pub region: Region,
}

fn default_region() -> Region { Region::Prefrontal }

fn hash(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn unit(h: u64, salt: u64) -> f32 {
    let x = h.wrapping_mul(6364136223846793005).wrapping_add(salt * 1442695040888963407) >> 40;
    (x as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let (mut dot, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let d = na.sqrt() * nb.sqrt();
    if d == 0.0 { 0.0 } else { dot / d }
}

/// Deterministic: same ids and embeddings always produce the same layout, so a
/// restart never reshuffles the owner's memory. New thoughts are appended and
/// relaxed without moving the existing ones (incremental).
pub fn project(points: &[Input], iterations: u32) -> Vec<[f32; 3]> {
    let n = points.len();
    if n == 0 { return Vec::new(); }
    let anchors: Vec<[f32; 3]> = points.iter().map(|p| p.region.anchor()).collect();
    let mut pos: Vec<[f32; 3]> = points
        .iter()
        .zip(&anchors)
        .map(|(p, a)| {
            let h = hash(&p.id);
            [a[0] + 0.20 * unit(h, 1), a[1] + 0.20 * unit(h, 2), a[2] + 0.20 * unit(h, 3)]
        })
        .collect();

    let k = 4.min(n.saturating_sub(1)).max(1);
    let neighbours: Vec<Vec<(usize, f32)>> = (0..n)
        .map(|i| {
            let mut scored: Vec<(usize, f32)> = (0..n)
                .filter(|&j| j != i)
                .map(|j| (j, cosine(&points[i].embedding, &points[j].embedding)))
                .collect();
            scored.sort_by(|a, b| b.1.total_cmp(&a.1));
            scored.truncate(k);
            scored
        })
        .collect();

    for _ in 0..iterations {
        let mut next = pos.clone();
        for i in 0..n {
            let mut centroid = [0.0f32; 3];
            let mut wsum = 0.0f32;
            for (j, w) in &neighbours[i] {
                let w = (w + 1.0) * 0.5;
                for c in 0..3 { centroid[c] += pos[*j][c] * w; }
                wsum += w;
            }
            if wsum > 0.0 {
                for c in 0..3 {
                    next[i][c] += (centroid[c] / wsum - pos[i][c]) * 0.10;
                }
            }
            for c in 0..3 {
                next[i][c] += (anchors[i][c] - pos[i][c]) * 0.04;
            }
            for j in 0..n {
                if j == i { continue; }
                let mut d = [0.0f32; 3];
                let mut d2 = 0.0f32;
                for c in 0..3 {
                    d[c] = pos[i][c] - pos[j][c];
                    d2 += d[c] * d[c];
                }
                let d2 = d2.max(0.001);
                let f = 0.0009 / d2;
                for c in 0..3 { next[i][c] += d[c] * f; }
            }
        }
        pos = next;
        let max = pos.iter().fold(0.0f32, |m, p| {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            m.max(r)
        });
        if max > 1.0 {
            let s = 1.0 / max;
            for p in &mut pos { for c in p { *c *= s; } }
        }
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(per_region: usize) -> Vec<Input> {
        let mut out = Vec::new();
        for (ri, region) in Region::ALL.iter().enumerate() {
            for i in 0..per_region {
                let embedding = (0..8)
                    .map(|d| if d == ri { 1.0 } else { 0.1 * ((i * 7 + d * 3) % 5) as f32 })
                    .collect();
                out.push(Input { id: format!("{}-{i}", region.label()), embedding, region: *region });
            }
        }
        out
    }

    #[test]
    fn projection_is_deterministic_and_bounded() {
        let pts = sample(5);
        let a = project(&pts, 120);
        let b = project(&pts, 120);
        assert_eq!(a, b);
        for p in &a {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(r <= 1.0001, "radius {r}");
        }
    }

    #[test]
    fn points_stay_near_their_functional_region() {
        let pts = sample(5);
        let pos = project(&pts, 200);
        let mut own = 0.0f32;
        let mut cross = 0.0f32;
        let mut count = 0.0f32;
        for (p, at) in pts.iter().zip(&pos) {
            for region in Region::ALL {
                let a = region.anchor();
                let d = ((at[0] - a[0]).powi(2) + (at[1] - a[1]).powi(2) + (at[2] - a[2]).powi(2)).sqrt();
                if region == p.region { own += d; } else { cross += d; }
            }
            count += 1.0;
        }
        assert!(own / count < cross / (count * 4.0), "own {} cross {}", own / count, cross / (count * 4.0));
    }
}
