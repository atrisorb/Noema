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



    /// Gemini 2.5 Flash pricing.

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



    /// Gemini 2.5 Pro pricing.

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



    /// Look up the standard rate card for a model identifier.

    pub fn for_model(model: &str) -> Option<Self> {

        let m = model.to_lowercase();

        if m.contains("deepseek-r1") || m.contains("reasoner") {

            Some(Self::deepseek_r1())

        } else if m.contains("deepseek") {

            Some(Self::deepseek_v3())

        } else if m.contains("mistral-large") {

            Some(Self::mistral_large())

        } else if m.contains("mistral") || m.contains("codestral") {

            Some(Self::mistral_small())

        } else if m.contains("gemini-2.5-pro") || m.contains("gemini-pro") {

            Some(Self::gemini_2_5_pro())

        } else if m.contains("gemini") {

            Some(Self::gemini_2_5_flash())

        } else {

            None

        }

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

    fn hand_computed_mixed_usage() {

        let card = RateCard {

            input: 3.0,

            output: 15.0,

            reasoning: 60.0,

            cache_read: 0.3,

            cache_write: 3.75,

            per_call: 0.002,

            ..RateCard::default()

        };

        let usage = Usage {

            input: 2_000_000,

            output: 1_000_000,

            reasoning: 200_000,

            cache_read: 1_000_000,

            cache_write: 500_000,

        };

        // 6 + 15 + 12 + 0.3 + 1.875 + 0.002

        close(card.cost_at(&usage, at("2026-10-30T10:00:00Z"), 0.0), 35.177);

    }



    #[test]

    fn peak_window_uses_iana_timezone_and_dst() {

        let card = RateCard {

            input: 1.0,

            windows: vec![RateWindow {

                timezone: "Europe/Rome".into(),

                weekday_mask: 0b0011111,

                start_minute: 9 * 60,

                end_minute: 17 * 60,

                multiplier: 2.0,

            }],

            ..RateCard::default()

        };

        let usage = Usage { input: 1_000_000, ..Usage::default() };

        // Friday 2026-10-30, CET (+1): 10:00Z -> 11:00 local, inside.

        close(card.cost_at(&usage, at("2026-10-30T10:00:00Z"), 0.0), 2.0);

        // Wednesday 2026-07-01, CEST (+2): 07:30Z -> 09:30 local, inside.

        close(card.cost_at(&usage, at("2026-07-01T07:30:00Z"), 0.0), 2.0);

        // Same Friday, 22:00Z -> 23:00 local, outside.

        close(card.cost_at(&usage, at("2026-10-30T22:00:00Z"), 0.0), 1.0);

        // Saturday 10:00Z -> 11:00 local, weekday mask excludes it.

        close(card.cost_at(&usage, at("2026-10-31T10:00:00Z"), 0.0), 1.0);

    }



    #[test]

    fn overnight_window_wraps_midnight() {

        let card = RateCard {

            input: 1.0,

            windows: vec![RateWindow {

                timezone: "Europe/Rome".into(),

                weekday_mask: 0b1111111,

                start_minute: 23 * 60,

                end_minute: 6 * 60,

                multiplier: 0.5,

            }],

            ..RateCard::default()

        };

        let usage = Usage { input: 1_000_000, ..Usage::default() };

        close(card.cost_at(&usage, at("2026-10-30T23:30:00Z"), 0.0), 0.5);

        close(card.cost_at(&usage, at("2026-10-30T03:00:00Z"), 0.0), 0.5);

        close(card.cost_at(&usage, at("2026-10-30T12:00:00Z"), 0.0), 1.0);

    }



    #[test]

    fn local_hardware_amortisation() {

        let card = RateCard {

            local: Some(LocalCost {

                hardware_usd: 2000.0,

                lifespan_hours: 20_000.0,

                watts: 300.0,

                kwh_price: 0.25,

            }),

            ..RateCard::default()

        };

        // 0.1 USD/h hardware + 0.3 kW * 0.25 = 0.075 USD/h energy.

        close(card.local.unwrap().per_hour(), 0.175);

        close(

            card.cost_at(&Usage::default(), at("2026-10-30T10:00:00Z"), 10.0),

            1.75,

        );

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

}

