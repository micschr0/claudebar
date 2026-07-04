//! Sample inputs for the live preview. Each is parsed once from a committed
//! fixture so the preview is byte-identical to what the hook would emit for that
//! input.

use crate::model::InputData;

/// A named sample input shown in the preview strip.
pub struct Sample {
    pub name: &'static str,
    pub input: InputData,
}

/// All preview samples, in cycle order. `p` advances through these.
pub fn all() -> Vec<Sample> {
    vec![
        Sample {
            name: "typical",
            input: InputData::parse(include_str!("../../fixtures/typical.json")),
        },
        Sample {
            name: "over-limit 5h",
            input: InputData::parse(include_str!("../../fixtures/over_limit_5h.json")),
        },
        Sample {
            name: "no git",
            input: InputData::parse(include_str!("../../fixtures/no_git.json")),
        },
        Sample {
            name: "no effort",
            input: InputData::parse(include_str!("../../fixtures/no_effort.json")),
        },
        Sample {
            name: "dev context",
            input: InputData::parse(include_str!("../../fixtures/dev_context.json")),
        },
        Sample {
            name: "weekly window",
            input: InputData::parse(include_str!("../../fixtures/weekly_at_50.json")),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_samples_including_weekly_window() {
        let samples = all();
        assert_eq!(samples.len(), 6);
        let weekly = samples
            .iter()
            .find(|s| s.name == "weekly window")
            .expect("weekly window sample must exist");
        assert!(weekly.input.rate_limits.seven_day.is_some());
    }
}
