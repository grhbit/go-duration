use serde::{de, ser};

use crate::{parse_go_duration, GoDuration};

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
        formatter.write_str("a formatted string or nanoseconds(i64)")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(GoDuration { nanos: v })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(GoDuration {
            nanos: parse_go_duration(v).unwrap().1,
        })
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<GoDuration, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_i64(GoDurationVisitor)
}
