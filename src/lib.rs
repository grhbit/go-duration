use std::{
    fmt::{self, Write},
    str::FromStr,
};

use ::nom::{Parser, Finish};
#[cfg(feature = "serde")]
use ::serde::{Deserialize, Serialize};

pub mod nom;
#[cfg(feature = "serde")]
pub mod serde;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GoDurationParseError {
    #[error("time: invalid duration")]
    InvalidDuration,
    #[error("time: missing unit in duration")]
    MissingUnit,
    #[error("time: unknown unit \"{0}\" in duration")]
    UnknownUnit(String),
}

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoDuration(
    /// nanoseconds
    pub i64,
);

impl GoDuration {
    pub const ZERO: Self = GoDuration(0);
    pub const MIN: Self = GoDuration(i64::MIN);
    pub const MAX: Self = GoDuration(i64::MAX);

    #[inline]
    pub fn nanoseconds(&self) -> i64 {
        self.0
    }

    pub fn abs(&self) -> Self {
        Self(0i64.saturating_add_unsigned(self.0.unsigned_abs()))
    }
}

impl TryFrom<&str> for GoDuration {
    type Error = GoDurationParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<i64> for GoDuration {
    fn from(nanoseconds: i64) -> Self {
        Self(nanoseconds)
    }
}

impl fmt::Display for GoDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        let nanos = self.0;
        if nanos.is_negative() {
            f.write_char('-')?;
        }
        let mut nanos = nanos.unsigned_abs();
        if nanos >= NANOS_PER_HOUR {
            f.write_fmt(format_args!("{}h", nanos / NANOS_PER_HOUR))?;
            nanos %= NANOS_PER_HOUR;
            f.write_fmt(format_args!("{}m", nanos / NANOS_PER_MINUTE))?;
            nanos %= NANOS_PER_MINUTE;
            return f.write_fmt(format_args!("{}s", nanos as f64 / NANOS_PER_SECOND as f64));
        }
        if nanos >= NANOS_PER_MINUTE {
            f.write_fmt(format_args!("{}m", nanos / NANOS_PER_MINUTE))?;
            nanos %= NANOS_PER_MINUTE;
            return f.write_fmt(format_args!("{}s", nanos as f64 / NANOS_PER_SECOND as f64));
        }
        if nanos >= NANOS_PER_SECOND {
            return f.write_fmt(format_args!("{}s", nanos as f64 / NANOS_PER_SECOND as f64));
        }
        if nanos >= NANOS_PER_MILLISECOND {
            return f.write_fmt(format_args!(
                "{}ms",
                nanos as f64 / NANOS_PER_MILLISECOND as f64
            ));
        }
        if nanos >= NANOS_PER_MICROSECOND {
            return f.write_fmt(format_args!(
                "{}\u{00B5}s",
                nanos as f64 / NANOS_PER_MICROSECOND as f64
            ));
        }
        if nanos > 0 {
            return f.write_fmt(format_args!("{nanos}ns"));
        }
        f.write_str("0s")
    }
}

impl FromStr for GoDuration {
    type Err = GoDurationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_go_duration(s)
    }
}

pub(crate) const NANOS_PER_MICROSECOND: u64 = 1_000;
pub(crate) const NANOS_PER_MILLISECOND: u64 = 1_000_000;
pub(crate) const NANOS_PER_SECOND: u64 = 1_000_000_000;
pub(crate) const NANOS_PER_MINUTE: u64 = NANOS_PER_SECOND * 60;
pub(crate) const NANOS_PER_HOUR: u64 = NANOS_PER_MINUTE * 60;

pub fn parse_go_duration(input: &str) -> Result<GoDuration, GoDurationParseError> {
    nom::go_duration.parse(input).finish().map(|(_, dur)| dur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let cases = [
            ("0s", 0),
            ("+42ns", 42),
            (".1us", 100),
            (".1ns0.9ns", 0),
            ("1ns9ns", 10),
            ("1.ns", 1),
            ("2ns", 2),
            ("2us", 2000),
            ("-2us", -2000),
            ("0.2us", 200),
            ("0.0000000000003h", 1),
            ("1ns", 1),
            ("1us", 1_000),
            ("1\u{00B5}s", 1_000),
            ("1\u{03BC}s", 1_000),
            ("1ms", 1_000_000),
            ("1s", 1_000_000_000),
            ("1m", 60_000_000_000),
            ("1h", 3_600_000_000_000),
            ("9223372036854775807ns", 9_223_372_036_854_775_807),
            ("-9223372036854775808ns", -9_223_372_036_854_775_808),
        ];

        for (input, expected) in cases {
            let output = parse_go_duration(input);
            let output = output.expect(&format!("{input}"));
            assert_eq!(expected, output.0, "{input}");
        }
    }

    #[test]
    fn test_parse_invalid() {
        let cases = [
            ("", GoDurationParseError::InvalidDuration),
            ("0", GoDurationParseError::MissingUnit),
            (
                "-1m-30s",
                GoDurationParseError::UnknownUnit("m-".to_string()),
            ),
            ("-2", GoDurationParseError::MissingUnit),
            ("0z", GoDurationParseError::UnknownUnit("z".to_string())),
            (
                "1m-30s",
                GoDurationParseError::UnknownUnit("m-".to_string()),
            ),
            (
                "1m+30s",
                GoDurationParseError::UnknownUnit("m+".to_string()),
            ),
            (
                "-1m+30s",
                GoDurationParseError::UnknownUnit("m+".to_string()),
            ),
            (
                "9223372036854775808ns",
                GoDurationParseError::InvalidDuration,
            ),
            (
                "-9223372036854775809ns",
                GoDurationParseError::InvalidDuration,
            ),
            ("-", GoDurationParseError::InvalidDuration),
            ("+", GoDurationParseError::InvalidDuration),
            (" ", GoDurationParseError::InvalidDuration),
            ("-1 m", GoDurationParseError::UnknownUnit(" m".to_string())),
            ("17h ", GoDurationParseError::UnknownUnit("h ".to_string())),
            (" 42s", GoDurationParseError::InvalidDuration),
        ];

        for (input, expected) in cases {
            let output = parse_go_duration(input);
            assert!(output.is_err(), "{input} {output:?}");

            let output = output.unwrap_err();
            assert_eq!(output, expected, "{input}");
        }
    }

    #[test]
    fn test_format() {
        let cases = [
            (4000 * NANOS_PER_SECOND as i64, "1h6m40s"),
            (90 * NANOS_PER_MINUTE as i64, "1h30m0s"),
            (-1, "-1ns"),
            (0, "0s"),
            (1, "1ns"),
            (NANOS_PER_MICROSECOND as i64 - 1, "999ns"),
            (NANOS_PER_MICROSECOND as i64, "1\u{00B5}s"),
            (NANOS_PER_MICROSECOND as i64 + 1, "1.001\u{00B5}s"),
            (NANOS_PER_MILLISECOND as i64 - 1, "999.999\u{00B5}s"),
            (NANOS_PER_MILLISECOND as i64, "1ms"),
            (NANOS_PER_MILLISECOND as i64 + 1, "1.000001ms"),
            (NANOS_PER_SECOND as i64 - 1, "999.999999ms"),
            (NANOS_PER_SECOND as i64, "1s"),
            (NANOS_PER_SECOND as i64 + 1, "1.000000001s"),
            (NANOS_PER_MINUTE as i64 - 1, "59.999999999s"),
            (NANOS_PER_MINUTE as i64, "1m0s"),
            (NANOS_PER_MINUTE as i64 + 1, "1m0.000000001s"),
            (NANOS_PER_HOUR as i64 - 1, "59m59.999999999s"),
            (NANOS_PER_HOUR as i64, "1h0m0s"),
            (NANOS_PER_HOUR as i64 + 1, "1h0m0.000000001s"),
            (i64::MIN, "-2562047h47m16.854775808s"),
            (i64::MAX, "2562047h47m16.854775807s"),
        ];
        for (input, expected) in cases {
            let output = GoDuration(input).to_string();
            assert_eq!(expected, output, "{input}");
        }
    }

    #[test]
    fn test_try_from_trait() {
        let output = GoDuration::try_from("42ns");
        assert!(output.is_ok());
        assert_eq!(GoDuration(42), output.unwrap());
    }

    #[test]
    fn test_from_trait() {
        let output = GoDuration::from(-23);
        assert_eq!(GoDuration(-23), output);
    }

    #[test]
    fn test_duration_abs() {
        let cases = [
            (i64::MIN, i64::MAX),
            (i64::MAX, i64::MAX),
            (0, 0),
            (-42, 42),
        ];

        for (input, expected) in cases {
            let output = GoDuration(input).abs();
            assert_eq!(expected, output.nanoseconds(), "{input}");
        }
    }
}
