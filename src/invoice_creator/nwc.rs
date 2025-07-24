use super::InvoiceCreator;
use anyhow::Result;
use nwc::prelude::*;
use std::str::FromStr;

#[async_trait::async_trait]
impl InvoiceCreator for NwcInvoiceCreator {
    async fn create_invoice(&self, amount_msat: u64, description_hash: &str) -> Result<String> {
        let req = MakeInvoiceRequest {
            amount: amount_msat,
            description: None,
            description_hash: Some(description_hash.to_string()),
            expiry: None,
        };
        let invoice = self.nwc.make_invoice(req).await?.invoice;
        Ok(invoice)
    }
}

pub struct NwcInvoiceCreator {
    nwc: NWC,
}

impl NwcInvoiceCreator {
    pub fn new(nwc_str: &str) -> Result<Self> {
        let uri = NostrWalletConnectURI::from_str(nwc_str)?;
        Ok(NwcInvoiceCreator { nwc: NWC::new(uri) })
    }
}
