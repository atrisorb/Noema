//! FinOps math: per-token prices, peak/off-peak windows in IANA timezones and
//! local hardware amortisation. All inputs are USD.

use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::{Standard, Usage};

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

    // --- Google Gemini ---

    /// Gemini 4 Argon: unified 1M input/output window flagship (released Oct 2026).
    /// Input: $2.00/M, Output/Thinking: $10.00/M, Implicit Cache Read: $0.10/M (95% discount).
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

    /// Gemini 3.8 Flash: long-horizon agentic and coding model.
    /// Input: $0.75/M, Output/Thinking: $3.75/M, Cache Read: $0.075/M.
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

    /// Gemini 3.5 Flash-Lite: high-throughput agentic model.
    /// Input: $0.30/M, Output: $2.50/M, Cache Read: $0.03/M.
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

    /// Gemini 3.1 Pro: deep complex reasoning and mathematics.
    /// Input: $2.00/M, Output/Thinking: $12.00/M, Cache Read: $0.20/M.
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

    /// Gemini 3.8 Live: sub-second audio/video conversational streaming.
    /// Input: $1.00/M, Output: $4.00/M, Cache Read: $0.10/M.
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

    /// Gemini 2.5 Flash (legacy).
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

    /// Gemini 2.5 Pro (legacy).
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

    // --- OpenAI ---

    /// GPT-6 Astra: frontier intelligence flagship (2026).
    /// Input: $10.00/M, Output/Thinking: $50.00/M, Cache Read: $1.00/M.
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

    /// GPT-5.6 Sol: frontier agentic and reasoning model.
    /// Input: $5.00/M, Output: $30.00/M, Cache Read: $0.50/M.
    pub fn gpt_5_6_sol() -> Self {
        Self {
            input: 5.00,
            output: 30.00,
            reasoning: 30.00,
            cache_read: 0.50,
            cache_write: 5.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// GPT-5.6 Terra: balanced frontier performance.
    /// Input: $2.50/M, Output: $15.00/M, Cache Read: $0.25/M.
    pub fn gpt_5_6_terra() -> Self {
        Self {
            input: 2.50,
            output: 15.00,
            reasoning: 15.00,
            cache_read: 0.25,
            cache_write: 2.50,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// GPT-5 mini: fast agentic model.
    /// Input: $0.20/M, Output: $0.80/M, Cache Read: $0.02/M.
    pub fn gpt_5_mini() -> Self {
        Self {
            input: 0.20,
            output: 0.80,
            reasoning: 0.80,
            cache_read: 0.02,
            cache_write: 0.20,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// OpenAI o3: flagship reasoning model.
    /// Input: $2.00/M, Output/Thinking: $8.00/M, Cache Read: $0.20/M.
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

    /// OpenAI o3-pro: ultra-deep reasoning and verified proofs.
    /// Input: $20.00/M, Output/Thinking: $80.00/M, Cache Read: $2.00/M.
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

    /// OpenAI o3-mini: STEM and code-focused reasoning model.
    /// Input: $1.10/M, Output/Thinking: $4.40/M, Cache Read: $0.55/M.
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

    /// OpenAI o4-mini: next-generation fast reasoning.
    /// Input: $1.10/M, Output/Thinking: $4.40/M, Cache Read: $0.11/M.
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

    /// OpenAI o1: first-generation reasoning model.
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

    /// OpenAI GPT-4o.
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

    /// OpenAI GPT-4o mini.
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

    // --- Anthropic ---

    /// Claude Sonnet 5.5: flagship 5.5 generation model.
    /// Input: $2.00/M, Output/Thinking: $10.00/M, Cache Read: $0.10/M, Cache Write: $2.50/M.
    pub fn claude_sonnet_5_5() -> Self {
        Self {
            input: 2.00,
            output: 10.00,
            reasoning: 10.00,
            cache_read: 0.10,
            cache_write: 2.50,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Claude Opus 5.5: deepest frontier reasoning.
    /// Input: $4.00/M, Output/Thinking: $20.00/M, Cache Read: $0.20/M, Cache Write: $5.00/M.
    pub fn claude_opus_5_5() -> Self {
        Self {
            input: 4.00,
            output: 20.00,
            reasoning: 20.00,
            cache_read: 0.20,
            cache_write: 5.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Claude Haiku 5.5: high-speed 5.5 tier model.
    /// Input: $0.10/M, Output: $0.50/M, Cache Read: $0.01/M, Cache Write: $0.125/M.
    pub fn claude_haiku_5_5() -> Self {
        Self {
            input: 0.10,
            output: 0.50,
            reasoning: 0.50,
            cache_read: 0.01,
            cache_write: 0.125,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Claude 3.7 Sonnet: hybrid reasoning and coding model.
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

    /// Claude 3.5 Sonnet.
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

    /// Claude 3.5 Haiku.
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

    /// Claude 3 Opus.
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

    // --- Mistral AI ---

    /// Mistral Large 4 ("Le Chonk"): 1.05T MoE architecture, 1M context.
    /// Input: $1.36/M, Output/Thinking: $4.18/M, Cache Read: $0.14/M (90% discount).
    pub fn mistral_large_4() -> Self {
        Self {
            input: 1.36,
            output: 4.18,
            reasoning: 4.18,
            cache_read: 0.14,
            cache_write: 1.36,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Mistral Large (2411).
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

    /// Mistral Small (2501).
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

    /// Codestral (2501).
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

    // --- DeepSeek ---

    /// DeepSeek-V4.1 Flash: dynamic peak/off-peak pricing with ultra-low cache read.
    /// Off-peak Input: $0.15/M, Output: $0.60/M, Cache Hit: $0.003/M.
    /// Peak window configured with 2.0x multiplier.
    pub fn deepseek_v4_flash() -> Self {
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
                    weekday_mask: 0b0011111, // Mon-Fri
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

    /// DeepSeek-R1 / Reasoner.
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

    /// DeepSeek-V3 / Chat.
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

    // --- Zhipu AI (GLM) ---

    /// GLM-5.3: 1M context MoE flagship model (Zhipu AI).
    /// Input: $1.40/M, Output/Thinking: $4.40/M, Cache Read: $0.26/M.
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

    /// GLM-5.3 Flash: high-throughput 1M context model.
    /// Input: $0.15/M, Output: $0.50/M, Cache Read: $0.03/M.
    pub fn glm_5_3_flash() -> Self {
        Self {
            input: 0.15,
            output: 0.50,
            reasoning: 0.50,
            cache_read: 0.03,
            cache_write: 0.15,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// GLM-5.3 FlashX: extreme throughput coding model.
    /// Input: $0.37/M, Output: $1.25/M, Cache Read: $0.075/M.
    pub fn glm_5_3_flashx() -> Self {
        Self {
            input: 0.37,
            output: 1.25,
            reasoning: 1.25,
            cache_read: 0.075,
            cache_write: 0.37,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// GLM-4.7 Flash: free-tier model.
    pub fn glm_4_7_flash() -> Self {
        Self::default()
    }

    // --- Moonshot AI (Kimi) ---

    /// Kimi K3.1 Pro: 1M context flagship model with superior analysis.
    /// Input: $2.00/M, Output/Thinking: $10.00/M, Cache Read: $0.20/M (90% discount).
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

    /// Kimi K3.1 Pro Highspeed: accelerated agent inference.
    /// Input: $4.00/M, Output: $20.00/M, Cache Read: $0.20/M.
    pub fn kimi_k3_1_pro_highspeed() -> Self {
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

    /// Kimi K3: original 1M context model.
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

    // --- Meta Llama & Qwen (Hosted / Cloud) ---

    /// Llama 3.3 70B Instruct (hosted rate).
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

    /// Llama 3.1 405B Instruct (hosted rate).
    pub fn llama_3_1_405b() -> Self {
        Self {
            input: 3.00,
            output: 3.00,
            reasoning: 3.00,
            cache_read: 0.75,
            cache_write: 3.00,
            per_call: 0.0,
            windows: Vec::new(),
            local: None,
        }
    }

    /// Qwen 2.5 Coder 32B Instruct (hosted rate).
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

    /// Zero token-rate card for local models (cost comes from hardware amortisation / kWh).
    pub fn local_zero() -> Self {
        Self::default()
    }

    /// Look up the standard rate card for any model identifier or alias.
    pub fn for_model(model: &str) -> Option<Self> {
        let m = model.to_lowercase();
        // 1. Google Gemini
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
        } else if m.contains("gemini-pro") {
            Some(Self::gemini_4_argon())
        } else if m.contains("gemini") {
            Some(Self::gemini_3_8_flash())
        // 2. OpenAI
        } else if m.contains("gpt-6") || m.contains("astra") {
            Some(Self::gpt_6_astra())
        } else if m.contains("gpt-5.6-sol") || m.contains("sol") {
            Some(Self::gpt_5_6_sol())
        } else if m.contains("gpt-5.6-terra") || m.contains("terra") {
            Some(Self::gpt_5_6_terra())
        } else if m.contains("gpt-5-mini") || m.contains("gpt-5-nano") {
            Some(Self::gpt_5_mini())
        } else if m.contains("o3-pro") {
            Some(Self::openai_o3_pro())
        } else if m.contains("o4-mini") {
            Some(Self::openai_o4_mini())
        } else if m.contains("o3-mini") {
            Some(Self::openai_o3_mini())
        } else if m.contains("o3") {
            Some(Self::openai_o3())
        } else if m.contains("o1") {
            Some(Self::openai_o1())
        } else if m.contains("gpt-4o-mini") {
            Some(Self::openai_gpt_4o_mini())
        } else if m.contains("gpt-4o") || m.contains("gpt-4") || m.contains("chatgpt") {
            Some(Self::openai_gpt_4o())
        // 3. Anthropic
        } else if m.contains("opus-5-5") || m.contains("claude-opus-5") {
            Some(Self::claude_opus_5_5())
        } else if m.contains("haiku-5-5") || m.contains("claude-haiku-5") {
            Some(Self::claude_haiku_5_5())
        } else if m.contains("sonnet-5-5") || m.contains("claude-sonnet-5") || m.contains("claude-5-5") {
            Some(Self::claude_sonnet_5_5())
        } else if m.contains("claude-3-7") || m.contains("claude-3.7") {
            Some(Self::claude_3_7_sonnet())
        } else if m.contains("claude-3-5-sonnet") || m.contains("claude-3.5-sonnet") || m.contains("claude-sonnet") {
            Some(Self::claude_3_5_sonnet())
        } else if m.contains("claude-3-5-haiku") || m.contains("claude-3.5-haiku") || m.contains("claude-haiku") {
            Some(Self::claude_3_5_haiku())
        } else if m.contains("claude-3-opus") || m.contains("claude-opus") {
            Some(Self::claude_3_opus())
        // 4. Mistral AI
        } else if m.contains("mistral-large-4") || m.contains("chonk") {
            Some(Self::mistral_large_4())
        } else if m.contains("mistral-large") {
            Some(Self::mistral_large())
        } else if m.contains("codestral") {
            Some(Self::mistral_codestral())
        } else if m.contains("pixtral") {
            Some(Self::mistral_large())
        } else if m.contains("mistral") {
            Some(Self::mistral_small())
        // 5. DeepSeek
        } else if m.contains("deepseek-v4") || m.contains("deepseek-flash") {
            Some(Self::deepseek_v4_flash())
        } else if m.contains("deepseek-r1") || m.contains("reasoner") {
            Some(Self::deepseek_r1())
        } else if m.contains("deepseek") {
            Some(Self::deepseek_v3())
        // 6. Zhipu AI (GLM)
        } else if m.contains("glm-5.3-flashx") {
            Some(Self::glm_5_3_flashx())
        } else if m.contains("glm-5.3-flash") {
            Some(Self::glm_5_3_flash())
        } else if m.contains("glm-5.3") || m.contains("glm-5") {
            Some(Self::glm_5_3())
        } else if m.contains("glm-4.7-flash") || m.contains("glm-4") {
            Some(Self::glm_4_7_flash())
        } else if m.contains("glm") {
            Some(Self::glm_5_3_flash())
        // 7. Moonshot AI (Kimi)
        } else if m.contains("kimi-k3.1-pro-highspeed") {
            Some(Self::kimi_k3_1_pro_highspeed())
        } else if m.contains("kimi-k3.1") {
            Some(Self::kimi_k3_1_pro())
        } else if m.contains("kimi-k3") {
            Some(Self::kimi_k3())
        } else if m.contains("kimi") {
            Some(Self::kimi_k3_1_pro())
        // 8. Meta Llama & Qwen
        } else if m.contains("llama-3.1-405b") {
            Some(Self::llama_3_1_405b())
        } else if m.contains("llama-3.3") || m.contains("llama") {
            Some(Self::llama_3_3_70b())
        } else if m.contains("qwen") {
            Some(Self::qwen_2_5_coder_32b())
        // 9. Local zero-token fallback
        } else if m.contains("ollama") || m.contains("vllm") || m.contains("local") {
            Some(Self::local_zero())
        } else {
            None
        }
    }
}

/// Comprehensive descriptor for a model in the 2026 inference catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

/// Exhaustive built-in model catalog for all supported cloud and local providers (October 2026).
pub fn default_catalog() -> Vec<ModelCatalogEntry> {
    vec![
        // ----------------------------------------------------
        // 1. Google Gemini
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "gemini-4-argon",
            provider: "google",
            name: "Gemini 4 Argon",
            standard: Standard::Gemini,
            capability: "reasoning,agentic,code,math,science,multimodal",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_4_argon(),
        },
        ModelCatalogEntry {
            id: "gemini-3.8-flash",
            provider: "google",
            name: "Gemini 3.8 Flash",
            standard: Standard::Gemini,
            capability: "general,reasoning,agentic,code",
            context_window: 1_048_576,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_3_8_flash(),
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
        },
        ModelCatalogEntry {
            id: "gemini-3.1-pro-preview",
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
        },
        ModelCatalogEntry {
            id: "gemini-2.5-flash",
            provider: "google",
            name: "Gemini 2.5 Flash (Legacy)",
            standard: Standard::Gemini,
            capability: "general,fast",
            context_window: 1_048_576,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gemini_2_5_flash(),
        },

        // ----------------------------------------------------
        // 2. OpenAI
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "gpt-6-astra",
            provider: "openai",
            name: "GPT-6 Astra",
            standard: Standard::OpenAiChat,
            capability: "frontier_intelligence,agentic,reasoning,multimodal",
            context_window: 500_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: true,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_6_astra(),
        },
        ModelCatalogEntry {
            id: "gpt-5.6-sol",
            provider: "openai",
            name: "GPT-5.6 Sol",
            standard: Standard::OpenAiChat,
            capability: "frontier,code,agentic,deep_understanding",
            context_window: 256_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_5_6_sol(),
        },
        ModelCatalogEntry {
            id: "gpt-5.6-terra",
            provider: "openai",
            name: "GPT-5.6 Terra",
            standard: Standard::OpenAiChat,
            capability: "general,balanced,code,multimodal",
            context_window: 256_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(8_192),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_5_6_terra(),
        },
        ModelCatalogEntry {
            id: "gpt-5-mini",
            provider: "openai",
            name: "GPT-5 mini",
            standard: Standard::OpenAiChat,
            capability: "fast,agentic,code",
            context_window: 256_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::gpt_5_mini(),
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
        },

        // ----------------------------------------------------
        // 3. Anthropic
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "claude-sonnet-5-5",
            provider: "anthropic",
            name: "Claude Sonnet 5.5",
            standard: Standard::AnthropicMessages,
            capability: "frontier,engineering,agentic,autonomous",
            context_window: 200_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_sonnet_5_5(),
        },
        ModelCatalogEntry {
            id: "claude-opus-5-5",
            provider: "anthropic",
            name: "Claude Opus 5.5",
            standard: Standard::AnthropicMessages,
            capability: "deepest_reasoning,synthesis,complex_system_design",
            context_window: 200_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_opus_5_5(),
        },
        ModelCatalogEntry {
            id: "claude-haiku-5-5",
            provider: "anthropic",
            name: "Claude Haiku 5.5",
            standard: Standard::AnthropicMessages,
            capability: "ultra_fast,agentic_subtasks,high_throughput",
            context_window: 200_000,
            max_output_tokens: 16_384,
            supports_thinking: true,
            default_thinking_budget: Some(4_096),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_haiku_5_5(),
        },
        ModelCatalogEntry {
            id: "claude-3-7-sonnet-20250219",
            provider: "anthropic",
            name: "Claude 3.7 Sonnet",
            standard: Standard::AnthropicMessages,
            capability: "hybrid_reasoning,agentic,code",
            context_window: 200_000,
            max_output_tokens: 64_000,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_7_sonnet(),
        },
        ModelCatalogEntry {
            id: "claude-3-5-sonnet-20241022",
            provider: "anthropic",
            name: "Claude 3.5 Sonnet",
            standard: Standard::AnthropicMessages,
            capability: "code,general,reasoning",
            context_window: 200_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_5_sonnet(),
        },
        ModelCatalogEntry {
            id: "claude-3-5-haiku-20241022",
            provider: "anthropic",
            name: "Claude 3.5 Haiku",
            standard: Standard::AnthropicMessages,
            capability: "fast,code,general",
            context_window: 200_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::claude_3_5_haiku(),
        },

        // ----------------------------------------------------
        // 4. Mistral AI
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "mistral-large-4",
            provider: "mistral",
            name: "Mistral Large 4 (Le Chonk)",
            standard: Standard::Mistral,
            capability: "1.05T_moe,multimodal,reasoning,1m_context",
            context_window: 1_000_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_large_4(),
        },
        ModelCatalogEntry {
            id: "mistral-large-2411",
            provider: "mistral",
            name: "Mistral Large 3",
            standard: Standard::Mistral,
            capability: "general,reasoning,multilingual",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_large(),
        },
        ModelCatalogEntry {
            id: "codestral-2501",
            provider: "mistral",
            name: "Codestral",
            standard: Standard::Mistral,
            capability: "code,fill_in_the_middle,repo_analysis",
            context_window: 256_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_codestral(),
        },
        ModelCatalogEntry {
            id: "mistral-small-2501",
            provider: "mistral",
            name: "Mistral Small",
            standard: Standard::Mistral,
            capability: "lightweight,fast,multilingual",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::mistral_small(),
        },

        // ----------------------------------------------------
        // 5. DeepSeek
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "deepseek-flash",
            provider: "deepseek",
            name: "DeepSeek-V4.1 Flash",
            standard: Standard::DeepSeek,
            capability: "fast,code,ultra_low_cache_price,peak_aware",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_v4_flash(),
        },
        ModelCatalogEntry {
            id: "deepseek-reasoner",
            provider: "deepseek",
            name: "DeepSeek-R1",
            standard: Standard::DeepSeek,
            capability: "open_weights_reasoning,math,formal_proofs",
            context_window: 64_000,
            max_output_tokens: 16_384,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_r1(),
        },
        ModelCatalogEntry {
            id: "deepseek-chat",
            provider: "deepseek",
            name: "DeepSeek-V3",
            standard: Standard::DeepSeek,
            capability: "671b_moe,general,code",
            context_window: 64_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::deepseek_v3(),
        },

        // ----------------------------------------------------
        // 6. Zhipu AI (GLM)
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "glm-5.3",
            provider: "zhipu",
            name: "GLM-5.3",
            standard: Standard::Zhipu,
            capability: "1m_context_moe,complex_reasoning,code",
            context_window: 1_000_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::glm_5_3(),
        },
        ModelCatalogEntry {
            id: "glm-5.3-flash",
            provider: "zhipu",
            name: "GLM-5.3 Flash",
            standard: Standard::Zhipu,
            capability: "1m_context,high_throughput,agentic",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(8_192),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::glm_5_3_flash(),
        },
        ModelCatalogEntry {
            id: "glm-5.3-flashx",
            provider: "zhipu",
            name: "GLM-5.3 FlashX",
            standard: Standard::Zhipu,
            capability: "extreme_throughput,agent_loops,code",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(8_192),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::glm_5_3_flashx(),
        },
        ModelCatalogEntry {
            id: "glm-4.7-flash",
            provider: "zhipu",
            name: "GLM-4.7 Flash (Free)",
            standard: Standard::Zhipu,
            capability: "free_tier,lightweight,general",
            context_window: 128_000,
            max_output_tokens: 4_096,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::glm_4_7_flash(),
        },

        // ----------------------------------------------------
        // 7. Moonshot AI (Kimi)
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "kimi-k3.1-pro",
            provider: "moonshot",
            name: "Kimi K3.1 Pro",
            standard: Standard::Moonshot,
            capability: "1m_context,deep_analysis,agentic_execution",
            context_window: 1_000_000,
            max_output_tokens: 65_536,
            supports_thinking: true,
            default_thinking_budget: Some(32_768),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::kimi_k3_1_pro(),
        },
        ModelCatalogEntry {
            id: "kimi-k3.1-pro-highspeed",
            provider: "moonshot",
            name: "Kimi K3.1 Pro Highspeed",
            standard: Standard::Moonshot,
            capability: "realtime_agents,1m_context,accelerated",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::kimi_k3_1_pro_highspeed(),
        },
        ModelCatalogEntry {
            id: "kimi-k3",
            provider: "moonshot",
            name: "Kimi K3",
            standard: Standard::Moonshot,
            capability: "1m_context_long_document_analysis",
            context_window: 1_000_000,
            max_output_tokens: 32_768,
            supports_thinking: true,
            default_thinking_budget: Some(16_384),
            supports_vision: true,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::kimi_k3(),
        },

        // ----------------------------------------------------
        // 8. Meta Llama (Hosted / Cloud API)
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "meta-llama/llama-3.3-70b-instruct",
            provider: "meta",
            name: "Llama 3.3 70B Instruct",
            standard: Standard::OpenAiChat,
            capability: "general,open_weights,code",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_3_70b(),
        },
        ModelCatalogEntry {
            id: "meta-llama/llama-3.1-405b-instruct",
            provider: "meta",
            name: "Llama 3.1 405B Instruct",
            standard: Standard::OpenAiChat,
            capability: "frontier_open_weights,heavy_reasoning",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::llama_3_1_405b(),
        },

        // ----------------------------------------------------
        // 9. Alibaba Qwen
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "qwen/qwen-2.5-coder-32b-instruct",
            provider: "qwen",
            name: "Qwen 2.5 Coder 32B",
            standard: Standard::OpenAiChat,
            capability: "code_specialist,agentic,fill_in_the_middle",
            context_window: 128_000,
            max_output_tokens: 16_384,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: true,
            rate_card: RateCard::qwen_2_5_coder_32b(),
        },

        // ----------------------------------------------------
        // 10. Local Self-Hosted Runtimes (Ollama, llama.cpp, vLLM)
        // ----------------------------------------------------
        ModelCatalogEntry {
            id: "local/llama-3.3-70b",
            provider: "local",
            name: "Local Llama 3.3 70B",
            standard: Standard::Ollama,
            capability: "local,private,zero_token_cost",
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_thinking: false,
            default_thinking_budget: None,
            supports_vision: false,
            supports_audio: false,
            supports_tools: true,
            supports_caching: false,
            rate_card: RateCard::local_zero(),
        },
        ModelCatalogEntry {
            id: "local/qwen2.5-coder:32b",
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
        },
    ]
}

/// Find a catalog entry by ID or model alias.
pub fn catalog_entry(model_id: &str) -> Option<ModelCatalogEntry> {
    let catalog = default_catalog();
    let m = model_id.to_lowercase();
    catalog
        .into_iter()
        .find(|entry| entry.id.to_lowercase() == m || m.contains(entry.id.to_lowercase().as_str()))
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
    fn deepseek_v4_flash_peak_window_and_rates() {
        let card = RateCard::deepseek_v4_flash();
        assert_eq!(card.input, 0.15);
        assert_eq!(card.output, 0.60);
        assert_eq!(card.cache_read, 0.003);
        // Off-peak: UTC 12:00
        assert!(!card.peak(at("2026-10-28T12:00:00Z")));
        // Peak: UTC 02:00 Mon-Fri
        assert!(card.peak(at("2026-10-28T02:00:00Z")));
    }

    #[test]
    fn gemini_4_argon_and_gemini_3_models() {
        let argon = RateCard::gemini_4_argon();
        assert_eq!(argon.input, 2.00);
        assert_eq!(argon.output, 10.00);
        assert_eq!(argon.cache_read, 0.10);
        assert_eq!(RateCard::for_model("gemini-4-argon"), Some(argon));

        let g38 = RateCard::gemini_3_8_flash();
        assert_eq!(RateCard::for_model("gemini-3.8-flash"), Some(g38));
    }

    #[test]
    fn mistral_large_4_and_glm_kimi_models() {
        let m4 = RateCard::mistral_large_4();
        assert_eq!(m4.input, 1.36);
        assert_eq!(m4.output, 4.18);
        assert_eq!(RateCard::for_model("mistral-large-4"), Some(m4));

        let glm = RateCard::glm_5_3();
        assert_eq!(glm.input, 1.40);
        assert_eq!(glm.output, 4.40);
        assert_eq!(RateCard::for_model("glm-5.3"), Some(glm));

        let kimi = RateCard::kimi_k3_1_pro();
        assert_eq!(kimi.input, 2.00);
        assert_eq!(kimi.output, 10.00);
        assert_eq!(RateCard::for_model("kimi-k3.1-pro"), Some(kimi));
    }

    #[test]
    fn claude_5_5_and_openai_gpt6_o3() {
        let sonnet55 = RateCard::claude_sonnet_5_5();
        assert_eq!(sonnet55.input, 2.00);
        assert_eq!(sonnet55.output, 10.00);
        assert_eq!(RateCard::for_model("claude-sonnet-5-5"), Some(sonnet55));

        let gpt6 = RateCard::gpt_6_astra();
        assert_eq!(gpt6.input, 10.00);
        assert_eq!(gpt6.output, 50.00);
        assert_eq!(RateCard::for_model("gpt-6-astra"), Some(gpt6));

        let o3 = RateCard::openai_o3();
        assert_eq!(o3.input, 2.00);
        assert_eq!(o3.output, 8.00);
        assert_eq!(RateCard::for_model("o3"), Some(o3));
    }

    #[test]
    fn catalog_completeness_and_filters() {
        let catalog = default_catalog();
        assert!(catalog.len() >= 30, "catalog has {}", catalog.len());

        let providers = all_providers();
        assert_eq!(providers.len(), 10);

        for p in &providers {
            let models = models_by_provider(p);
            assert!(
                !models.is_empty(),
                "Provider {p} must have at least one registered model"
            );
        }

        let argon_entry = catalog_entry("gemini-4-argon").expect("gemini-4-argon must be present");
        assert_eq!(argon_entry.provider, "google");
        assert!(argon_entry.supports_thinking);
        assert_eq!(argon_entry.context_window, 1_048_576);

        let chonk = catalog_entry("mistral-large-4").expect("mistral-large-4 must be present");
        assert_eq!(chonk.provider, "mistral");
        assert_eq!(chonk.context_window, 1_000_000);
    }
}
