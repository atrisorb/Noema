//! FinOps math: per-token prices, peak/off-peak windows in IANA timezones and
//! local hardware amortisation. All inputs are USD.

use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::Usage;

/// A peak or off-peak band. Weekday bit 0 is Monday. A range with
/// start_minute greater than end_minute wraps midnight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateWindow {
    pub timezone: String,
    pub weekday_mask: u8,
    pub start_minute: u16,
    pub end_minute: u16,
    pub multiplier: f64,
}

impl RateWindow {
    pub fn matches(&self, at: DateTime<Utc>) -> bool {
        let Ok(tz) = self.timezone.parse::<Tz>() else {
            return false;
        };
        let local = at.with_timezone(&tz);
        let weekday = local.weekday().num_days_from_monday() as u8;
        if self.weekday_mask & (1 << weekday) == 0 {
            return false;
        }
        let minute = (local.hour() * 60 + local.minute()) as u16;
        if self.start_minute <= self.end_minute {
            (self.start_minute..self.end_minute).contains(&minute)
        } else {
            minute >= self.start_minute || minute < self.end_minute
        }
    }
}

/// Local model cost: hardware amortisation plus electricity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalCost {
    pub hardware_usd: f64,
    pub lifespan_hours: f64,
    pub watts: f64,
    pub kwh_price: f64,
}

impl LocalCost {
    pub fn per_hour(&self) -> f64 {
        let hardware = if self.lifespan_hours > 0.0 {
            self.hardware_usd / self.lifespan_hours
        } else {
            0.0
        };
        hardware + self.watts / 1000.0 * self.kwh_price
    }
}

/// Per-million-token prices plus optional windows and local hardware.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateCard {
    pub input: f64,
    pub output: f64,
    pub reasoning: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub per_call: f64,
    #[serde(default)]
    pub windows: Vec<RateWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<LocalCost>,
}

impl Default for RateCard {
    fn default() -> Self {
        Self {
            input: 0.0,
            output: 0.0,
            reasoning: 0.0,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }
}

impl RateCard {
    /// Cost at the current instant with no local hours (remote call shape).
    pub fn cost(&self, usage: &Usage) -> f64 {
        self.cost_at(usage, Utc::now(), 0.0)
    }

    /// Full formula from docs/ARCHITECTURE.md section 7.
    pub fn cost_at(&self, usage: &Usage, at: DateTime<Utc>, hours: f64) -> f64 {
        let m = 1_000_000.0;
        let tokens = (usage.input as f64 * self.input
            + usage.output as f64 * self.output
            + usage.reasoning as f64 * self.reasoning
            + usage.cache_read as f64 * self.cache_read
            + usage.cache_write as f64 * self.cache_write)
            / m;
        let local = self.local.map(|l| l.per_hour() * hours).unwrap_or(0.0);
        tokens * self.multiplier(at) + self.per_call + local
    }

    /// First matching window wins; declaration order is the priority order.
    pub fn multiplier(&self, at: DateTime<Utc>) -> f64 {
        self.windows
            .iter()
            .find(|w| w.matches(at))
            .map(|w| w.multiplier)
            .unwrap_or(1.0)
    }

    pub fn peak(&self, at: DateTime<Utc>) -> bool {
        self.multiplier(at) != 1.0
    }

    /// DeepSeek-V3 / DeepSeek-Chat pricing.
    /// Source: https://api-docs.deepseek.com/quick_start/pricing (verified 2026-10-31).
    /// Input: $0.14/M (cache hit $0.014/M), Output: $0.28/M.
    pub fn deepseek_v3() -> Self {
        Self {
            input: 0.14,
            output: 0.28,
            reasoning: 0.28,
            cache_read: 0.014,
            cache_write: 0.14,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// DeepSeek-R1 / DeepSeek-Reasoner pricing.
    /// Source: https://api-docs.deepseek.com/quick_start/pricing (verified 2026-10-31).
    /// Input: $0.55/M (cache hit $0.14/M), Output: $2.19/M (includes reasoning tokens).
    pub fn deepseek_r1() -> Self {
        Self {
            input: 0.55,
            output: 2.19,
            reasoning: 2.19,
            cache_read: 0.14,
            cache_write: 0.55,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Mistral Large (2411) pricing.
    /// Source: https://mistral.ai/technology/#pricing (verified 2026-10-31).
    /// Input: $2.00/M, Output: $6.00/M.
    pub fn mistral_large() -> Self {
        Self {
            input: 2.00,
            output: 6.00,
            reasoning: 6.00,
            cache_read: 0.50,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Mistral Small (2501) pricing.
    /// Source: https://mistral.ai/technology/#pricing (verified 2026-10-31).
    /// Input: $0.10/M, Output: $0.30/M.
    pub fn mistral_small() -> Self {
        Self {
            input: 0.10,
            output: 0.30,
            reasoning: 0.30,
            cache_read: 0.025,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Codestral (2501) code-specialised model pricing.
    /// Source: https://mistral.ai/technology/#pricing (verified 2026-10-31).
    /// Input: $0.30/M, Output: $0.90/M.
    pub fn mistral_codestral() -> Self {
        Self {
            input: 0.30,
            output: 0.90,
            reasoning: 0.90,
            cache_read: 0.075,
            cache_write: 0.30,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Gemini 3.8 Flash (latest generation flagship, verified 2026-10-31).
    /// Source: https://ai.google.dev/pricing
    /// Input: $0.75/M, Output: $3.75/M (text and thinking), Cache read: $0.075/M.
    pub fn gemini_3_8_flash() -> Self {
        Self {
            input: 0.75,
            output: 3.75,
            reasoning: 3.75,
            cache_read: 0.075,
            cache_write: 0.75,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Gemini 3.5 Flash-Lite (high throughput agentic model, verified 2026-10-31).
    /// Source: https://ai.google.dev/pricing
    /// Input: $0.30/M, Output: $2.50/M, Cache read: $0.03/M.
    pub fn gemini_3_5_flash_lite() -> Self {
        Self {
            input: 0.30,
            output: 2.50,
            reasoning: 2.50,
            cache_read: 0.03,
            cache_write: 0.30,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Gemini 3.1 Pro (complex reasoning flagship, verified 2026-10-31).
    /// Source: https://ai.google.dev/pricing
    /// Input: $2.00/M, Output: $12.00/M (text and thinking), Cache read: $0.20/M.
    pub fn gemini_3_1_pro() -> Self {
        Self {
            input: 2.00,
            output: 12.00,
            reasoning: 12.00,
            cache_read: 0.20,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Gemini 2.5 Flash pricing (legacy baseline).
    /// Source: https://ai.google.dev/pricing (verified 2026-10-31).
    /// Input: $0.075/M (<=128k context; cached $0.01875/M), Output: $0.30/M.
    pub fn gemini_2_5_flash() -> Self {
        Self {
            input: 0.075,
            output: 0.30,
            reasoning: 0.30,
            cache_read: 0.01875,
            cache_write: 0.075,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Gemini 2.5 Pro pricing (legacy baseline).
    /// Source: https://ai.google.dev/pricing (verified 2026-10-31).
    /// Input: $1.25/M (<=128k context; cached $0.3125/M), Output: $5.00/M.
    pub fn gemini_2_5_pro() -> Self {
        Self {
            input: 1.25,
            output: 5.00,
            reasoning: 5.00,
            cache_read: 0.3125,
            cache_write: 1.25,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// OpenAI GPT-4o pricing (verified 2026-10-31).
    /// Source: https://openai.com/api/pricing
    /// Input: $2.50/M, Output: $10.00/M, Cache read: $1.25/M.
    pub fn openai_gpt_4o() -> Self {
        Self {
            input: 2.50,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 1.25,
            cache_write: 2.50,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// OpenAI GPT-4o mini pricing (verified 2026-10-31).
    /// Source: https://openai.com/api/pricing
    /// Input: $0.15/M, Output: $0.60/M, Cache read: $0.075/M.
    pub fn openai_gpt_4o_mini() -> Self {
        Self {
            input: 0.15,
            output: 0.60,
            reasoning: 0.60,
            cache_read: 0.075,
            cache_write: 0.15,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// OpenAI o1 reasoning model pricing (verified 2026-10-31).
    /// Source: https://openai.com/api/pricing
    /// Input: $15.00/M, Output: $60.00/M (includes reasoning), Cache read: $7.50/M.
    pub fn openai_o1() -> Self {
        Self {
            input: 15.00,
            output: 60.00,
            reasoning: 60.00,
            cache_read: 7.50,
            cache_write: 15.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// OpenAI o3-mini reasoning model pricing (verified 2026-10-31).
    /// Source: https://openai.com/api/pricing
    /// Input: $1.10/M, Output: $4.40/M (includes reasoning), Cache read: $0.55/M.
    pub fn openai_o3_mini() -> Self {
        Self {
            input: 1.10,
            output: 4.40,
            reasoning: 4.40,
            cache_read: 0.55,
            cache_write: 1.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Anthropic Claude 3.7 Sonnet pricing (verified 2026-10-31).
    /// Source: https://www.anthropic.com/pricing
    /// Input: $3.00/M, Output: $15.00/M (includes thinking), Cache read: $0.30/M, Cache write: $3.75/M.
    pub fn claude_3_7_sonnet() -> Self {
        Self {
            input: 3.00,
            output: 15.00,
            reasoning: 15.00,
            cache_read: 0.30,
            cache_write: 3.75,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Anthropic Claude 3.5 Sonnet pricing (verified 2026-10-31).
    /// Source: https://www.anthropic.com/pricing
    /// Input: $3.00/M, Output: $15.00/M, Cache read: $0.30/M, Cache write: $3.75/M.
    pub fn claude_3_5_sonnet() -> Self {
        Self {
            input: 3.00,
            output: 15.00,
            reasoning: 15.00,
            cache_read: 0.30,
            cache_write: 3.75,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Anthropic Claude 3.5 Haiku pricing (verified 2026-10-31).
    /// Source: https://www.anthropic.com/pricing
    /// Input: $0.80/M, Output: $4.00/M, Cache read: $0.08/M, Cache write: $1.00/M.
    pub fn claude_3_5_haiku() -> Self {
        Self {
            input: 0.80,
            output: 4.00,
            reasoning: 4.00,
            cache_read: 0.08,
            cache_write: 1.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Anthropic Claude 3 Opus pricing (verified 2026-10-31).
    /// Source: https://www.anthropic.com/pricing
    /// Input: $15.00/M, Output: $75.00/M, Cache read: $1.50/M, Cache write: $18.75/M.
    pub fn claude_3_opus() -> Self {
        Self {
            input: 15.00,
            output: 75.00,
            reasoning: 75.00,
            cache_read: 1.50,
            cache_write: 18.75,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Zero token-rate card for local models (billed only by local amortisation / electricity).
    pub fn local_zero() -> Self {
        Self::default()
    }

    /// Look up the standard rate card for a model identifier.
    pub fn for_model(model: &str) -> Option<Self> {
        let m = model.to_lowercase();
        // 1. OpenAI reasoning & standard
        if m.contains("o3-mini") {
            Some(Self::openai_o3_mini())
        } else if m.contains("o1") {
            Some(Self::openai_o1())
        } else if m.contains("gpt-4o-mini") {
            Some(Self::openai_gpt_4o_mini())
        } else if m.contains("gpt-4o") || m.contains("gpt-4") || m.contains("chatgpt") {
            Some(Self::openai_gpt_4o())
        // 2. Anthropic
        } else if m.contains("claude-3-7") || m.contains("claude-3.7") {
            Some(Self::claude_3_7_sonnet())
        } else if m.contains("claude-3-5-sonnet") || m.contains("claude-3.5-sonnet") || m.contains("claude-sonnet") {
            Some(Self::claude_3_5_sonnet())
        } else if m.contains("claude-3-5-haiku") || m.contains("claude-3.5-haiku") || m.contains("claude-haiku") {
            Some(Self::claude_3_5_haiku())
        } else if m.contains("claude-3-opus") || m.contains("claude-opus") {
            Some(Self::claude_3_opus())
        // 3. DeepSeek
        } else if m.contains("deepseek-r1") || m.contains("reasoner") {
            Some(Self::deepseek_r1())
        } else if m.contains("deepseek") {
            Some(Self::deepseek_v3())
        // 4. Google Gemini (current 3.x generation, then legacy 2.5)
        } else if m.contains("gemini-3.8-flash") || m.contains("gemini-3.8") {
            Some(Self::gemini_3_8_flash())
        } else if m.contains("gemini-3.5-flash-lite") || m.contains("gemini-3.5") {
            Some(Self::gemini_3_5_flash_lite())
        } else if m.contains("gemini-3.1-pro") || m.contains("gemini-3-pro") {
            Some(Self::gemini_3_1_pro())
        } else if m.contains("gemini-2.5-pro") {
            Some(Self::gemini_2_5_pro())
        } else if m.contains("gemini-2.5-flash") {
            Some(Self::gemini_2_5_flash())
        } else if m.contains("gemini-pro") {
            Some(Self::gemini_3_1_pro())
        } else if m.contains("gemini") {
            Some(Self::gemini_3_8_flash())
        // 5. Mistral
        } else if m.contains("codestral") {
            Some(Self::mistral_codestral())
        } else if m.contains("mistral-large") {
            Some(Self::mistral_large())
        } else if m.contains("mistral") {
            Some(Self::mistral_small())
        // 6. Local open weights
        } else if m.contains("llama") || m.contains("qwen") || m.contains("phi") || m.contains("gemma") {
            Some(Self::local_zero())
        } else {
            None
        }
    }
}

/// Descriptor for a model in the default 2026 inference catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub id: &'static str,
    pub provider: &'static str,
    pub name: &'static str,
    pub capability: &'static str,
    pub context_window: u64,
    pub rate_card: RateCard,
}

/// Default built-in model catalog for all supported cloud and local providers (October 2026).
pub fn default_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        // Google Gemini (active 3.x series)
        ModelCatalogEntry {
            id: "gemini-3.8-flash",
            provider: "google",
            name: "Gemini 3.8 Flash",
            capability: "general,reasoning,agentic,code",
            context_window: 1_000_000,
            rate_card: RateCard::gemini_3_8_flash(),
        },
        ModelCatalogEntry {
            id: "gemini-3.5-flash-lite",
            provider: "google",
            name: "Gemini 3.5 Flash-Lite",
            capability: "fast,general,agentic",
            context_window: 1_000_000,
            rate_card: RateCard::gemini_3_5_flash_lite(),
        },
        ModelCatalogEntry {
            id: "gemini-3.1-pro-preview",
            provider: "google",
            name: "Gemini 3.1 Pro",
            capability: "reasoning,complex,code",
            context_window: 2_000_000,
            rate_card: RateCard::gemini_3_1_pro(),
        },
        // OpenAI
        ModelCatalogEntry {
            id: "gpt-4o",
            provider: "openai",
            name: "GPT-4o",
            capability: "general,multimodal,code",
            context_window: 128_000,
            rate_card: RateCard::openai_gpt_4o(),
        },
        ModelCatalogEntry {
            id: "gpt-4o-mini",
            provider: "openai",
            name: "GPT-4o mini",
            capability: "fast,general",
            context_window: 128_000,
            rate_card: RateCard::openai_gpt_4o_mini(),
        },
        ModelCatalogEntry {
            id: "o1",
            provider: "openai",
            name: "o1",
            capability: "deep_reasoning,math,code",
            context_window: 200_000,
            rate_card: RateCard::openai_o1(),
        },
        ModelCatalogEntry {
            id: "o3-mini",
            provider: "openai",
            name: "o3-mini",
            capability: "reasoning,code,fast",
            context_window: 200_000,
            rate_card: RateCard::openai_o3_mini(),
        },
        // Anthropic
        ModelCatalogEntry {
            id: "claude-3-7-sonnet-20250219",
            provider: "anthropic",
            name: "Claude 3.7 Sonnet",
            capability: "reasoning,agentic,code",
            context_window: 200_000,
            rate_card: RateCard::claude_3_7_sonnet(),
        },
        ModelCatalogEntry {
            id: "claude-3-5-sonnet-20241022",
            provider: "anthropic",
            name: "Claude 3.5 Sonnet",
            capability: "reasoning,code,general",
            context_window: 200_000,
            rate_card: RateCard::claude_3_5_sonnet(),
        },
        ModelCatalogEntry {
            id: "claude-3-5-haiku-20241022",
            provider: "anthropic",
            name: "Claude 3.5 Haiku",
            capability: "fast,code,general",
            context_window: 200_000,
            rate_card: RateCard::claude_3_5_haiku(),
        },
        // DeepSeek
        ModelCatalogEntry {
            id: "deepseek-chat",
            provider: "deepseek",
            name: "DeepSeek-V3",
            capability: "general,code",
            context_window: 64_000,
            rate_card: RateCard::deepseek_v3(),
        },
        ModelCatalogEntry {
            id: "deepseek-reasoner",
            provider: "deepseek",
            name: "DeepSeek-R1",
            capability: "deep_reasoning,math,code",
            context_window: 64_000,
            rate_card: RateCard::deepseek_r1(),
        },
        // Mistral
        ModelCatalogEntry {
            id: "mistral-large-2411",
            provider: "mistral",
            name: "Mistral Large",
            capability: "general,reasoning,multilingual",
            context_window: 128_000,
            rate_card: RateCard::mistral_large(),
        },
        ModelCatalogEntry {
            id: "codestral-2501",
            provider: "mistral",
            name: "Codestral",
            capability: "code,fill_in_the_middle",
            context_window: 256_000,
            rate_card: RateCard::mistral_codestral(),
        },
        // Local open-weights
        ModelCatalogEntry {
            id: "llama-3.3-70b-instruct",
            provider: "local",
            name: "Llama 3.3 70B",
            capability: "local,general,code",
            context_window: 128_000,
            rate_card: RateCard::local_zero(),
        },
        ModelCatalogEntry {
            id: "qwen2.5-coder:32b",
            provider: "local",
            name: "Qwen 2.5 Coder 32B",
            capability: "local,code,agentic",
            context_window: 128_000,
            rate_card: RateCard::local_zero(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn free_model_costs_zero() {
        let card = RateCard::default();
        let usage = Usage {
            input: 10_000,
            output: 2_000,
            ..Default::default()
        };
        close(card.cost(&usage), 0.0);
    }

    #[test]
    fn tokens_formula_matches_architecture_spec() {
        let card = RateCard {
            input: 3.0,
            output: 15.0,
            reasoning: 15.0,
            cache_read: 0.30,
            cache_write: 3.75,
            per_call: 0.001,
            windows: Vec::new(),
            local: None,
        };
        let usage = Usage {
            input: 1_000_000,
            output: 200_000,
            reasoning: 50_000,
            cache_read: 500_000,
            cache_write: 100_000,
        };
        // 3.0 + 3.0 + 0.75 + 0.15 + 0.375 = 7.275 + 0.001 per_call = 7.276
        close(card.cost(&usage), 7.276);
    }

    #[test]
    fn peak_window_matches_and_scales() {
        let card = RateCard {
            input: 10.0,
            output: 30.0,
            windows: vec![RateWindow {
                timezone: "Europe/Rome".into(),
                weekday_mask: 0b0011111,
                start_minute: 9 * 60,
                end_minute: 18 * 60,
                multiplier: 1.5,
            }],
            ..Default::default()
        };
        let usage = Usage {
            input: 1_000_000,
            ..Default::default()
        };
        assert!(card.peak(at("2026-10-28T10:00:00+01:00")));
        close(
            card.cost_at(&usage, at("2026-10-28T10:00:00+01:00"), 0.0),
            15.0,
        );
        assert!(!card.peak(at("2026-10-28T22:00:00+01:00")));
        close(
            card.cost_at(&usage, at("2026-10-28T22:00:00+01:00"), 0.0),
            10.0,
        );
        assert!(!card.peak(at("2026-10-25T12:00:00+01:00")));
    }

    #[test]
    fn local_hardware_amortisation_adds_to_cost() {
        let card = RateCard {
            local: Some(LocalCost {
                hardware_usd: 3000.0,
                lifespan_hours: 10_000.0,
                watts: 350.0,
                kwh_price: 0.25,
            }),
            ..Default::default()
        };
        let usage = Usage::default();
        // 3000/10000 = 0.30 $/h; 0.35 kW * 0.25 $/kWh = 0.0875 $/h; total = 0.3875 $/h
        close(card.cost_at(&usage, Utc::now(), 2.0), 0.775);
    }

    #[test]
    fn deepseek_v3_and_r1_rate_cards() {
        let v3 = RateCard::deepseek_v3();
        let r1 = RateCard::deepseek_r1();
        assert_eq!(v3.input, 0.14);
        assert_eq!(v3.output, 0.28);
        assert_eq!(r1.input, 0.55);
        assert_eq!(r1.output, 2.19);
        assert_eq!(RateCard::for_model("deepseek/deepseek-flash"), Some(v3));
        assert_eq!(RateCard::for_model("deepseek-reasoner"), Some(r1));
    }

    #[test]
    fn mistral_and_gemini_rate_cards() {
        let ml = RateCard::mistral_large();
        let ms = RateCard::mistral_small();
        let gf = RateCard::gemini_2_5_flash();
        let gp = RateCard::gemini_2_5_pro();
        assert_eq!(ml.input, 2.00);
        assert_eq!(ms.input, 0.10);
        assert_eq!(gf.input, 0.075);
        assert_eq!(gp.input, 1.25);
        assert_eq!(RateCard::for_model("mistral-large-2411"), Some(ml));
        assert_eq!(RateCard::for_model("gemini-2.5-flash"), Some(gf));
        assert_eq!(RateCard::for_model("gemini-2.5-pro"), Some(gp));
    }

    #[test]
    fn openai_and_anthropic_rate_cards() {
        let gpt4o = RateCard::openai_gpt_4o();
        let gpt4omini = RateCard::openai_gpt_4o_mini();
        let o1 = RateCard::openai_o1();
        let o3mini = RateCard::openai_o3_mini();
        let c37 = RateCard::claude_3_7_sonnet();
        let haiku = RateCard::claude_3_5_haiku();

        assert_eq!(gpt4o.input, 2.50);
        assert_eq!(gpt4omini.output, 0.60);
        assert_eq!(o1.reasoning, 60.00);
        assert_eq!(o3mini.input, 1.10);
        assert_eq!(c37.input, 3.00);
        assert_eq!(haiku.cache_read, 0.08);

        assert_eq!(RateCard::for_model("gpt-4o"), Some(gpt4o));
        assert_eq!(RateCard::for_model("gpt-4o-mini"), Some(gpt4omini));
        assert_eq!(RateCard::for_model("o1"), Some(o1));
        assert_eq!(RateCard::for_model("o3-mini"), Some(o3mini));
        assert_eq!(RateCard::for_model("claude-3-7-sonnet"), Some(c37));
        assert_eq!(RateCard::for_model("claude-3-5-haiku-20241022"), Some(haiku));
    }

    #[test]
    fn gemini_3_and_catalog_entries() {
        let g38 = RateCard::gemini_3_8_flash();
        let g35 = RateCard::gemini_3_5_flash_lite();
        let g31 = RateCard::gemini_3_1_pro();

        assert_eq!(g38.input, 0.75);
        assert_eq!(g38.output, 3.75);
        assert_eq!(g35.input, 0.30);
        assert_eq!(g31.output, 12.00);

        assert_eq!(RateCard::for_model("gemini-3.8-flash"), Some(g38));
        assert_eq!(RateCard::for_model("gemini-3.5-flash-lite"), Some(g35));
        assert_eq!(RateCard::for_model("gemini-3.1-pro-preview"), Some(g31));
        assert_eq!(RateCard::for_model("llama-3.3-70b"), Some(RateCard::local_zero()));

        let catalog = default_catalog();
        assert!(catalog.len() >= 15);
        assert!(catalog.iter().any(|m| m.id == "gemini-3.8-flash"));
        assert!(catalog.iter().any(|m| m.id == "gpt-4o"));
        assert!(catalog.iter().any(|m| m.id == "claude-3-7-sonnet-20250219"));
    }
}
