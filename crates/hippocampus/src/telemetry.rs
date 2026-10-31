//! Activation telemetry: logprob entropy, thinking-token density and cosine
//! drift become a pulse frequency, intensity and hue for the graph.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Activation {
    pub logprob_entropy: f32,
    pub thinking_density: f32,
    pub cosine_drift: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pulse {
    pub hz: f32,
    pub intensity: f32,
    pub hue: f32,
}

impl Activation {
    pub fn pulse(&self) -> Pulse {
        let entropy = (self.logprob_entropy / 3.0).clamp(0.0, 1.0);
        let thinking = self.thinking_density.clamp(0.0, 1.0);
        let drift = self.cosine_drift.clamp(0.0, 1.0);
        let mix = 0.45 * entropy + 0.35 * thinking + 0.20 * drift;
        Pulse {
            hz: 0.35 + 2.4 * mix,
            intensity: 0.35 + 0.65 * mix,
            hue: 190.0 + 120.0 * drift,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_is_bounded_and_monotonic() {
        let low = Activation { logprob_entropy: 0.1, thinking_density: 0.0, cosine_drift: 0.0 }.pulse();
        let high = Activation { logprob_entropy: 2.5, thinking_density: 0.9, cosine_drift: 0.8 }.pulse();
        assert!(high.hz > low.hz && high.intensity > low.intensity);
        for p in [low, high] {
            assert!((0.35..=2.75).contains(&p.hz));
            assert!((0.35..=1.0).contains(&p.intensity));
            assert!((190.0..=310.0).contains(&p.hue));
        }
    }
}
