use rst_common::standard::serde::de::{self, Visitor};
use rst_common::standard::serde::{self, Deserialize, Serialize};

/// `RpcId` used to modeling the request and response `id`, which is an identifier
///
/// Taken from it's specs: <https://www.jsonrpc.org/specification#request_object>
///
/// > **About Id**
/// >
/// > An identifier established by the Client that MUST contain a String, Number, 
/// > or NULL value if included. If it is not included it is assumed to be a notification. 
/// > The value SHOULD normally not be Null and Numbers SHOULD NOT contain fractional parts
///
/// This object will implement [`serde::de::Visitor`] used to parse given json string and need to
/// parse the `id` value based on it's type, an integer or a string
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
