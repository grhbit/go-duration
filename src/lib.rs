use std::{
    fmt::{self, Write},
    str::FromStr,
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{char, digit0, digit1},
    combinator::{all_consuming, cut, map_res, opt, value},
    error::{FromExternalError, ParseError},
    multi::fold_many1,
    sequence::{pair, preceded, tuple},
    Finish, IResult, Parser,
};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DurationParseError {
    #[error("time: invalid duration")]
    InvalidDuration,
    #[error("time: missing unit in duration")]
    MissingUnit,
    #[error("time: unknown unit {0} in duration")]
    UnknownUnit(String),
}

impl<I> ParseError<I> for DurationParseError {
    fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
        DurationParseError::InvalidDuration
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for DurationParseError {
    fn from_external_error(_input: I, _kind: nom::error::ErrorKind, _e: E) -> Self {
        DurationParseError::InvalidDuration
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GoDuration {
    pub nanos: i64,
}

impl fmt::Display for GoDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        let nanos = self.nanos;
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
    type Err = DurationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_go_duration(s)
            .finish()
            .map(|(_, nanos)| GoDuration { nanos })
    }
}

const NANOS_PER_MICROSECOND: u64 = 1_000;
const NANOS_PER_MILLISECOND: u64 = 1_000_000;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MINUTE: u64 = NANOS_PER_SECOND * 60;
const NANOS_PER_HOUR: u64 = NANOS_PER_MINUTE * 60;

fn decimal_parts(input: &str) -> IResult<&str, (u64, Option<&str>), DurationParseError> {
    alt((
        preceded(char::<&str, DurationParseError>('.'), digit1).map(|frac| (0, Some(frac))),
        pair(
            map_res(digit1, str::parse::<u64>),
            opt(preceded(char('.'), digit0)),
        ),
    ))(input)
}

fn sign(input: &str) -> IResult<&str, bool, DurationParseError> {
    opt(alt((value(false, char('-')), value(true, char('+')))))
        .map(|sign| sign.unwrap_or(true))
        .parse(input)
}

fn unit(input: &str) -> IResult<&str, u64, DurationParseError> {
    let (input, unit) = take_till(|c: char| c.is_ascii_digit() || c == '.')(input)?;
    if unit.is_empty() {
        return Err(nom::Err::Error(DurationParseError::MissingUnit));
    }
    let (_, unit) = all_consuming(alt((
        value(1u64, tag::<&str, &str, DurationParseError>("ns")),
        value(NANOS_PER_MICROSECOND, tag("\u{00B5}s")),
        value(NANOS_PER_MICROSECOND, tag("\u{03BC}s")),
        value(NANOS_PER_MICROSECOND, tag("us")),
        value(NANOS_PER_MILLISECOND, tag("ms")),
        value(NANOS_PER_SECOND, char('s')),
        value(NANOS_PER_MINUTE, char('m')),
        value(NANOS_PER_HOUR, char('h')),
    )))
    .parse(unit)
    .map_err(|_| nom::Err::Error(DurationParseError::UnknownUnit(unit.to_string())))?;
    Ok((input, unit))
}

pub fn parse_go_duration(input: &str) -> IResult<&str, i64, DurationParseError> {
    let (input, sign) = sign(input)?;
    let (input, nanos) = all_consuming(fold_many1(
        tuple((decimal_parts, cut(unit))).map(|((int, frac), scale)| {
            let nanos = frac
                .map(|frac: &str| {
                    let mut total = 0.0;
                    let mut scale = scale as f64;
                    for c in frac.chars() {
                        scale /= 10.0;
                        total += scale * c.to_digit(10).unwrap() as f64;
                    }
                    total as u64
                })
                .unwrap_or(0);
            int * scale + nanos
        }),
        || 0u64,
        u64::saturating_add,
    ))
    .parse(input)?;

    let nanos = if sign {
        if nanos <= i64::MAX as u64 {
            nanos as i64
        } else {
            return Err(nom::Err::Error(DurationParseError::InvalidDuration));
        }
    } else {
        0i64.checked_sub_unsigned(nanos)
            .ok_or(nom::Err::Error(DurationParseError::InvalidDuration))?
    };
    Ok((input, nanos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        let cases = [
            ("0s", 0),
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
        ];

        for (input, expected) in cases {
            let output = parse_go_duration(input);
            let (remaining, output) = output.expect(&format!("{input}"));
            assert!(remaining.is_empty(), "{input}");
            assert_eq!(expected, output, "{input}");
        }
    }

    #[test]
    fn parse_invalid() {
        let cases = [
            ("0", DurationParseError::MissingUnit),
            ("-1m-30s", DurationParseError::UnknownUnit("m-".to_string())),
            ("-2", DurationParseError::MissingUnit),
            ("0z", DurationParseError::UnknownUnit("z".to_string())),
            ("1m-30s", DurationParseError::UnknownUnit("m-".to_string())),
            ("1m+30s", DurationParseError::UnknownUnit("m+".to_string())),
            ("-1m+30s", DurationParseError::UnknownUnit("m+".to_string())),
            ("9223372036854775808ns", DurationParseError::InvalidDuration),
            ("-", DurationParseError::InvalidDuration),
            (" ", DurationParseError::InvalidDuration),
            ("1 m", DurationParseError::UnknownUnit(" m".to_string())),
            ("1h ", DurationParseError::UnknownUnit("h ".to_string())),
            (" 1s", DurationParseError::InvalidDuration),
        ];

        for (input, expected) in cases {
            let output = parse_go_duration(input).finish();
            assert!(output.is_err(), "{input} {output:?}");

            let output = output.unwrap_err();
            assert_eq!(output, expected, "{input}");
            println!("{}", output);
        }
    }

    #[test]
    fn format() {
        let cases = [
            (0, "0s"),
            (100, "100ns"),
            (1, "1ns"),
            (2, "2ns"),
            (2000, "2\u{00B5}s"),
            (-2000, "-2\u{00B5}s"),
            (200, "200ns"),
            (1, "1ns"),
            (NANOS_PER_HOUR as i64, "1h0m0s"),
            (NANOS_PER_MINUTE as i64, "1m0s"),
            (i64::MIN, "-2562047h47m16.854775808s"),
            (i64::MAX, "2562047h47m16.854775807s"),
        ];
        for (input, expected) in cases {
            let output = GoDuration { nanos: input }.to_string();
            assert_eq!(expected, output, "{input}");
        }
    }
}
