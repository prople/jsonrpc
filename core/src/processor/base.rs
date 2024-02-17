use rst_common::with_errors::anyhow::Result;
use rst_common::standard::async_trait::async_trait;
use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::standard::serde_json::Value;

#[async_trait]
pub trait Handler {
    async fn call(&self, params: Value) -> Result<Option<Box<dyn ErasedSerialized>>>;
}

pub type Method = String;
