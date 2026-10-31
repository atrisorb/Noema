//! Force-directed graph step: Hooke springs, pairwise repulsion, damping and a
//! hard tether clamp. The worker runs the same equations at 60 fps.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const REST: f32 = 0.32;
const SPATIAL_MIN_NODES: usize = 200;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub pos: [f32; 3],
    #[serde(skip)]
    pub vel: [f32; 3],
    /// Drag target; while set the node is held and cannot pull the rest away.
    pub pinned: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
    pub a: usize,
    pub b: usize,
    #[serde(default = "one")]
    pub weight: f32,
}

fn one() -> f32 { 1.0 }

fn cell_of(pos: [f32; 3], cell: f32) -> (i32, i32, i32) {
    (
        (pos[0] / cell).floor() as i32,
        (pos[1] / cell).floor() as i32,
        (pos[2] / cell).floor() as i32,
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub repulsion: f32,
    pub spring: f32,
    pub damping: f32,
    pub tether_radius: f32,
    pub max_speed: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self { repulsion: 0.02, spring: 1.2, damping: 0.86, tether_radius: 1.0, max_speed: 0.35 }
    }
}

pub struct Simulation {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
    pub params: Params,
}

impl Simulation {
    pub fn new(nodes: Vec<Node>, links: Vec<Link>) -> Self {
        Self { nodes, links, params: Params::default() }
    }

    pub fn set_mode(&mut self, dream: bool) {
        self.params = if dream {
            Params { repulsion: 0.05, spring: 0.35, damping: 0.94, tether_radius: 1.0, max_speed: 0.12 }
        } else {
            Params::default()
        };
    }

    /// Above 200 nodes only neighbours within one grid cell interact. Cell size
    /// tracks mean spacing, so the neighbourhood stays near 27 buckets.
    fn repulsion(&self, force: &mut [[f32; 3]]) {
        if self.nodes.len() <= SPATIAL_MIN_NODES {
            self.repulsion_all(force);
        } else {
            self.repulsion_near(force);
        }
    }

    fn repulsion_all(&self, force: &mut [[f32; 3]]) {
        for i in 0..self.nodes.len() {
            for j in (i + 1)..self.nodes.len() {
                self.repel(i, j, force);
            }
        }
    }

    fn repulsion_near(&self, force: &mut [[f32; 3]]) {
        let n = self.nodes.len();
        let cell = (4.2 / n as f32).cbrt().clamp(0.08, 0.35);
        let mut grid: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::with_capacity(n);
        for (i, node) in self.nodes.iter().enumerate() {
            grid.entry(cell_of(node.pos, cell)).or_default().push(i);
        }
        for (&(cx, cy, cz), bucket) in &grid {
            for &i in bucket {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let Some(neighbours) = grid.get(&(cx + dx, cy + dy, cz + dz)) else {
                                continue;
                            };
                            for &j in neighbours {
                                if j > i {
                                    self.repel(i, j, force);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn repel(&self, i: usize, j: usize, force: &mut [[f32; 3]]) {
        let mut d = [0.0f32; 3];
        for k in 0..3 { d[k] = self.nodes[i].pos[k] - self.nodes[j].pos[k]; }
        let d2 = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).max(0.0004);
        let f = self.params.repulsion / d2;
        for k in 0..3 {
            force[i][k] += d[k] * f;
            force[j][k] -= d[k] * f;
        }
    }

    /// Advance one tick. Forces are accelerations; the clamp keeps every node
    /// inside `tether_radius` from the origin, so one drag never drags the whole graph.
    pub fn step(&mut self, dt: f32) {
        let dt = dt.clamp(0.001, 0.05);
        let n = self.nodes.len();
        let mut force = vec![[0.0f32; 3]; n];
        self.repulsion(&mut force);
        for link in &self.links {
            let (a, b) = (link.a, link.b);
            let mut d = [0.0f32; 3];
            for k in 0..3 { d[k] = self.nodes[b].pos[k] - self.nodes[a].pos[k]; }
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(0.0001);
            let pull = self.params.spring * (len - REST) * link.weight / len;
            for k in 0..3 {
                force[a][k] += d[k] * pull;
                force[b][k] -= d[k] * pull;
            }
        }
        for i in 0..n {
            if let Some(mut target) = self.nodes[i].pinned {
                let len = (target[0].powi(2) + target[1].powi(2) + target[2].powi(2)).sqrt();
                if len > self.params.tether_radius {
                    let s = self.params.tether_radius / len;
                    for c in &mut target { *c *= s; }
                }
                self.nodes[i].pos = target;
                self.nodes[i].vel = [0.0; 3];
                continue;
            }
            for k in 0..3 {
                self.nodes[i].vel[k] = (self.nodes[i].vel[k] + force[i][k] * dt) * self.params.damping;
                self.nodes[i].pos[k] += self.nodes[i].vel[k] * dt;
            }
            let mut len2 = 0.0f32;
            for k in 0..3 { len2 += self.nodes[i].pos[k] * self.nodes[i].pos[k]; }
            let len = len2.sqrt();
            if len > self.params.tether_radius {
                let s = self.params.tether_radius / len;
                for k in 0..3 {
                    self.nodes[i].pos[k] *= s;
                    self.nodes[i].vel[k] *= s;
                }
            }
            let speed = (self.nodes[i].vel[0].powi(2) + self.nodes[i].vel[1].powi(2) + self.nodes[i].vel[2].powi(2)).sqrt();
            if speed > self.params.max_speed {
                let s = self.params.max_speed / speed;
                for k in 0..3 { self.nodes[i].vel[k] *= s; }
            }
        }
    }

    pub fn radius(&self) -> f32 {
        self.nodes.iter().fold(0.0f32, |m, node| {
            let r = (node.pos[0].powi(2) + node.pos[1].powi(2) + node.pos[2].powi(2)).sqrt();
            m.max(r)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded(n: usize) -> Vec<Node> {
        (0..n)
            .map(|i| {
                let a = i as f32 * 2.399_963;
                let r = 0.2 + (i % 7) as f32 * 0.09;
                Node { id: format!("t{i}"), pos: [r * a.cos(), r * a.sin(), (i as f32 % 5.0 - 2.0) * 0.08], vel: [0.0; 3], pinned: None }
            })
            .collect()
    }

    #[test]
    fn tether_holds_after_many_ticks() {
        let mut sim = Simulation::new(seeded(60), (0..59).map(|i| Link { a: i, b: i + 1, weight: 1.0 }).collect());
        for _ in 0..600 { sim.step(1.0 / 60.0); }
        assert!(sim.radius() <= 1.0001, "radius {}", sim.radius());
    }

    #[test]
    fn dragging_one_node_keeps_the_graph_tethered() {
        let mut sim = Simulation::new(seeded(40), (0..39).map(|i| Link { a: i, b: i + 1, weight: 1.0 }).collect());
        sim.nodes[0].pinned = Some([4.0, 4.0, 4.0]);
        for _ in 0..300 { sim.step(1.0 / 60.0); }
        assert!(sim.radius() <= 1.0001);
        let dragged = sim.nodes[0].pos;
        assert!((dragged[0] - 0.577).abs() < 0.01, "clamped drag {dragged:?}");
        let others = &sim.nodes[1..];
        assert!(others.iter().all(|n| n.pos != dragged));
    }

    #[test]
    fn spatial_hash_matches_brute_force_on_local_pairs() {
        let nodes: Vec<Node> = (0..240)
            .map(|i| {
                let x = (i % 6) as f32 * 0.03;
                let y = ((i / 6) % 6) as f32 * 0.03;
                let z = ((i / 36) % 6) as f32 * 0.03;
                Node { id: format!("s{i}"), pos: [x, y, z], vel: [0.0; 3], pinned: None }
            })
            .collect();
        let sim = Simulation::new(nodes, vec![]);
        let mut brute = vec![[0.0f32; 3]; 240];
        let mut grid = vec![[0.0f32; 3]; 240];
        sim.repulsion_all(&mut brute);
        sim.repulsion_near(&mut grid);
        for (a, b) in brute.iter().zip(&grid) {
            for k in 0..3 {
                assert!((a[k] - b[k]).abs() < 1e-6, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn spatial_hash_keeps_a_large_graph_tethered() {
        let mut sim = Simulation::new(seeded(500), (0..499).map(|i| Link { a: i, b: i + 1, weight: 1.0 }).collect());
        for _ in 0..120 { sim.step(1.0 / 60.0); }
        assert!(sim.radius() <= 1.0001);
        assert!(sim.nodes.iter().all(|n| n.pos.iter().all(|c| c.is_finite())));
    }

    #[test]
    fn dream_mode_keeps_the_clamp() {
        let mut sim = Simulation::new(seeded(30), vec![]);
        sim.set_mode(true);
        for _ in 0..600 { sim.step(1.0 / 60.0); }
        assert!(sim.radius() <= 1.0001);
    }
}
