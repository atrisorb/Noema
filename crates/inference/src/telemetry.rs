//! Stream events -> activation sample. Thalamus feeds the brain with the same
//! normalised events the adapters emit; no provider parsing is duplicated.

use serde::{Deserialize, Serialize};

use crate::StreamEvent;

/// Mirrors `noema_hippocampus::Activation` on the wire. Kept local so the
/// inference crate does not depend on the graph crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Activation {
    pub logprob_entropy: f32,
    pub thinking_density: f32,
    pub cosine_drift: f32,
}

/// Accumulates one completion stream into an activation sample.
#[derive(Debug, Default)]
pub struct ActivationWindow {
    surprisal: f64,
    logprobs: u64,
    reasoning: u64,
    output: u64,
    drift: f32,
}

impl ActivationWindow {
    pub fn ingest(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::Logprob { logprob, .. } => {
                // Mean surprisal (-ln p) is the token-entropy proxy.
                self.surprisal += (-logprob).clamp(0.0, 20.0);
                self.logprobs += 1;
            }
            StreamEvent::Reasoning { text } => self.reasoning += approx_tokens(text),
            StreamEvent::Delta { text } => self.output += approx_tokens(text),
            StreamEvent::Usage(usage) => {
                self.reasoning = self.reasoning.max(usage.reasoning);
                if usage.output > 0 {
                    self.output = self.output.max(usage.output);
                }
            }
            StreamEvent::Done { .. } => {}
        }
    }

    pub fn set_cosine_drift(&mut self, drift: f32) {
        self.drift = drift.clamp(0.0, 1.0);
    }

    pub fn is_empty(&self) -> bool {
        self.logprobs == 0 && self.reasoning == 0 && self.output == 0 && self.drift == 0.0
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn activation(&self) -> Activation {
        let entropy = if self.logprobs == 0 {
            0.0
        } else {
            (self.surprisal / self.logprobs as f64) as f32
        };
        let total = self.output + self.reasoning;
        let density = if total == 0 {
            0.0
        } else {
            self.reasoning as f32 / total as f32
        };
        Activation {
            logprob_entropy: entropy.clamp(0.0, 3.0),
            thinking_density: density.clamp(0.0, 1.0),
            cosine_drift: self.drift,
        }
    }
}

fn approx_tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Usage;

    #[test]
    fn logprobs_become_mean_surprisal() {
        let mut window = ActivationWindow::default();
        window.ingest(&StreamEvent::Logprob { token: "a".into(), logprob: -0.2 });
        window.ingest(&StreamEvent::Logprob { token: "b".into(), logprob: -0.8 });
        let activation = window.activation();
        assert!((activation.logprob_entropy - 0.5).abs() < 1e-6);
        assert_eq!(activation.thinking_density, 0.0);
    }

    #[test]
    fn reasoning_density_uses_stream_then_usage() {
        let mut window = ActivationWindow::default();
        window.ingest(&StreamEvent::Reasoning { text: "12345678".into() });
        window.ingest(&StreamEvent::Delta { text: "12345678".into() });
        assert!((window.activation().thinking_density - 0.5).abs() < 1e-6);
        window.ingest(&StreamEvent::Usage(Usage { input: 1, output: 4, reasoning: 2, cache_read: 0, cache_write: 0 }));
        assert!((window.activation().thinking_density - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn activation_is_bounded_and_resettable() {
        let mut window = ActivationWindow::default();
        window.ingest(&StreamEvent::Logprob { token: "a".into(), logprob: -50.0 });
        window.set_cosine_drift(9.0);
        let activation = window.activation();
        assert_eq!(activation.logprob_entropy, 3.0);
        assert_eq!(activation.cosine_drift, 1.0);
        assert!(!window.is_empty());
        window.reset();
        assert!(window.is_empty());
        assert_eq!(window.activation(), Activation::default());
    }
}
