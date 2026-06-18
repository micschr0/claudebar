//! Parsed shape of the JSON Claude Code writes to the hook's stdin.
//!
//! Every numeric field is wrapped in [`Coerce`], a forgiving deserializer that
//! mirrors jq's `tonumber? // <default>`: a wrong-typed or unparseable value
//! degrades *that one field* to `None` instead of aborting the whole parse.
//! Combined with `#[serde(default)]` everywhere and a top-level
//! `unwrap_or_default()`, the render path can never fail to produce a line.

use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;
use std::marker::PhantomData;

/// Top-level input object. All fields optional and independently absent.
#[derive(Debug, Default, Deserialize)]
pub struct InputData {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub context_window: ContextWindow,
    #[serde(default)]
    pub rate_limits: RateLimits,
    #[serde(default)]
    pub model: Model,
    #[serde(default)]
    pub effort: Effort,
}

#[derive(Debug, Default, Deserialize)]
pub struct ContextWindow {
    #[serde(default)]
    pub total_input_tokens: Coerce<u64>,
    #[serde(default)]
    pub total_output_tokens: Coerce<u64>,
    /// Percentage of the context window used. **Can exceed 100.**
    #[serde(default)]
    pub used_percentage: Coerce<f64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RateLimits {
    #[serde(default)]
    pub five_hour: Option<Window>,
    #[serde(default)]
    pub seven_day: Option<Window>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Window {
    /// Percentage of the window consumed. **Can exceed 100** (over the limit).
    #[serde(default)]
    pub used_percentage: Coerce<f64>,
    /// Unix epoch **seconds** at which the window resets.
    #[serde(default)]
    pub resets_at: Coerce<i64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Effort {
    /// `low|medium|high|xhigh|max` — **absent** for models without an effort param.
    #[serde(default)]
    pub level: Option<String>,
}

impl InputData {
    /// Parse from a JSON string. On any failure (even invalid JSON), returns
    /// `InputData::default()` so the caller still renders a (possibly empty) line.
    pub fn parse(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

/// Forgiving numeric wrapper. Holds `Option<T>`; deserializes a number, a
/// numeric string, or null into `Some`/`None`, and turns any other type
/// (bool, array, object, unparseable string) into `None` rather than erroring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coerce<T>(pub Option<T>);

impl<T> Default for Coerce<T> {
    fn default() -> Self {
        Coerce(None)
    }
}

impl<T> Coerce<T> {
    /// The contained value, or `T::default()` when absent.
    pub fn or_default(self) -> T
    where
        T: Default,
    {
        self.0.unwrap_or_default()
    }

    pub fn get(self) -> Option<T> {
        self.0
    }
}

/// Conversion from the JSON scalar types into the target numeric type.
/// Implemented for the few types we actually parse (`u64`, `i64`, `f64`).
pub trait FromJsonNumber: Sized {
    fn from_u64(v: u64) -> Option<Self>;
    fn from_i64(v: i64) -> Option<Self>;
    fn from_f64(v: f64) -> Option<Self>;
    fn from_str_num(s: &str) -> Option<Self>;
}

impl FromJsonNumber for u64 {
    fn from_u64(v: u64) -> Option<Self> {
        Some(v)
    }
    fn from_i64(v: i64) -> Option<Self> {
        u64::try_from(v).ok()
    }
    fn from_f64(v: f64) -> Option<Self> {
        if v.is_finite() && v >= 0.0 {
            Some(v as u64)
        } else {
            None
        }
    }
    fn from_str_num(s: &str) -> Option<Self> {
        let s = s.trim();
        s.parse::<u64>()
            .ok()
            .or_else(|| s.parse::<f64>().ok().and_then(Self::from_f64))
    }
}

impl FromJsonNumber for i64 {
    fn from_u64(v: u64) -> Option<Self> {
        i64::try_from(v).ok()
    }
    fn from_i64(v: i64) -> Option<Self> {
        Some(v)
    }
    fn from_f64(v: f64) -> Option<Self> {
        if v.is_finite() {
            Some(v as i64)
        } else {
            None
        }
    }
    fn from_str_num(s: &str) -> Option<Self> {
        let s = s.trim();
        s.parse::<i64>()
            .ok()
            .or_else(|| s.parse::<f64>().ok().and_then(Self::from_f64))
    }
}

impl FromJsonNumber for f64 {
    fn from_u64(v: u64) -> Option<Self> {
        Some(v as f64)
    }
    fn from_i64(v: i64) -> Option<Self> {
        Some(v as f64)
    }
    fn from_f64(v: f64) -> Option<Self> {
        if v.is_finite() {
            Some(v)
        } else {
            None
        }
    }
    fn from_str_num(s: &str) -> Option<Self> {
        let v = s.trim().parse::<f64>().ok()?;
        if v.is_finite() {
            Some(v)
        } else {
            None
        }
    }
}

impl<'de, T> Deserialize<'de> for Coerce<T>
where
    T: FromJsonNumber,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CoerceVisitor<T>(PhantomData<T>);

        impl<'de, T: FromJsonNumber> Visitor<'de> for CoerceVisitor<T> {
            type Value = Coerce<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number, numeric string, or null")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Coerce(T::from_u64(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(Coerce(T::from_i64(v)))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(Coerce(T::from_f64(v)))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Coerce(T::from_str_num(v)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(Coerce(None))
            }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(Coerce(None))
            }
            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
                d.deserialize_any(self)
            }
            // Anything else (bool, seq, map) degrades to None rather than erroring.
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
                Ok(Coerce(None))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                while a.next_element::<de::IgnoredAny>()?.is_some() {}
                Ok(Coerce(None))
            }
            fn visit_map<A: de::MapAccess<'de>>(self, mut m: A) -> Result<Self::Value, A::Error> {
                while m.next_entry::<de::IgnoredAny, de::IgnoredAny>()?.is_some() {}
                Ok(Coerce(None))
            }
        }

        deserializer.deserialize_any(CoerceVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cw(json: &str) -> ContextWindow {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn number_parses() {
        let c = cw(r#"{"total_input_tokens": 35000, "used_percentage": 67.5}"#);
        assert_eq!(c.total_input_tokens.get(), Some(35000));
        assert_eq!(c.used_percentage.get(), Some(67.5));
    }

    #[test]
    fn numeric_string_coerces() {
        let c = cw(r#"{"total_input_tokens": "35000", "used_percentage": "67.5"}"#);
        assert_eq!(c.total_input_tokens.get(), Some(35000));
        assert_eq!(c.used_percentage.get(), Some(67.5));
    }

    #[test]
    fn wrong_type_degrades_to_none() {
        // bool, object, array, garbage string → None, but other fields survive.
        let c = cw(r#"{"total_input_tokens": true, "used_percentage": "abc"}"#);
        assert_eq!(c.total_input_tokens.get(), None);
        assert_eq!(c.used_percentage.get(), None);
    }

    #[test]
    fn null_is_none() {
        let c = cw(r#"{"total_input_tokens": null}"#);
        assert_eq!(c.total_input_tokens.get(), None);
    }

    #[test]
    fn missing_is_none() {
        let c = cw(r#"{}"#);
        assert_eq!(c.total_input_tokens.get(), None);
        assert_eq!(c.used_percentage.get(), None);
    }

    #[test]
    fn over_100_percent_preserved() {
        let c = cw(r#"{"used_percentage": 142.0}"#);
        assert_eq!(c.used_percentage.get(), Some(142.0));
    }

    #[test]
    fn whole_parse_never_fails() {
        // Garbage top-level → default, empty.
        let d = InputData::parse("not json at all");
        assert!(d.cwd.is_none());
        let d = InputData::parse("{}");
        assert!(d.cwd.is_none());
    }
}
