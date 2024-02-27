use rst_common::standard::serde::de::{self, Visitor};
use rst_common::standard::serde::{self, Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum RpcId {
    StringVal(String),
    IntegerVal(u64),
}

struct RpcIdVisitor;

impl<'de> Visitor<'de> for RpcIdVisitor {
    type Value = RpcId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expected a value of string or integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RpcId::StringVal(v.to_string()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RpcId::IntegerVal(v))
    }
}

impl Serialize for RpcId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RpcId::IntegerVal(val) => serializer.serialize_u64(*val),
            RpcId::StringVal(val) => serializer.serialize_str(val.as_str()),
        }
    }
}

impl<'de> Deserialize<'de> for RpcId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RpcIdVisitor)
    }
}
