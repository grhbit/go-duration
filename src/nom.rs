use ::nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{char, digit0, digit1},
    combinator::{all_consuming, cut, map_res, opt, value},
    error::{FromExternalError, ParseError},
    multi::fold_many1,
    sequence::{pair, preceded, tuple},
    Err as NomErr, IResult, Parser,
};

use crate::{
    GoDuration, GoDurationParseError, NANOS_PER_HOUR, NANOS_PER_MICROSECOND, NANOS_PER_MILLISECOND,
    NANOS_PER_MINUTE, NANOS_PER_SECOND,
};

impl<I> ParseError<I> for GoDurationParseError {
    fn from_error_kind(_input: I, _kind: ::nom::error::ErrorKind) -> Self {
        GoDurationParseError::InvalidDuration
    }

    fn append(_input: I, _kind: ::nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for GoDurationParseError {
    fn from_external_error(_input: I, _kind: ::nom::error::ErrorKind, _e: E) -> Self {
        GoDurationParseError::InvalidDuration
    }
}

fn decimal_parts(input: &str) -> IResult<&str, (u64, Option<&str>), GoDurationParseError> {
    alt((
        preceded(char::<&str, GoDurationParseError>('.'), digit1).map(|frac| (0, Some(frac))),
        pair(
            map_res(digit1, str::parse::<u64>),
            opt(preceded(char('.'), digit0)),
        ),
    ))(input)
}

fn sign(input: &str) -> IResult<&str, bool, GoDurationParseError> {
    opt(alt((value(false, char('-')), value(true, char('+')))))
        .map(|sign| sign.unwrap_or(true))
        .parse(input)
}

fn unit(input: &str) -> IResult<&str, u64, GoDurationParseError> {
    let (input, unit) = take_till(|c: char| c.is_ascii_digit() || c == '.')(input)?;
    if unit.is_empty() {
        return Err(NomErr::Error(GoDurationParseError::MissingUnit));
    }
    let (_, unit) = all_consuming(alt((
        value(1u64, tag::<&str, &str, GoDurationParseError>("ns")),
        value(NANOS_PER_MICROSECOND, tag("\u{00B5}s")),
        value(NANOS_PER_MICROSECOND, tag("\u{03BC}s")),
        value(NANOS_PER_MICROSECOND, tag("us")),
        value(NANOS_PER_MILLISECOND, tag("ms")),
        value(NANOS_PER_SECOND, char('s')),
        value(NANOS_PER_MINUTE, char('m')),
        value(NANOS_PER_HOUR, char('h')),
    )))
    .parse(unit)
    .map_err(|_| NomErr::Error(GoDurationParseError::UnknownUnit(unit.to_string())))?;
    Ok((input, unit))
}

pub fn go_duration(input: &str) -> IResult<&str, GoDuration, GoDurationParseError> {
    let (input, sign) = sign(input)?;
    let (input, nanos) = all_consuming(fold_many1(
        tuple((decimal_parts, cut(unit))).map(|((int, frac), scale)| {
            let nanos = frac.map_or(0, |frac: &str| {
                let mut total = 0.0;
                let mut scale = scale as f64;
                for c in frac.chars() {
                    scale /= 10.0;
                    total += scale * c.to_digit(10).unwrap() as f64
                }
                total as u64
            });
            int.saturating_mul(scale).saturating_add(nanos)
        }),
        || 0u64,
        u64::saturating_add,
    ))
    .parse(input)?;

    let nanos = if sign {
        i64::try_from(nanos).map_err(|_| NomErr::Error(GoDurationParseError::InvalidDuration))?
    } else {
        0i64.checked_sub_unsigned(nanos)
            .ok_or(NomErr::Error(GoDurationParseError::InvalidDuration))?
    };
    Ok((input, GoDuration(nanos)))
}

#[cfg(test)]
mod tests {
    use nom::{combinator::map_res, Finish};

    use crate::GoDurationParseError;

    use super::*;

    #[test]
    fn test_from_external_error_trait() {
        let output = map_res(::nom::combinator::rest, str::parse::<u64>)
            .parse("invalid")
            .finish();
        assert!(output.is_err());
        let output: GoDurationParseError = output.unwrap_err();
        assert_eq!(output, GoDurationParseError::InvalidDuration);
    }
}
