use anyhow::Result;

pub mod nwc;

#[async_trait::async_trait]
pub trait InvoiceCreator: Send + Sync {
    async fn create_invoice(&self, amount_msat: u64, description_hash: &str) -> Result<String>;
}

pub use nwc::NwcInvoiceCreator;
