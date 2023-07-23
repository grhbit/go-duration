use std::str::FromStr;

use ::serde::{de, ser};

use crate::GoDuration;

pub fn serialize<S>(dur: &GoDuration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    serializer.serialize_str(&dur.to_string())
}

pub struct GoDurationVisitor;

impl<'de> de::Visitor<'de> for GoDurationVisitor {
    type Value = GoDuration;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Go-lang style `time.Duration` string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        GoDuration::from_str(v).map_err(E::custom)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<GoDuration, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_str(GoDurationVisitor)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    use super::*;

    #[test]
    fn test_ser_de() {
        let dur = GoDuration(0);
        assert_tokens(
            &dur,
            &[Token::NewtypeStruct { name: "GoDuration" }, Token::I64(0)],
        );
    }

    #[test]
    fn test_de_error() {
        assert_de_tokens_error::<GoDuration>(
            &[
                Token::NewtypeStruct { name: "GoDuration" },
                Token::U64(u64::MAX),
            ],
            "invalid value: integer `18446744073709551615`, expected i64",
        );
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct GoDurationTest {
        #[serde(with = "super")]
        pub dur: GoDuration,
    }

    #[test]
    fn test_json_ser_de() -> Result<(), serde_json::Error> {
        let output: GoDurationTest = serde_json::from_str(r#"{"dur":"20ns"}"#)?;
        let expected = GoDurationTest {
            dur: GoDuration(20),
        };
        assert_eq!(expected, output);

        let output = serde_json::to_string(&output).unwrap();
        let expected = r#"{"dur":"20ns"}"#;
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_json_de_error() {
        let output = serde_json::from_str::<'_, GoDurationTest>(r#"{"dur":11}"#);
        assert!(output.is_err());
        let output = output.unwrap_err();
        assert_eq!(
            output.to_string(),
            "invalid type: integer `11`, expected Go-lang style `time.Duration` string at line 1 column 9",
        );
    }
}
