use ::serde::{de, ser};

use crate::{GoDuration, GoDurationParseError};

impl ser::Serialize for GoDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.collect_str(&self)
    }
}

impl<'de> de::Deserialize<'de> for GoDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(GoDurationVisitor)
    }
}

pub struct GoDurationVisitor;

impl<'de> de::Visitor<'de> for GoDurationVisitor {
    type Value = GoDuration;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Go-lang style `time.Duration` string")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::Value::from(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value <= i64::MAX as u64 {
            self.visit_i64(value as i64)
        } else {
            Err(E::custom(GoDurationParseError::InvalidDuration))
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::try_from(value).map_err(E::custom)
    }
}

pub mod nanoseconds {
    use super::*;

    pub fn serialize<S>(value: &GoDuration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(value.nanoseconds())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<GoDuration, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_i64(GoDurationVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    use super::*;

    #[test]
    fn test_ser_de() {
        let dur = GoDuration(0);
        assert_tokens(&dur, &[Token::String("0s")]);
    }

    #[test]
    fn test_de_error() {
        assert_de_tokens_error::<GoDuration>(&[Token::U64(u64::MAX)], "time: invalid duration");
        assert_de_tokens_error::<GoDuration>(
            &[Token::String("0")],
            "time: missing unit in duration",
        );
        assert_de_tokens_error::<GoDuration>(
            &[Token::String("10z")],
            "time: unknown unit \"z\" in duration",
        );
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct GoDurationTest {
        pub dur: GoDuration,
        #[serde(with = "super::nanoseconds")]
        pub nanos: GoDuration,
    }

    #[test]
    fn test_json_ser_de() -> Result<(), serde_json::Error> {
        let output: GoDurationTest = serde_json::from_str(r#"{"dur":"20ns","nanos": 17}"#)?;
        let expected = GoDurationTest {
            dur: GoDuration(20),
            nanos: GoDuration(17),
        };
        assert_eq!(expected, output);

        let output = serde_json::to_string(&output).unwrap();
        let expected = r#"{"dur":"20ns","nanos":17}"#;
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_json_de_error() {
        let output = serde_json::from_str::<'_, GoDurationTest>(r#"{"dur":11,"nanos":0}"#);
        assert!(output.is_err());
        let output = output.unwrap_err();
        assert_eq!(
            output.to_string(),
            "invalid type: integer `11`, expected Go-lang style `time.Duration` string at line 1 column 9",
        );

        let output = serde_json::from_str::<'_, GoDurationTest>(r#"{"dur":"0s","nanos":"2s"}"#);
        assert!(output.is_err());
        let output = output.unwrap_err();
        assert_eq!(
            output.to_string(),
            "invalid type: string \"2s\", expected Go-lang style `time.Duration` string at line 1 column 24",
        );
    }
}
