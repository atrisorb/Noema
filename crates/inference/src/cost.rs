// crates/inference/src/cost.rs
// Universal FinOps token pricing, peak/off-peak windows, local hardware
// amortisation, and exhaustive October 2026 model registry across 10 providers.

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::{Standard, Usage};

/// Multiplier that applies when the current time falls inside a recurring window.
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
        let weekday_bit = match at.weekday() {
            chrono::Weekday::Mon => 0b0000001,
            chrono::Weekday::Tue => 0b0000010,
            chrono::Weekday::Wed => 0b0000100,
            chrono::Weekday::Thu => 0b0001000,
            chrono::Weekday::Fri => 0b0010000,
            chrono::Weekday::Sat => 0b0100000,
            chrono::Weekday::Sun => 0b1000000,
        };
        if self.weekday_mask & weekday_bit == 0 {
            return false;
        }
        let minute = at.hour() as u16 * 60 + at.minute() as u16;
        minute >= self.start_minute && minute < self.end_minute
    }
}

/// Local runtime hardware cost amortisation + electricity consumption.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalCost {
    pub hardware_capex_usd: f64,
    pub lifespan_hours: f64,
    pub power_watts: f64,
    pub electricity_kwh_usd: f64,
}

impl LocalCost {
    pub fn per_hour(&self) -> f64 {
        let depreciation = if self.lifespan_hours > 0.0 {
            self.hardware_capex_usd / self.lifespan_hours
        } else {
            0.0
        };
        let power_kwh = self.power_watts / 1000.0;
        let electricity = power_kwh * self.electricity_kwh_usd;
        depreciation + electricity
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
    pub fn cost(&self, usage: &Usage) -> f64 {
        self.cost_at(usage, Utc::now(), 0.0)
    }

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

    // =========================================================================
    // 1. Google Gemini
    // =========================================================================

    pub fn gemini_4_argon() -> Self {
        Self {
            input: 2.00,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 0.10,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

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

    pub fn gemini_3_8_live() -> Self {
        Self {
            input: 1.00,
            output: 4.00,
            reasoning: 4.00,
            cache_read: 0.10,
            cache_write: 1.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

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

    pub fn gemini_2_0_flash() -> Self {
        Self {
            input: 0.10,
            output: 0.40,
            reasoning: 0.40,
            cache_read: 0.025,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn gemini_1_5_pro() -> Self {
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

    pub fn gemini_1_5_flash() -> Self {
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

    pub fn gemini_1_0_pro() -> Self {
        Self {
            input: 0.50,
            output: 1.50,
            reasoning: 1.50,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 2. OpenAI
    // =========================================================================

    pub fn gpt_6_astra() -> Self {
        Self {
            input: 10.00,
            output: 50.00,
            reasoning: 50.00,
            cache_read: 1.00,
            cache_write: 10.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn gpt_6_1_sol() -> Self {
        Self {
            input: 2.00,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 0.20,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn gpt_6_1_terra() -> Self {
        Self {
            input: 1.00,
            output: 5.00,
            reasoning: 5.00,
            cache_read: 0.10,
            cache_write: 1.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn gpt_6_luna() -> Self {
        Self {
            input: 0.10,
            output: 0.50,
            reasoning: 0.50,
            cache_read: 0.01,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn gpt_realtime_2_1() -> Self {
        Self {
            input: 4.00,
            output: 24.00,
            reasoning: 24.00,
            cache_read: 0.40,
            cache_write: 4.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn gpt_realtime_2_1_mini() -> Self {
        Self {
            input: 0.60,
            output: 2.40,
            reasoning: 2.40,
            cache_read: 0.06,
            cache_write: 0.60,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o3() -> Self {
        Self {
            input: 2.00,
            output: 8.00,
            reasoning: 8.00,
            cache_read: 0.20,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o3_pro() -> Self {
        Self {
            input: 20.00,
            output: 80.00,
            reasoning: 80.00,
            cache_read: 2.00,
            cache_write: 20.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o4_mini() -> Self {
        Self {
            input: 1.10,
            output: 4.40,
            reasoning: 4.40,
            cache_read: 0.11,
            cache_write: 1.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o3_mini() -> Self {
        Self {
            input: 1.10,
            output: 4.40,
            reasoning: 4.40,
            cache_read: 0.11,
            cache_write: 1.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o1() -> Self {
        Self {
            input: 15.00,
            output: 60.00,
            reasoning: 60.00,
            cache_read: 1.50,
            cache_write: 15.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o1_preview() -> Self {
        Self {
            input: 15.00,
            output: 60.00,
            reasoning: 60.00,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_o1_mini() -> Self {
        Self {
            input: 3.00,
            output: 12.00,
            reasoning: 12.00,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_gpt_4o() -> Self {
        Self {
            input: 2.50,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 0.25,
            cache_write: 2.50,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_gpt_4o_mini() -> Self {
        Self {
            input: 0.15,
            output: 0.60,
            reasoning: 0.60,
            cache_read: 0.015,
            cache_write: 0.15,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_gpt_4_turbo() -> Self {
        Self {
            input: 10.00,
            output: 30.00,
            reasoning: 30.00,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_gpt_4() -> Self {
        Self {
            input: 30.00,
            output: 60.00,
            reasoning: 60.00,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn openai_gpt_3_5_turbo() -> Self {
        Self {
            input: 0.50,
            output: 1.50,
            reasoning: 1.50,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 3. Anthropic
    // =========================================================================

    pub fn claude_fable_5_1() -> Self {
        Self {
            input: 10.00,
            output: 50.00,
            reasoning: 50.00,
            cache_read: 1.00,
            cache_write: 10.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn claude_opus_5_5() -> Self {
        Self {
            input: 4.00,
            output: 20.00,
            reasoning: 20.00,
            cache_read: 0.20,
            cache_write: 4.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn claude_sonnet_5_5() -> Self {
        Self {
            input: 2.00,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 0.10,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn claude_haiku_5_5() -> Self {
        Self {
            input: 0.10,
            output: 0.50,
            reasoning: 0.50,
            cache_read: 0.01,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

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

    pub fn claude_3_sonnet() -> Self {
        Self {
            input: 3.00,
            output: 15.00,
            reasoning: 15.00,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn claude_3_haiku() -> Self {
        Self {
            input: 0.25,
            output: 1.25,
            reasoning: 1.25,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 4. Mistral AI
    // =========================================================================

    pub fn mistral_large_4() -> Self {
        Self {
            input: 0.70,
            output: 2.10,
            reasoning: 2.10,
            cache_read: 0.07,
            cache_write: 0.70,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn mistral_medium_3_5() -> Self {
        Self {
            input: 0.40,
            output: 1.20,
            reasoning: 1.20,
            cache_read: 0.04,
            cache_write: 0.40,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn mistral_large_3() -> Self {
        Self {
            input: 2.00,
            output: 6.00,
            reasoning: 6.00,
            cache_read: 0.20,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn mistral_small_4() -> Self {
        Self {
            input: 0.10,
            output: 0.30,
            reasoning: 0.30,
            cache_read: 0.01,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn mistral_small_2501() -> Self {
        Self {
            input: 0.10,
            output: 0.30,
            reasoning: 0.30,
            cache_read: 0.02,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn ministral_3_14b() -> Self {
        Self {
            input: 0.20,
            output: 0.20,
            reasoning: 0.20,
            cache_read: 0.02,
            cache_write: 0.20,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn ministral_3_8b() -> Self {
        Self {
            input: 0.10,
            output: 0.10,
            reasoning: 0.10,
            cache_read: 0.01,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn ministral_3_3b() -> Self {
        Self {
            input: 0.04,
            output: 0.04,
            reasoning: 0.04,
            cache_read: 0.004,
            cache_write: 0.04,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

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

    pub fn pixtral_12b() -> Self {
        Self {
            input: 0.15,
            output: 0.15,
            reasoning: 0.15,
            cache_read: 0.015,
            cache_write: 0.15,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn open_mixtral_8x22b() -> Self {
        Self {
            input: 2.00,
            output: 6.00,
            reasoning: 6.00,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn open_mixtral_8x7b() -> Self {
        Self {
            input: 0.70,
            output: 0.70,
            reasoning: 0.70,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn open_mistral_7b() -> Self {
        Self {
            input: 0.25,
            output: 0.25,
            reasoning: 0.25,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 5. DeepSeek
    // =========================================================================

    pub fn deepseek_flash() -> Self {
        Self {
            input: 0.15,
            output: 0.60,
            reasoning: 0.60,
            cache_read: 0.003,
            cache_write: 0.15,
            per_call: 0.0,
            windows: vec![
                RateWindow {
                    timezone: "UTC".into(),
                    weekday_mask: 0b0011111,
                    start_minute: 1 * 60,
                    end_minute: 4 * 60,
                    multiplier: 2.0,
                },
                RateWindow {
                    timezone: "UTC".into(),
                    weekday_mask: 0b0011111,
                    start_minute: 6 * 60,
                    end_minute: 10 * 60,
                    multiplier: 2.0,
                },
            ],
            local: None,
        }
    }

    pub fn deepseek_v4_pro() -> Self {
        Self {
            input: 0.66,
            output: 1.98,
            reasoning: 1.98,
            cache_read: 0.022,
            cache_write: 0.66,
            per_call: 0.0,
            windows: vec![
                RateWindow {
                    timezone: "UTC".into(),
                    weekday_mask: 0b0011111,
                    start_minute: 1 * 60,
                    end_minute: 4 * 60,
                    multiplier: 2.0,
                },
                RateWindow {
                    timezone: "UTC".into(),
                    weekday_mask: 0b0011111,
                    start_minute: 6 * 60,
                    end_minute: 10 * 60,
                    multiplier: 2.0,
                },
            ],
            local: None,
        }
    }

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

    pub fn deepseek_coder() -> Self {
        Self {
            input: 0.14,
            output: 0.28,
            reasoning: 0.28,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 6. Zhipu AI (GLM)
    // =========================================================================

    pub fn glm_5_3() -> Self {
        Self {
            input: 1.40,
            output: 4.40,
            reasoning: 4.40,
            cache_read: 0.26,
            cache_write: 1.40,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn glm_5_3_flash() -> Self {
        Self {
            input: 0.15,
            output: 0.50,
            reasoning: 0.50,
            cache_read: 0.02,
            cache_write: 0.15,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn glm_5_3_flashx() -> Self {
        Self {
            input: 0.37,
            output: 1.25,
            reasoning: 1.25,
            cache_read: 0.04,
            cache_write: 0.37,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn glm_4_plus() -> Self {
        Self {
            input: 1.40,
            output: 1.40,
            reasoning: 1.40,
            cache_read: 0.14,
            cache_write: 1.40,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn glm_4_air() -> Self {
        Self {
            input: 0.14,
            output: 0.14,
            reasoning: 0.14,
            cache_read: 0.014,
            cache_write: 0.14,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn glm_4_long() -> Self {
        Self {
            input: 0.14,
            output: 0.14,
            reasoning: 0.14,
            cache_read: 0.014,
            cache_write: 0.14,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn glm_4_7_flash() -> Self {
        Self::default()
    }

    pub fn glm_3_turbo() -> Self {
        Self {
            input: 0.07,
            output: 0.07,
            reasoning: 0.07,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 7. Moonshot AI (Kimi)
    // =========================================================================

    pub fn kimi_k3_1_pro() -> Self {
        Self {
            input: 2.00,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 0.20,
            cache_write: 2.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn kimi_k3_1_pro_highspeed() -> Self {
        Self {
            input: 4.00,
            output: 20.00,
            reasoning: 20.00,
            cache_read: 0.40,
            cache_write: 4.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn kimi_k3() -> Self {
        Self {
            input: 3.00,
            output: 15.00,
            reasoning: 15.00,
            cache_read: 0.30,
            cache_write: 3.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn moonshot_v1_128k() -> Self {
        Self {
            input: 8.40,
            output: 8.40,
            reasoning: 8.40,
            cache_read: 0.84,
            cache_write: 8.40,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn moonshot_v1_32k() -> Self {
        Self {
            input: 3.36,
            output: 3.36,
            reasoning: 3.36,
            cache_read: 0.336,
            cache_write: 3.36,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn moonshot_v1_8k() -> Self {
        Self {
            input: 1.68,
            output: 1.68,
            reasoning: 1.68,
            cache_read: 0.168,
            cache_write: 1.68,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    // =========================================================================
    // 8. Meta Llama & Qwen & Local
    // =========================================================================

    pub fn llama_3_3_70b() -> Self {
        Self {
            input: 0.60,
            output: 0.90,
            reasoning: 0.90,
            cache_read: 0.15,
            cache_write: 0.60,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_1_405b() -> Self {
        Self {
            input: 3.00,
            output: 3.00,
            reasoning: 3.00,
            cache_read: 0.50,
            cache_write: 3.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_1_70b() -> Self {
        Self {
            input: 0.50,
            output: 0.75,
            reasoning: 0.75,
            cache_read: 0.10,
            cache_write: 0.50,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_1_8b() -> Self {
        Self {
            input: 0.05,
            output: 0.08,
            reasoning: 0.08,
            cache_read: 0.01,
            cache_write: 0.05,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_2_90b_vision() -> Self {
        Self {
            input: 0.80,
            output: 1.20,
            reasoning: 1.20,
            cache_read: 0.15,
            cache_write: 0.80,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_2_11b_vision() -> Self {
        Self {
            input: 0.16,
            output: 0.16,
            reasoning: 0.16,
            cache_read: 0.03,
            cache_write: 0.16,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_2_3b() -> Self {
        Self {
            input: 0.04,
            output: 0.04,
            reasoning: 0.04,
            cache_read: 0.008,
            cache_write: 0.04,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_3_2_1b() -> Self {
        Self {
            input: 0.02,
            output: 0.02,
            reasoning: 0.02,
            cache_read: 0.004,
            cache_write: 0.02,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn llama_2_70b() -> Self {
        Self {
            input: 0.70,
            output: 0.90,
            reasoning: 0.90,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_coder_32b() -> Self {
        Self {
            input: 0.20,
            output: 0.20,
            reasoning: 0.20,
            cache_read: 0.05,
            cache_write: 0.20,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_coder_7b() -> Self {
        Self {
            input: 0.05,
            output: 0.05,
            reasoning: 0.05,
            cache_read: 0.01,
            cache_write: 0.05,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_72b() -> Self {
        Self {
            input: 0.35,
            output: 0.40,
            reasoning: 0.40,
            cache_read: 0.08,
            cache_write: 0.35,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_32b() -> Self {
        Self {
            input: 0.18,
            output: 0.18,
            reasoning: 0.18,
            cache_read: 0.04,
            cache_write: 0.18,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_14b() -> Self {
        Self {
            input: 0.10,
            output: 0.10,
            reasoning: 0.10,
            cache_read: 0.02,
            cache_write: 0.10,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_7b() -> Self {
        Self {
            input: 0.04,
            output: 0.04,
            reasoning: 0.04,
            cache_read: 0.008,
            cache_write: 0.04,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_5_math_72b() -> Self {
        Self {
            input: 0.40,
            output: 0.40,
            reasoning: 0.40,
            cache_read: 0.08,
            cache_write: 0.40,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwq_32b_preview() -> Self {
        Self {
            input: 0.25,
            output: 0.25,
            reasoning: 0.25,
            cache_read: 0.05,
            cache_write: 0.25,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn qwen_2_72b() -> Self {
        Self {
            input: 0.50,
            output: 0.50,
            reasoning: 0.50,
            cache_read: 0.0,
            cache_write: 0.0,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    pub fn local_zero() -> Self {
        Self::default()
    }

    /// Look up the standard rate card for any model identifier, endpoint alias, or snapshot.
    pub fn for_model(model: &str) -> Option<Self> {
        // 1. Direct query through the comprehensive model catalog (exact ID or alias)
        if let Some(entry) = catalog_entry(model) {
            return Some(entry.rate_card);
        }

        // 2. Fuzzy heuristic fallback for unusual custom endpoint variations
        let m = model.to_lowercase();
        if m.contains("argon") || m.contains("gemini-4") {
            Some(Self::gemini_4_argon())
        } else if m.contains("gemini-3.8-live") || m.contains("gemini-live") {
            Some(Self::gemini_3_8_live())
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
        } else if m.contains("gemini-2.0-flash") || m.contains("gemini-2.0") {
            Some(Self::gemini_2_0_flash())
        } else if m.contains("gemini-1.5-pro") {
            Some(Self::gemini_1_5_pro())
        } else if m.contains("gemini-1.5-flash") {
            Some(Self::gemini_1_5_flash())
        } else if m.contains("gemini-pro") || m.contains("gemini-1.0") {
            Some(Self::gemini_1_0_pro())
        } else if m.contains("gemini") {
            Some(Self::gemini_3_8_flash())
        } else if m.contains("gpt-6-astra") || m.contains("astra") {
            Some(Self::gpt_6_astra())
        } else if m.contains("gpt-6.1-sol") || m.contains("6.1-sol") || m.contains("sol") {
            Some(Self::gpt_6_1_sol())
        } else if m.contains("gpt-6.1-terra") || m.contains("6.1-terra") || m.contains("terra") {
            Some(Self::gpt_6_1_terra())
        } else if m.contains("gpt-6-luna") || m.contains("luna") {
            Some(Self::gpt_6_luna())
        } else if m.contains("realtime-2.1-mini") {
            Some(Self::gpt_realtime_2_1_mini())
        } else if m.contains("realtime-2.1") || m.contains("realtime") {
            Some(Self::gpt_realtime_2_1())
        } else if m.contains("o3-pro") {
            Some(Self::openai_o3_pro())
        } else if m.contains("o4-mini") {
            Some(Self::openai_o4_mini())
        } else if m.contains("o3-mini") {
            Some(Self::openai_o3_mini())
        } else if m.contains("o3") {
            Some(Self::openai_o3())
        } else if m.contains("o1-preview") {
            Some(Self::openai_o1_preview())
        } else if m.contains("o1-mini") {
            Some(Self::openai_o1_mini())
        } else if m.contains("o1") {
            Some(Self::openai_o1())
        } else if m.contains("gpt-4o-mini") {
            Some(Self::openai_gpt_4o_mini())
        } else if m.contains("gpt-4o") {
            Some(Self::openai_gpt_4o())
        } else if m.contains("gpt-4-turbo") {
            Some(Self::openai_gpt_4_turbo())
        } else if m.contains("gpt-4") {
            Some(Self::openai_gpt_4())
        } else if m.contains("gpt-3.5") {
            Some(Self::openai_gpt_3_5_turbo())
        } else if m.contains("fable") || m.contains("claude-fable") {
            Some(Self::claude_fable_5_1())
        } else if m.contains("opus-5-5") || m.contains("claude-opus-5") || m.contains("claude-5-5-opus") {
            Some(Self::claude_opus_5_5())
        } else if m.contains("haiku-5-5") || m.contains("claude-haiku-5") || m.contains("claude-5-5-haiku") {
            Some(Self::claude_haiku_5_5())
        } else if m.contains("sonnet-5-5") || m.contains("claude-sonnet-5") || m.contains("claude-5-5-sonnet") || m.contains("claude-5-5") {
            Some(Self::claude_sonnet_5_5())
        } else if m.contains("claude-3-7") || m.contains("claude-3.7") {
            Some(Self::claude_3_7_sonnet())
        } else if m.contains("claude-3-5-sonnet") || m.contains("claude-3.5-sonnet") {
            Some(Self::claude_3_5_sonnet())
        } else if m.contains("claude-3-5-haiku") || m.contains("claude-3.5-haiku") {
            Some(Self::claude_3_5_haiku())
        } else if m.contains("claude-3-opus") || m.contains("claude-opus") {
            Some(Self::claude_3_opus())
        } else if m.contains("claude-3-sonnet") {
            Some(Self::claude_3_sonnet())
        } else if m.contains("claude-3-haiku") {
            Some(Self::claude_3_haiku())
        } else if m.contains("mistral-large-4") || m.contains("chonk") {
            Some(Self::mistral_large_4())
        } else if m.contains("mistral-medium-3.5") || m.contains("medium-3.5") {
            Some(Self::mistral_medium_3_5())
        } else if m.contains("mistral-large-3") || m.contains("mistral-large-2411") {
            Some(Self::mistral_large_3())
        } else if m.contains("mistral-small-4") {
            Some(Self::mistral_small_4())
        } else if m.contains("mistral-small-2501") {
            Some(Self::mistral_small_2501())
        } else if m.contains("ministral-3-14b") {
            Some(Self::ministral_3_14b())
        } else if m.contains("ministral-3-8b") {
            Some(Self::ministral_3_8b())
        } else if m.contains("ministral-3-3b") {
            Some(Self::ministral_3_3b())
        } else if m.contains("codestral") {
            Some(Self::mistral_codestral())
        } else if m.contains("pixtral") {
            Some(Self::pixtral_12b())
        } else if m.contains("deepseek-v4-pro") || m.contains("v4-pro") {
            Some(Self::deepseek_v4_pro())
        } else if m.contains("deepseek-v4") || m.contains("deepseek-flash") {
            Some(Self::deepseek_flash())
        } else if m.contains("deepseek-r1") || m.contains("reasoner") {
            Some(Self::deepseek_r1())
        } else if m.contains("deepseek-coder") {
            Some(Self::deepseek_coder())
        } else if m.contains("deepseek") {
            Some(Self::deepseek_v3())
        } else if m.contains("glm-5.3-flashx") {
            Some(Self::glm_5_3_flashx())
        } else if m.contains("glm-5.3-flash") {
            Some(Self::glm_5_3_flash())
        } else if m.contains("glm-5.3") || m.contains("glm-5") {
            Some(Self::glm_5_3())
        } else if m.contains("glm-4-plus") {
            Some(Self::glm_4_plus())
        } else if m.contains("glm-4-air") {
            Some(Self::glm_4_air())
        } else if m.contains("glm-4-long") {
            Some(Self::glm_4_long())
        } else if m.contains("glm-4.7-flash") || m.contains("glm-4") {
            Some(Self::glm_4_7_flash())
        } else if m.contains("glm-3") {
            Some(Self::glm_3_turbo())
        } else if m.contains("kimi-k3.1-pro-highspeed") {
            Some(Self::kimi_k3_1_pro_highspeed())
        } else if m.contains("kimi-k3.1") {
            Some(Self::kimi_k3_1_pro())
        } else if m.contains("kimi-k3") {
            Some(Self::kimi_k3())
        } else if m.contains("moonshot-v1-128k") {
            Some(Self::moonshot_v1_128k())
        } else if m.contains("moonshot-v1-32k") {
            Some(Self::moonshot_v1_32k())
        } else if m.contains("moonshot-v1-8k") || m.contains("moonshot-v1") {
            Some(Self::moonshot_v1_8k())
        } else if m.contains("llama-3.1-405b") {
            Some(Self::llama_3_1_405b())
        } else if m.contains("llama-3.1-70b") {
            Some(Self::llama_3_1_70b())
        } else if m.contains("llama-3.1-8b") {
            Some(Self::llama_3_1_8b())
        } else if m.contains("llama-3.2-90b") {
            Some(Self::llama_3_2_90b_vision())
        } else if m.contains("llama-3.2-11b") {
            Some(Self::llama_3_2_11b_vision())
        } else if m.contains("llama-3.2-3b") {
            Some(Self::llama_3_2_3b())
        } else if m.contains("llama-3.2-1b") {
            Some(Self::llama_3_2_1b())
        } else if m.contains("llama-3.3") || m.contains("llama") {
            Some(Self::llama_3_3_70b())
        } else if m.contains("qwen-2.5-coder-7b") {
            Some(Self::qwen_2_5_coder_7b())
        } else if m.contains("qwen-2.5-coder") {
            Some(Self::qwen_2_5_coder_32b())
        } else if m.contains("qwen-2.5-72b") {
            Some(Self::qwen_2_5_72b())
        } else if m.contains("qwen-2.5-32b") {
            Some(Self::qwen_2_5_32b())
        } else if m.contains("qwen-2.5-14b") {
            Some(Self::qwen_2_5_14b())
        } else if m.contains("qwen-2.5-7b") {
            Some(Self::qwen_2_5_7b())
        } else if m.contains("qwen-2.5-math") {
            Some(Self::qwen_2_5_math_72b())
        } else if m.contains("qwq") {
            Some(Self::qwq_32b_preview())
        } else if m.contains("ollama") || m.contains("vllm") || m.contains("local") {
            Some(Self::local_zero())
        } else {
            None
        }
    }
}

/// Comprehensive descriptor for a model in the 2026 inference catalog.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelCatalogEntry {
    pub id: &'static str,
    pub provider: &'static str,
    pub name: &'static str,
    pub standard: Standard,
    pub capability: &'static str,
    pub context_window: u64,
    pub max_output_tokens: u64,
    pub supports_thinking: bool,
    pub default_thinking_budget: Option<u32>,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub supports_tools: bool,
    pub supports_caching: bool,
    pub rate_card: RateCard,
    pub aliases: &'static [&'static str],
    pub deprecated: bool,
    pub deprecation_notice: Option<&'static str>,
    pub replacement_id: Option<&'static str>,
}

impl ModelCatalogEntry {
    pub fn accepts(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if self.id.to_lowercase() == q || self.name.to_lowercase() == q {
            return true;
        }
        self.aliases.iter().any(|&a| a.to_lowercase() == q)
    }

    pub fn is_deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn is_active(&self) -> bool {
        !self.deprecated
    }
}

/// Exhaustive built-in model catalog for all supported cloud and local providers (October 2026).
pub fn default_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        // =====================================================================
        // 1. Google Gemini
        // =====================================================================
        ModelCatalogEntry {
            id: "gemini-4-argon",
            provider: "google",
            name: "Gemini 4 Argon",
            standard: Standard::Gemini,
            capability: "reasoning,agentic,code,math,science,multimodal,unified_1m",
            context_window: 1_048_576,
            max_output_tokens: 1_048_576,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_4_argon(),
            aliases: &[
                "gemini-4-argon-preview",
                "gemini-4-argon-latest",
                "gemini-4.0-argon",
                "models/gemini-4-argon",
                "models/gemini-4-argon-preview",
                "gemini-4",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-3.8-flash",
            provider: "google",
            name: "Gemini 3.8 Flash",
            standard: Standard::Gemini,
            capability: "general,reasoning,agentic,code,long_horizon",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_3_8_flash(),
            aliases: &[
                "gemini-3.8-flash-preview",
                "gemini-3.8-flash-latest",
                "models/gemini-3.8-flash",
                "gemini-3.8",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-3.5-flash-lite",
            provider: "google",
            name: "Gemini 3.5 Flash-Lite",
            standard: Standard::Gemini,
            capability: "fast,agentic,high_throughput",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(8_192),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_3_5_flash_lite(),
            aliases: &[
                "gemini-3.5-flash-lite-preview",
                "gemini-3.5-flash-lite-latest",
                "models/gemini-3.5-flash-lite",
                "gemini-3.5",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-3.1-pro",
            provider: "google",
            name: "Gemini 3.1 Pro",
            standard: Standard::Gemini,
            capability: "reasoning,complex,code,math",
            context_window: 2_097_152,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_3_1_pro(),
            aliases: &[
                "gemini-3.1-pro-preview",
                "gemini-3.1-pro-latest",
                "models/gemini-3.1-pro",
                "gemini-3-pro",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-3.8-live",
            provider: "google",
            name: "Gemini 3.8 Live",
            standard: Standard::Gemini,
            capability: "conversational,native_audio,realtime",
            context_window: 524_288,
            max_output_tokens: 8_192,
            supports_thinking: true,
            default_thinking_budget: Some(4_096),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_3_8_live(),
            aliases: &[
                "gemini-3.8-live-preview",
                "gemini-live-2026",
                "models/gemini-3.8-live",
                "gemini-live",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-2.5-pro",
            provider: "google",
            name: "Gemini 2.5 Pro",
            standard: Standard::Gemini,
            capability: "reasoning,coding,multimodal_production",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_2_5_pro(),
            aliases: &[
                "gemini-2.5-pro-latest",
                "gemini-2.5-pro-exp-0205",
                "models/gemini-2.5-pro",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-2.5-flash",
            provider: "google",
            name: "Gemini 2.5 Flash",
            standard: Standard::Gemini,
            capability: "fast,multimodal,efficient",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(8_192),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_2_5_flash(),
            aliases: &["gemini-2.5-flash-latest", "models/gemini-2.5-flash"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-2.0-flash",
            provider: "google",
            name: "Gemini 2.0 Flash",
            standard: Standard::Gemini,
            capability: "fast,multimodal_general",
            context_window: 1_048_576,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_2_0_flash(),
            aliases: &[
                "gemini-2.0-flash-001",
                "gemini-2.0-flash-latest",
                "gemini-2.0-flash-exp",
                "models/gemini-2.0-flash",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gemini-1.5-pro",
            provider: "google",
            name: "Gemini 1.5 Pro (Deprecated)",
            standard: Standard::Gemini,
            capability: "legacy_frontier,code,long_context",
            context_window: 2_097_152,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_1_5_pro(),
            aliases: &[
                "gemini-1.5-pro-latest",
                "gemini-1.5-pro-001",
                "gemini-1.5-pro-002",
                "models/gemini-1.5-pro",
            ],
            deprecated: true,
            deprecation_notice: Some(
                "Deprecated mid-2026; superseded by gemini-2.5-pro and gemini-4-argon",
            ),
            replacement_id: Some("gemini-4-argon"),
        },
        ModelCatalogEntry {
            id: "gemini-1.5-flash",
            provider: "google",
            name: "Gemini 1.5 Flash (Deprecated)",
            standard: Standard::Gemini,
            capability: "legacy_fast,multimodal",
            context_window: 1_048_576,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_1_5_flash(),
            aliases: &[
                "gemini-1.5-flash-latest",
                "gemini-1.5-flash-001",
                "gemini-1.5-flash-002",
                "gemini-1.5-flash-8b",
                "models/gemini-1.5-flash",
            ],
            deprecated: true,
            deprecation_notice: Some(
                "Deprecated mid-2026; superseded by gemini-2.5-flash and gemini-3.5-flash-lite",
            ),
            replacement_id: Some("gemini-2.5-flash"),
        },
        ModelCatalogEntry {
            id: "gemini-1.0-pro",
            provider: "google",
            name: "Gemini 1.0 Pro (Sunset)",
            standard: Standard::Gemini,
            capability: "sunset_text",
            context_window: 32_768,
            max_output_tokens: 2_048,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::gemini_1_0_pro(),
            aliases: &[
                "gemini-1.0-pro-latest",
                "gemini-pro",
                "models/gemini-1.0-pro",
                "models/gemini-pro",
            ],
            deprecated: true,
            deprecation_notice: Some("Sunset 2025; superseded by gemini-2.5-flash and gemini-4-argon"),
            replacement_id: Some("gemini-4-argon"),
        },

        // =====================================================================
        // 2. OpenAI
        // =====================================================================
        ModelCatalogEntry {
            id: "gpt-6-astra",
            provider: "openai",
            name: "GPT-6 Astra",
            standard: Standard::OpenAiChat,
            capability: "frontier_intelligence,scientific,reasoning,autonomous_agent",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_6_astra(),
            aliases: &["gpt-6-astra-2026-10-15", "gpt-6", "gpt-6-latest", "astra"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-6.1-sol",
            provider: "openai",
            name: "GPT-6.1 Sol",
            standard: Standard::OpenAiChat,
            capability: "frontier_reasoning,agentic_orchestration,deep_coding",
            context_window: 1_048_576,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_6_1_sol(),
            aliases: &["gpt-6.1-sol-2026-09-01", "gpt-6-sol", "gpt-6.1-sol-preview", "sol"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-6.1-terra",
            provider: "openai",
            name: "GPT-6.1 Terra",
            standard: Standard::OpenAiChat,
            capability: "visual_reasoning,computer_use,fast_execution",
            context_window: 1_048_576,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_6_1_terra(),
            aliases: &[
                "gpt-6.1-terra-2026-08-15",
                "gpt-6-terra",
                "gpt-6.1-terra-preview",
                "terra",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-6-luna",
            provider: "openai",
            name: "GPT-6 Luna",
            standard: Standard::OpenAiChat,
            capability: "high_volume,cost_sensitive,fast_agentic",
            context_window: 524_288,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_6_luna(),
            aliases: &["gpt-6-luna-2026-10-01", "gpt-6-mini", "gpt-6-luna-latest", "luna"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-realtime-2.1",
            provider: "openai",
            name: "GPT-Realtime 2.1",
            standard: Standard::OpenAiChat,
            capability: "realtime_voice,speech_to_speech,native_audio",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_realtime_2_1(),
            aliases: &[
                "gpt-realtime-2.1-2026-09-15",
                "gpt-realtime-latest",
                "gpt-4o-realtime-preview",
                "realtime-2.1",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-realtime-2.1-mini",
            provider: "openai",
            name: "GPT-Realtime 2.1 Mini",
            standard: Standard::OpenAiChat,
            capability: "realtime_voice_mini,fast_audio",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_realtime_2_1_mini(),
            aliases: &[
                "gpt-realtime-2.1-mini-2026-09-15",
                "gpt-4o-mini-realtime-preview",
                "realtime-mini",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "o3",
            provider: "openai",
            name: "o3",
            standard: Standard::OpenAiChat,
            capability: "deep_reasoning,math,physics,code",
            context_window: 200_000,
            max_output_tokens: 100_000,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_o3(),
            aliases: &["o3-2026-06-15", "o3-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "o3-pro",
            provider: "openai",
            name: "o3-pro",
            standard: Standard::OpenAiChat,
            capability: "ultra_deep_reasoning,formal_verification,massive_refactor",
            context_window: 200_000,
            max_output_tokens: 100_000,
            supports_thinking: true,
            default_thinking_budget: Some(65_536),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_o3_pro(),
            aliases: &["o3-pro-2026-08-01", "o3-pro-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "o4-mini",
            provider: "openai",
            name: "o4-mini",
            standard: Standard::OpenAiChat,
            capability: "next_gen_fast_reasoning,multimodal_stem",
            context_window: 200_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_o4_mini(),
            aliases: &["o4-mini-2026-09-15", "o4-mini-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "o3-mini",
            provider: "openai",
            name: "o3-mini",
            standard: Standard::OpenAiChat,
            capability: "fast_reasoning,stem,code",
            context_window: 200_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_o3_mini(),
            aliases: &["o3-mini-2025-01-31", "o3-mini-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "o1",
            provider: "openai",
            name: "o1",
            standard: Standard::OpenAiChat,
            capability: "deep_reasoning_v1,science,code",
            context_window: 200_000,
            max_output_tokens: 100_000,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_o1(),
            aliases: &["o1-2024-12-17", "o1-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "o1-preview",
            provider: "openai",
            name: "o1-preview (Deprecated)",
            standard: Standard::OpenAiChat,
            capability: "early_reasoning_preview",
            context_window: 128_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::openai_o1_preview(),
            aliases: &["o1-preview-2024-09-12"],
            deprecated: true,
            deprecation_notice: Some("Deprecated late 2024; superseded by o1 and o3"),
            replacement_id: Some("o3"),
        },
        ModelCatalogEntry {
            id: "o1-mini",
            provider: "openai",
            name: "o1-mini (Deprecated)",
            standard: Standard::OpenAiChat,
            capability: "early_fast_reasoning",
            context_window: 128_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::openai_o1_mini(),
            aliases: &["o1-mini-2024-09-12"],
            deprecated: true,
            deprecation_notice: Some("Deprecated mid-2025; superseded by o3-mini and o4-mini"),
            replacement_id: Some("o4-mini"),
        },
        ModelCatalogEntry {
            id: "gpt-4o",
            provider: "openai",
            name: "GPT-4o",
            standard: Standard::OpenAiChat,
            capability: "general,multimodal,code",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_gpt_4o(),
            aliases: &[
                "gpt-4o-2024-11-20",
                "gpt-4o-2024-08-06",
                "gpt-4o-2024-05-13",
                "gpt-4o-latest",
                "chatgpt-4o-latest",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-4o-mini",
            provider: "openai",
            name: "GPT-4o mini",
            standard: Standard::OpenAiChat,
            capability: "fast,general",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::openai_gpt_4o_mini(),
            aliases: &["gpt-4o-mini-2024-07-18", "gpt-4o-mini-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "gpt-4-turbo",
            provider: "openai",
            name: "GPT-4 Turbo (Deprecated)",
            standard: Standard::OpenAiChat,
            capability: "legacy_high_capability",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::openai_gpt_4_turbo(),
            aliases: &[
                "gpt-4-turbo-2024-04-09",
                "gpt-4-0125-preview",
                "gpt-4-1106-preview",
                "gpt-4-turbo-preview",
            ],
            deprecated: true,
            deprecation_notice: Some("Deprecated late 2024; superseded by gpt-4o and gpt-6.1-terra"),
            replacement_id: Some("gpt-4o"),
        },
        ModelCatalogEntry {
            id: "gpt-4",
            provider: "openai",
            name: "GPT-4 (Deprecated)",
            standard: Standard::OpenAiChat,
            capability: "legacy_flagship",
            context_window: 8_192,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::openai_gpt_4(),
            aliases: &["gpt-4-0613", "gpt-4-0314"],
            deprecated: true,
            deprecation_notice: Some("Deprecated mid-2025; superseded by gpt-4o and gpt-6.1-sol"),
            replacement_id: Some("gpt-4o"),
        },
        ModelCatalogEntry {
            id: "gpt-3.5-turbo",
            provider: "openai",
            name: "GPT-3.5 Turbo (Sunset)",
            standard: Standard::OpenAiChat,
            capability: "legacy_lightweight",
            context_window: 16_385,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::openai_gpt_3_5_turbo(),
            aliases: &[
                "gpt-3.5-turbo-0125",
                "gpt-3.5-turbo-1106",
                "gpt-3.5-turbo-0613",
                "gpt-3.5-turbo-16k",
            ],
            deprecated: true,
            deprecation_notice: Some("Sunset 2025; superseded by gpt-4o-mini and gpt-6-luna"),
            replacement_id: Some("gpt-4o-mini"),
        },

        // =====================================================================
        // 3. Anthropic (Claude 5.x Series & Claude 3.x Series)
        // =====================================================================
        ModelCatalogEntry {
            id: "claude-fable-5-1",
            provider: "anthropic",
            name: "Claude Fable 5.1",
            standard: Standard::AnthropicMessages,
            capability: "frontier_reasoning,long_horizon_synthesis,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 128_000,
            supports_thinking: true,
            default_thinking_budget: Some(65_536),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_fable_5_1(),
            aliases: &[
                "claude-fable-5-1-20261001",
                "claude-5-1-fable",
                "claude-fable",
                "claude-fable-latest",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-opus-5-5",
            provider: "anthropic",
            name: "Claude Opus 5.5",
            standard: Standard::AnthropicMessages,
            capability: "deepest_agentic_reasoning,architecture_synthesis,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 128_000,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_opus_5_5(),
            aliases: &[
                "claude-opus-5-5-20261001",
                "claude-opus-5.5",
                "claude-5-5-opus",
                "claude-5-5-opus-20261001",
                "claude-opus-5-5-latest",
                "claude-5-5-opus-latest",
                "claude-opus-5",
                "claude-5-opus",
                "claude-opus-latest",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-sonnet-5-5",
            provider: "anthropic",
            name: "Claude Sonnet 5.5",
            standard: Standard::AnthropicMessages,
            capability: "frontier_engineering,agentic_workhorse,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 128_000,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_sonnet_5_5(),
            aliases: &[
                "claude-sonnet-5-5-20260915",
                "claude-sonnet-5.5",
                "claude-5-5-sonnet",
                "claude-5-5-sonnet-20260915",
                "claude-sonnet-5-5-latest",
                "claude-5-5-sonnet-latest",
                "claude-sonnet-5",
                "claude-5-sonnet",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-haiku-5-5",
            provider: "anthropic",
            name: "Claude Haiku 5.5",
            standard: Standard::AnthropicMessages,
            capability: "lightning_fast_agentic,near_instant,500k_context",
            context_window: 500_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_haiku_5_5(),
            aliases: &[
                "claude-haiku-5-5-20261001",
                "claude-haiku-5.5",
                "claude-5-5-haiku",
                "claude-5-5-haiku-latest",
                "claude-haiku-5",
                "claude-5-haiku",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-3-7-sonnet",
            provider: "anthropic",
            name: "Claude 3.7 Sonnet",
            standard: Standard::AnthropicMessages,
            capability: "hybrid_reasoning_coding,extended_thinking",
            context_window: 200_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_7_sonnet(),
            aliases: &[
                "claude-3-7-sonnet-20250219",
                "claude-3-7-sonnet-latest",
                "claude-3.7-sonnet",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-3-5-sonnet",
            provider: "anthropic",
            name: "Claude 3.5 Sonnet",
            standard: Standard::AnthropicMessages,
            capability: "production_coding,complex_agentic",
            context_window: 200_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_5_sonnet(),
            aliases: &[
                "claude-3-5-sonnet-20241022",
                "claude-3-5-sonnet-20240620",
                "claude-3-5-sonnet-latest",
                "claude-3.5-sonnet",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-3-5-haiku",
            provider: "anthropic",
            name: "Claude 3.5 Haiku",
            standard: Standard::AnthropicMessages,
            capability: "fast,economical",
            context_window: 200_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_5_haiku(),
            aliases: &[
                "claude-3-5-haiku-20241022",
                "claude-3-5-haiku-latest",
                "claude-3.5-haiku",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "claude-3-opus",
            provider: "anthropic",
            name: "Claude 3 Opus (Deprecated)",
            standard: Standard::AnthropicMessages,
            capability: "legacy_deep_reasoning",
            context_window: 200_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_opus(),
            aliases: &[
                "claude-3-opus-20240229",
                "claude-3-opus-latest",
                "claude-3.0-opus",
            ],
            deprecated: true,
            deprecation_notice: Some(
                "Deprecated mid-2025; superseded by claude-opus-5-5 and claude-fable-5-1",
            ),
            replacement_id: Some("claude-opus-5-5"),
        },
        ModelCatalogEntry {
            id: "claude-3-sonnet",
            provider: "anthropic",
            name: "Claude 3 Sonnet (Sunset)",
            standard: Standard::AnthropicMessages,
            capability: "legacy_balanced",
            context_window: 200_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::claude_3_sonnet(),
            aliases: &["claude-3-sonnet-20240229", "claude-3.0-sonnet"],
            deprecated: true,
            deprecation_notice: Some(
                "Sunset late 2024; superseded by claude-3-5-sonnet and claude-sonnet-5-5",
            ),
            replacement_id: Some("claude-sonnet-5-5"),
        },
        ModelCatalogEntry {
            id: "claude-3-haiku",
            provider: "anthropic",
            name: "Claude 3 Haiku (Sunset)",
            standard: Standard::AnthropicMessages,
            capability: "legacy_compact",
            context_window: 200_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::claude_3_haiku(),
            aliases: &["claude-3-haiku-20240307", "claude-3.0-haiku"],
            deprecated: true,
            deprecation_notice: Some("Sunset 2025; superseded by claude-3-5-haiku and claude-haiku-5-5"),
            replacement_id: Some("claude-haiku-5-5"),
        },

        // =====================================================================
        // 4. Mistral AI
        // =====================================================================
        ModelCatalogEntry {
            id: "mistral-large-4",
            provider: "mistral",
            name: "Mistral Large 4 (Le Chonk)",
            standard: Standard::Mistral,
            capability: "frontier_moe,agentic,code,math,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_large_4(),
            aliases: &[
                "mistral-large-4-2610",
                "mistral-large-latest",
                "mistral-large-4-latest",
                "open-mistral-large-4",
                "chonk",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "mistral-medium-3.5",
            provider: "mistral",
            name: "Mistral Medium 3.5",
            standard: Standard::Mistral,
            capability: "high_efficiency_reasoning,agentic,512k_context",
            context_window: 512_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(8_192),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_medium_3_5(),
            aliases: &[
                "mistral-medium-3.5-latest",
                "mistral-medium-latest",
                "mistral-medium-3",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "mistral-large-2411",
            provider: "mistral",
            name: "Mistral Large 3 (2411)",
            standard: Standard::Mistral,
            capability: "flagship_general,code,multilingual",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_large_3(),
            aliases: &[
                "mistral-large-2411-latest",
                "mistral-large-2407",
                "mistral-large-3",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "mistral-small-4",
            provider: "mistral",
            name: "Mistral Small 4",
            standard: Standard::Mistral,
            capability: "compact_reasoning,agentic_workhorse,256k_context",
            context_window: 256_000,
            max_output_tokens: 16_384,
            supports_thinking: true,
            default_thinking_budget: Some(4_096),
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_small_4(),
            aliases: &[
                "mistral-small-4-latest",
                "mistral-small-latest",
                "mistral-small",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "mistral-small-2501",
            provider: "mistral",
            name: "Mistral Small 2501",
            standard: Standard::Mistral,
            capability: "lightweight,fast,responsive",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_small_2501(),
            aliases: &[
                "mistral-small-2501-latest",
                "mistral-small-2409",
                "mistral-small-2402",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "ministral-3-14b",
            provider: "mistral",
            name: "Ministral 3 14B",
            standard: Standard::Mistral,
            capability: "edge_powerhouse,agentic_slm",
            context_window: 256_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::ministral_3_14b(),
            aliases: &[
                "ministral-3-14b-latest",
                "ministral-14b-latest",
                "ministral-14b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "ministral-3-8b",
            provider: "mistral",
            name: "Ministral 3 8B",
            standard: Standard::Mistral,
            capability: "on_device_edge,fast_retrieval",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::ministral_3_8b(),
            aliases: &[
                "ministral-3-8b-latest",
                "ministral-8b-latest",
                "ministral-8b-2410",
                "ministral-8b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "ministral-3-3b",
            provider: "mistral",
            name: "Ministral 3 3B",
            standard: Standard::Mistral,
            capability: "ultra_light_edge,mobile",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::ministral_3_3b(),
            aliases: &[
                "ministral-3-3b-latest",
                "ministral-3b-latest",
                "ministral-3b-2410",
                "ministral-3b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "codestral-2501",
            provider: "mistral",
            name: "Codestral 2501",
            standard: Standard::Mistral,
            capability: "code_specialist,fill_in_the_middle,repository",
            context_window: 256_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_codestral(),
            aliases: &["codestral-latest", "codestral-2405", "codestral"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "pixtral-12b",
            provider: "mistral",
            name: "Pixtral 12B",
            standard: Standard::Mistral,
            capability: "vision_multimodal,document_analysis",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::pixtral_12b(),
            aliases: &["pixtral-12b-2409", "pixtral-12b-latest", "pixtral"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "open-mixtral-8x22b",
            provider: "mistral",
            name: "Mixtral 8x22B (Legacy)",
            standard: Standard::Mistral,
            capability: "legacy_open_moe",
            context_window: 65_536,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::open_mixtral_8x22b(),
            aliases: &["mixtral-8x22b", "open-mixtral-8x22b-latest"],
            deprecated: true,
            deprecation_notice: Some(
                "Legacy model; superseded by mistral-large-4 and mistral-medium-3.5",
            ),
            replacement_id: Some("mistral-large-4"),
        },
        ModelCatalogEntry {
            id: "open-mixtral-8x7b",
            provider: "mistral",
            name: "Mixtral 8x7B (Legacy)",
            standard: Standard::Mistral,
            capability: "legacy_moe",
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::open_mixtral_8x7b(),
            aliases: &["mixtral-8x7b"],
            deprecated: true,
            deprecation_notice: Some(
                "Legacy model; superseded by ministral-3-14b and mistral-small-4",
            ),
            replacement_id: Some("ministral-3-14b"),
        },

        // =====================================================================
        // 5. DeepSeek
        // =====================================================================
        ModelCatalogEntry {
            id: "deepseek-flash",
            provider: "deepseek",
            name: "DeepSeek-V4.1 Flash",
            standard: Standard::DeepSeek,
            capability: "high_throughput,1m_context,dynamic_peak_rates",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_flash(),
            aliases: &[
                "deepseek-v4.1-flash",
                "deepseek-v4.1-flash-1017",
                "deepseek-flash-latest",
                "deepseek-v4-flash",
                "deepseek/deepseek-v4.1-flash",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "deepseek-v4-pro",
            provider: "deepseek",
            name: "DeepSeek-V4 Pro",
            standard: Standard::DeepSeek,
            capability: "frontier_agentic,multimodal,1m_context,dynamic_peak_rates",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_v4_pro(),
            aliases: &[
                "deepseek-v4-pro-0813",
                "deepseek-v4-pro-latest",
                "deepseek-pro",
                "deepseek/deepseek-v4-pro",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "deepseek-reasoner",
            provider: "deepseek",
            name: "DeepSeek Reasoner (R1)",
            standard: Standard::DeepSeek,
            capability: "chain_of_thought,math,formal_logic,code",
            context_window: 64_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_r1(),
            aliases: &[
                "deepseek-r1",
                "deepseek-reasoner-latest",
                "deepseek-r1-latest",
                "deepseek/deepseek-r1",
                "deepseek-r1-distill-qwen-32b",
                "deepseek-r1-distill-llama-70b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "deepseek-chat",
            provider: "deepseek",
            name: "DeepSeek Chat (V3)",
            standard: Standard::DeepSeek,
            capability: "general_dialog,code,multilingual",
            context_window: 64_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_v3(),
            aliases: &[
                "deepseek-v3",
                "deepseek-chat-latest",
                "deepseek-v3-latest",
                "deepseek/deepseek-v3",
                "deepseek-v2.5",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "deepseek-coder",
            provider: "deepseek",
            name: "DeepSeek Coder (Deprecated)",
            standard: Standard::DeepSeek,
            capability: "legacy_coding",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::deepseek_coder(),
            aliases: &[
                "deepseek-coder-v2",
                "deepseek-coder-33b",
                "deepseek-coder-6.7b",
            ],
            deprecated: true,
            deprecation_notice: Some(
                "Deprecated mid-2025; superseded by deepseek-chat (V3) and deepseek-flash",
            ),
            replacement_id: Some("deepseek-chat"),
        },

        // =====================================================================
        // 6. Zhipu AI (GLM)
        // =====================================================================
        ModelCatalogEntry {
            id: "glm-5.3",
            provider: "zhipu",
            name: "GLM-5.3",
            standard: Standard::Zhipu,
            capability: "frontier_bilingual,complex_agentic,math,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::glm_5_3(),
            aliases: &["glm-5.3-latest", "glm-5", "glm-5-pro"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-5.3-flash",
            provider: "zhipu",
            name: "GLM-5.3 Flash",
            standard: Standard::Zhipu,
            capability: "high_speed,low_latency,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::glm_5_3_flash(),
            aliases: &["glm-5.3-flash-latest", "glm-5-flash"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-5.3-flashx",
            provider: "zhipu",
            name: "GLM-5.3 FlashX",
            standard: Standard::Zhipu,
            capability: "enterprise_fast,enhanced_reasoning,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::glm_5_3_flashx(),
            aliases: &["glm-5.3-flashx-latest", "glm-5-flashx"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-4-plus",
            provider: "zhipu",
            name: "GLM-4 Plus",
            standard: Standard::Zhipu,
            capability: "bilingual_flagship,multimodal",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::glm_4_plus(),
            aliases: &["glm-4-plus-latest", "glm-4-0520", "glm-4"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-4-air",
            provider: "zhipu",
            name: "GLM-4 Air",
            standard: Standard::Zhipu,
            capability: "fast,economical",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::glm_4_air(),
            aliases: &["glm-4-air-latest", "glm-4-airx"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-4-long",
            provider: "zhipu",
            name: "GLM-4 Long",
            standard: Standard::Zhipu,
            capability: "long_context_retrieval,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::glm_4_long(),
            aliases: &["glm-4-long-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-4.7-flash",
            provider: "zhipu",
            name: "GLM-4.7 Flash (Free)",
            standard: Standard::Zhipu,
            capability: "free_tier,high_throughput",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::glm_4_7_flash(),
            aliases: &["glm-4.7-flash-latest", "glm-4-flash"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "glm-3-turbo",
            provider: "zhipu",
            name: "GLM-3 Turbo (Sunset)",
            standard: Standard::Zhipu,
            capability: "legacy_fast",
            context_window: 32_768,
            max_output_tokens: 2_048,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::glm_3_turbo(),
            aliases: &["glm-3turbo"],
            deprecated: true,
            deprecation_notice: Some(
                "Sunset 2024; superseded by glm-4.7-flash and glm-5.3-flash",
            ),
            replacement_id: Some("glm-5.3-flash"),
        },

        // =====================================================================
        // 7. Moonshot AI (Kimi)
        // =====================================================================
        ModelCatalogEntry {
            id: "kimi-k3.1-pro",
            provider: "moonshot",
            name: "Kimi K3.1 Pro",
            standard: Standard::Moonshot,
            capability: "long_horizon_reasoning,agentic,deep_synthesis,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::kimi_k3_1_pro(),
            aliases: &["kimi-k3.1-pro-latest", "kimi-k3.1", "kimi-k3-pro"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "kimi-k3.1-pro-highspeed",
            provider: "moonshot",
            name: "Kimi K3.1 Pro Highspeed",
            standard: Standard::Moonshot,
            capability: "prioritised_latency,enterprise_throughput,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::kimi_k3_1_pro_highspeed(),
            aliases: &["kimi-k3.1-highspeed", "kimi-k3-highspeed"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "kimi-k3",
            provider: "moonshot",
            name: "Kimi K3",
            standard: Standard::Moonshot,
            capability: "deep_reasoning,research,500k_context",
            context_window: 500_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::kimi_k3(),
            aliases: &["kimi-k3-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "moonshot-v1-128k",
            provider: "moonshot",
            name: "Moonshot v1 128k",
            standard: Standard::Moonshot,
            capability: "long_document_analysis",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::moonshot_v1_128k(),
            aliases: &["moonshot-v1-128k-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "moonshot-v1-32k",
            provider: "moonshot",
            name: "Moonshot v1 32k",
            standard: Standard::Moonshot,
            capability: "medium_document_analysis",
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::moonshot_v1_32k(),
            aliases: &["moonshot-v1-32k-latest"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "moonshot-v1-8k",
            provider: "moonshot",
            name: "Moonshot v1 8k",
            standard: Standard::Moonshot,
            capability: "general_dialog,short_context",
            context_window: 8_192,
            max_output_tokens: 2_048,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::moonshot_v1_8k(),
            aliases: &["moonshot-v1-8k-latest", "moonshot-v1-auto"],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },

        // =====================================================================
        // 8. Meta Llama
        // =====================================================================
        ModelCatalogEntry {
            id: "llama-3.3-70b-instruct",
            provider: "meta",
            name: "Llama 3.3 70B Instruct",
            standard: Standard::OpenAiChat,
            capability: "open_weights_flagship,coding,stem",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_3_70b(),
            aliases: &[
                "meta-llama/Llama-3.3-70B-Instruct",
                "llama-3.3-70b",
                "meta-llama/llama-3.3-70b-instruct",
                "llama-3.3",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.1-405b-instruct",
            provider: "meta",
            name: "Llama 3.1 405B Instruct",
            standard: Standard::OpenAiChat,
            capability: "frontier_open_weights,synthetic_data,complex_code",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_1_405b(),
            aliases: &[
                "meta-llama/Meta-Llama-3.1-405B-Instruct",
                "llama-3.1-405b",
                "meta-llama/llama-3.1-405b-instruct",
                "llama-3.1-405b-fp8",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.1-70b-instruct",
            provider: "meta",
            name: "Llama 3.1 70B Instruct",
            standard: Standard::OpenAiChat,
            capability: "strong_open_model,general",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_1_70b(),
            aliases: &[
                "meta-llama/Meta-Llama-3.1-70B-Instruct",
                "llama-3.1-70b",
                "meta-llama/llama-3.1-70b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.1-8b-instruct",
            provider: "meta",
            name: "Llama 3.1 8B Instruct",
            standard: Standard::OpenAiChat,
            capability: "efficient_open_slm,lightweight",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_1_8b(),
            aliases: &[
                "meta-llama/Meta-Llama-3.1-8B-Instruct",
                "llama-3.1-8b",
                "meta-llama/llama-3.1-8b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.2-90b-vision-instruct",
            provider: "meta",
            name: "Llama 3.2 90B Vision Instruct",
            standard: Standard::OpenAiChat,
            capability: "open_multimodal_vision,document_visual",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_2_90b_vision(),
            aliases: &[
                "meta-llama/Llama-3.2-90B-Vision-Instruct",
                "llama-3.2-90b-vision",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.2-11b-vision-instruct",
            provider: "meta",
            name: "Llama 3.2 11B Vision Instruct",
            standard: Standard::OpenAiChat,
            capability: "efficient_multimodal_vision",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_2_11b_vision(),
            aliases: &[
                "meta-llama/Llama-3.2-11B-Vision-Instruct",
                "llama-3.2-11b-vision",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.2-3b-instruct",
            provider: "meta",
            name: "Llama 3.2 3B Instruct",
            standard: Standard::OpenAiChat,
            capability: "edge_mobile_open_slm",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_2_3b(),
            aliases: &[
                "meta-llama/Llama-3.2-3B-Instruct",
                "llama-3.2-3b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-3.2-1b-instruct",
            provider: "meta",
            name: "Llama 3.2 1B Instruct",
            standard: Standard::OpenAiChat,
            capability: "ultra_light_edge_open",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_2_1b(),
            aliases: &[
                "meta-llama/Llama-3.2-1B-Instruct",
                "llama-3.2-1b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "llama-2-70b-chat",
            provider: "meta",
            name: "Llama 2 70B Chat (Sunset)",
            standard: Standard::OpenAiChat,
            capability: "legacy_llama2",
            context_window: 4_096,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::llama_2_70b(),
            aliases: &["meta-llama/Llama-2-70b-chat-hf"],
            deprecated: true,
            deprecation_notice: Some(
                "Sunset 2024; superseded by llama-3.1-70b-instruct and llama-3.3-70b-instruct",
            ),
            replacement_id: Some("llama-3.3-70b-instruct"),
        },

        // =====================================================================
        // 9. Alibaba Qwen
        // =====================================================================
        ModelCatalogEntry {
            id: "qwen-2.5-coder-32b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 Coder 32B Instruct",
            standard: Standard::OpenAiChat,
            capability: "open_code_champion,repo_level,synthetic_code",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_coder_32b(),
            aliases: &[
                "qwen/Qwen2.5-Coder-32B-Instruct",
                "qwen-2.5-coder-32b",
                "qwen2.5-coder-32b-instruct",
                "qwen-coder-32b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2.5-coder-7b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 Coder 7B Instruct",
            standard: Standard::OpenAiChat,
            capability: "lightweight_code_slm",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_coder_7b(),
            aliases: &[
                "qwen/Qwen2.5-Coder-7B-Instruct",
                "qwen-2.5-coder-7b",
                "qwen2.5-coder-7b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2.5-72b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 72B Instruct",
            standard: Standard::OpenAiChat,
            capability: "open_multilingual_flagship,math,coding",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_72b(),
            aliases: &[
                "qwen/Qwen2.5-72B-Instruct",
                "qwen-2.5-72b",
                "qwen2.5-72b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2.5-32b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 32B Instruct",
            standard: Standard::OpenAiChat,
            capability: "balanced_general,strong_reasoning",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_32b(),
            aliases: &[
                "qwen/Qwen2.5-32B-Instruct",
                "qwen-2.5-32b",
                "qwen2.5-32b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2.5-14b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 14B Instruct",
            standard: Standard::OpenAiChat,
            capability: "compact_workhorse,fast",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_14b(),
            aliases: &[
                "qwen/Qwen2.5-14B-Instruct",
                "qwen-2.5-14b",
                "qwen2.5-14b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2.5-7b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 7B Instruct",
            standard: Standard::OpenAiChat,
            capability: "edge_slm,responsive",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_7b(),
            aliases: &[
                "qwen/Qwen2.5-7B-Instruct",
                "qwen-2.5-7b",
                "qwen2.5-7b-instruct",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2.5-math-72b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 Math 72B Instruct",
            standard: Standard::OpenAiChat,
            capability: "mathematics_specialist,competition_math",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_math_72b(),
            aliases: &[
                "qwen/Qwen2.5-Math-72B-Instruct",
                "qwen-math-72b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwq-32b-preview",
            provider: "qwen",
            name: "QwQ 32B Preview",
            standard: Standard::OpenAiChat,
            capability: "open_reasoning,math,formal_proofs",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwq_32b_preview(),
            aliases: &[
                "qwen/QwQ-32B-Preview",
                "qwq-32b",
                "qwq-32b-preview-latest",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "qwen-2-72b-instruct",
            provider: "qwen",
            name: "Qwen 2 72B Instruct (Sunset)",
            standard: Standard::OpenAiChat,
            capability: "legacy_qwen2",
            context_window: 32_768,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: false,
            supports_caching: false,
            rate_card: RateCard::qwen_2_72b(),
            aliases: &["qwen/Qwen2-72B-Instruct"],
            deprecated: true,
            deprecation_notice: Some("Sunset 2024; superseded by qwen-2.5-72b-instruct"),
            replacement_id: Some("qwen-2.5-72b-instruct"),
        },

        // =====================================================================
        // 10. Local Runtimes (Zero token cost, hardware amortisation)
        // =====================================================================
        ModelCatalogEntry {
            id: "local/llama3.3",
            provider: "local",
            name: "Local Llama 3.3 70B",
            standard: Standard::Ollama,
            capability: "local,private,zero_token_cost",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::local_zero(),
            aliases: &[
                "ollama/llama3.3",
                "vllm/llama3.3",
                "llama3.3:70b",
                "llama3.3:latest",
                "llama3.3",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "local/deepseek-r1",
            provider: "local",
            name: "Local DeepSeek R1",
            standard: Standard::Ollama,
            capability: "local,reasoning,private,zero_token_cost",
            context_window: 64_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::local_zero(),
            aliases: &[
                "ollama/deepseek-r1",
                "vllm/deepseek-r1",
                "deepseek-r1:70b",
                "deepseek-r1:32b",
                "deepseek-r1:14b",
                "deepseek-r1:8b",
                "deepseek-r1:1.5b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "local/qwen2.5-coder",
            provider: "local",
            name: "Local Qwen 2.5 Coder 32B",
            standard: Standard::Vllm,
            capability: "local,code_specialist,zero_token_cost",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::local_zero(),
            aliases: &[
                "ollama/qwen2.5-coder",
                "vllm/qwen2.5-coder",
                "qwen2.5-coder:32b",
                "qwen2.5-coder:14b",
                "qwen2.5-coder:7b",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "local/mistral-nemo",
            provider: "local",
            name: "Local Mistral NeMo 12B",
            standard: Standard::Ollama,
            capability: "local,multilingual_edge,zero_token_cost",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::local_zero(),
            aliases: &[
                "ollama/mistral-nemo",
                "vllm/mistral-nemo",
                "mistral-nemo:12b",
                "mistral-nemo:latest",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
        ModelCatalogEntry {
            id: "local/phi-4",
            provider: "local",
            name: "Local Phi-4 14B",
            standard: Standard::Ollama,
            capability: "local,reasoning_slm,zero_token_cost",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::local_zero(),
            aliases: &[
                "ollama/phi-4",
                "vllm/phi-4",
                "phi-4:14b",
                "phi-4:latest",
            ],
            deprecated: false,
            deprecation_notice: None,
            replacement_id: None,
        },
    ]
}

/// Find a catalog entry by ID, exact endpoint alias, snapshot string, or partial name.
pub fn catalog_entry(model_id: &str) -> Option<ModelCatalogEntry> {
    let catalog = default_catalog();
    let q = model_id.trim().to_lowercase();

    // 1. Exact match on id or any accepted alias
    if let Some(entry) = catalog.iter().find(|e| e.accepts(&q)) {
        return Some(entry.clone());
    }

    // 2. Normalized prefix / contains match
    catalog.into_iter().find(|e| {
        let id_clean = e.id.to_lowercase();
        q == id_clean
            || q.contains(&id_clean)
            || e.aliases.iter().any(|&a| {
                let a_clean = a.to_lowercase();
                q == a_clean || q.contains(&a_clean)
            })
    })
}

/// Return all active provider IDs.
pub fn all_providers() -> Vec<&'static str> {
    vec![
        "google", "openai", "anthropic", "mistral", "deepseek", "zhipu", "moonshot", "meta",
        "qwen", "local",
    ]
}

/// Return all models for a given provider.
pub fn models_by_provider(provider: &str) -> Vec<ModelCatalogEntry> {
    let p = provider.to_lowercase();
    default_catalog()
        .into_iter()
        .filter(|entry| entry.provider.to_lowercase() == p)
        .collect()
}

/// Return all currently active (non-deprecated) models.
pub fn active_models() -> Vec<ModelCatalogEntry> {
    default_catalog()
        .into_iter()
        .filter(|e| !e.deprecated)
        .collect()
}

/// Return all deprecated, legacy, or sunset models.
pub fn deprecated_models() -> Vec<ModelCatalogEntry> {
    default_catalog()
        .into_iter()
        .filter(|e| e.deprecated)
        .collect()
}

/// Resolve an endpoint alias to its canonical model ID.
pub fn resolve_endpoint_alias(alias: &str) -> Option<&'static str> {
    catalog_entry(alias).map(|e| e.id)
}

/// Return all accepted endpoint alias strings for a given model.
pub fn all_accepted_endpoint_names(model_id: &str) -> Vec<&'static str> {
    if let Some(entry) = catalog_entry(model_id) {
        let mut names = vec![entry.id];
        names.extend_from_slice(entry.aliases);
        names
    } else {
        Vec::new()
    }
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
        close(card.cost(&usage), 7.276);
    }

    #[test]
    fn deepseek_flash_and_v4_pro_peak_window() {
        let flash = RateCard::deepseek_flash();
        assert_eq!(flash.input, 0.15);
        assert_eq!(flash.output, 0.60);
        assert_eq!(flash.cache_read, 0.003);
        assert!(!flash.peak(at("2026-10-28T12:00:00Z")));
        assert!(flash.peak(at("2026-10-28T02:00:00Z")));

        let pro = RateCard::deepseek_v4_pro();
        assert_eq!(pro.input, 0.66);
        assert_eq!(pro.output, 1.98);
        assert_eq!(pro.cache_read, 0.022);
    }

    #[test]
    fn gemini_4_argon_and_gemini_3_models() {
        let argon = RateCard::gemini_4_argon();
        assert_eq!(argon.input, 2.00);
        assert_eq!(argon.output, 10.00);
        assert_eq!(argon.cache_read, 0.10);
        assert_eq!(RateCard::for_model("gemini-4-argon"), Some(argon.clone()));
        assert_eq!(RateCard::for_model("gemini-4.0-argon"), Some(argon.clone()));
        assert_eq!(RateCard::for_model("models/gemini-4-argon"), Some(argon));

        let g38 = RateCard::gemini_3_8_flash();
        assert_eq!(RateCard::for_model("gemini-3.8-flash"), Some(g38.clone()));
        assert_eq!(RateCard::for_model("gemini-3.8-flash-preview"), Some(g38));
    }

    #[test]
    fn mistral_large_4_and_ministral_models() {
        let m4 = RateCard::mistral_large_4();
        assert_eq!(m4.input, 0.70);
        assert_eq!(m4.output, 2.10);
        assert_eq!(RateCard::for_model("mistral-large-4"), Some(m4.clone()));
        assert_eq!(RateCard::for_model("mistral-large-4-latest"), Some(m4));

        let m14b = RateCard::ministral_3_14b();
        assert_eq!(m14b.input, 0.20);
        assert_eq!(m14b.output, 0.20);
        assert_eq!(RateCard::for_model("ministral-3-14b"), Some(m14b));
    }

    #[test]
    fn claude_fable_and_opus_5_5_series() {
        let fable = RateCard::claude_fable_5_1();
        assert_eq!(fable.input, 10.00);
        assert_eq!(fable.output, 50.00);
        assert_eq!(RateCard::for_model("claude-fable-5-1"), Some(fable));

        let opus55 = RateCard::claude_opus_5_5();
        assert_eq!(opus55.input, 4.00);
        assert_eq!(opus55.output, 20.00);
        assert_eq!(RateCard::for_model("claude-opus-5-5"), Some(opus55.clone()));
        assert_eq!(RateCard::for_model("claude-opus-5.5"), Some(opus55.clone()));
        assert_eq!(RateCard::for_model("claude-5-5-opus-20261001"), Some(opus55.clone()));
        assert_eq!(RateCard::for_model("claude-opus-5-5-latest"), Some(opus55));

        let sonnet55 = RateCard::claude_sonnet_5_5();
        assert_eq!(sonnet55.input, 2.00);
        assert_eq!(sonnet55.output, 10.00);
        assert_eq!(RateCard::for_model("claude-sonnet-5-5"), Some(sonnet55));
    }

    #[test]
    fn openai_gpt6_sol_terra_o3() {
        let sol = RateCard::gpt_6_1_sol();
        assert_eq!(sol.input, 2.00);
        assert_eq!(sol.output, 10.00);
        assert_eq!(RateCard::for_model("gpt-6.1-sol"), Some(sol));

        let terra = RateCard::gpt_6_1_terra();
        assert_eq!(terra.input, 1.00);
        assert_eq!(terra.output, 5.00);
        assert_eq!(RateCard::for_model("gpt-6.1-terra"), Some(terra));

        let o3 = RateCard::openai_o3();
        assert_eq!(o3.input, 2.00);
        assert_eq!(o3.output, 8.00);
        assert_eq!(RateCard::for_model("o3"), Some(o3));
    }

    #[test]
    fn catalog_completeness_and_filters() {
        let catalog = default_catalog();
        assert!(catalog.len() >= 45, "catalog has {}", catalog.len());

        let providers = all_providers();
        assert_eq!(providers.len(), 10);

        for p in &providers {
            let models = models_by_provider(p);
            assert!(
                !models.is_empty(),
                "Provider {p} must have at least one registered model"
            );
        }

        let argon = catalog_entry("gemini-4-argon").expect("gemini-4-argon must be present");
        assert_eq!(argon.provider, "google");
        assert!(argon.supports_thinking);
        assert_eq!(argon.context_window, 1_048_576);

        let chonk = catalog_entry("mistral-large-4").expect("mistral-large-4 must be present");
        assert_eq!(chonk.provider, "mistral");
        assert_eq!(chonk.context_window, 1_000_000);

        let sol = catalog_entry("gpt-6.1-sol").expect("gpt-6.1-sol must be present");
        assert_eq!(sol.provider, "openai");
        assert_eq!(sol.context_window, 1_048_576);

        let fable = catalog_entry("claude-fable-5-1").expect("claude-fable-5-1 must be present");
        assert_eq!(fable.provider, "anthropic");
        assert_eq!(fable.max_output_tokens, 128_000);

        let opus55 = catalog_entry("claude-opus-5-5").expect("claude-opus-5-5 must be present");
        assert_eq!(opus55.provider, "anthropic");
        assert_eq!(opus55.max_output_tokens, 128_000);
        assert!(opus55.supports_thinking);

        let flash = catalog_entry("deepseek-flash").expect("deepseek-flash must be present");
        assert_eq!(flash.provider, "deepseek");
        assert_eq!(flash.context_window, 1_000_000);
    }

    #[test]
    fn endpoint_aliases_and_deprecated_models() {
        // Test alias resolution
        let opus_alias = catalog_entry("claude-5-5-opus-20261001").expect("alias must resolve");
        assert_eq!(opus_alias.id, "claude-opus-5-5");

        let gpt4o_snap = catalog_entry("gpt-4o-2024-11-20").expect("snapshot must resolve");
        assert_eq!(gpt4o_snap.id, "gpt-4o");

        let gemini_alias = catalog_entry("models/gemini-4-argon").expect("gemini URL path alias must resolve");
        assert_eq!(gemini_alias.id, "gemini-4-argon");

        // Test deprecated models mapping
        let c3opus = catalog_entry("claude-3-opus-20240229").expect("claude-3-opus must resolve");
        assert!(c3opus.deprecated);
        assert_eq!(c3opus.replacement_id, Some("claude-opus-5-5"));
        assert!(c3opus.deprecation_notice.is_some());

        let gpt35 = catalog_entry("gpt-3.5-turbo").expect("gpt-3.5-turbo must resolve");
        assert!(gpt35.deprecated);
        assert_eq!(gpt35.replacement_id, Some("gpt-4o-mini"));

        let g15pro = catalog_entry("gemini-1.5-pro").expect("gemini-1.5-pro must resolve");
        assert!(g15pro.deprecated);
        assert_eq!(g15pro.replacement_id, Some("gemini-4-argon"));

        assert!(!deprecated_models().is_empty());
        assert!(!active_models().is_empty());
    }
}
