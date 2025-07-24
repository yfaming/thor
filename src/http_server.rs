use crate::config::Config;
use crate::error::{HttpError, Lud06Error};
use crate::invoice_creator::{InvoiceCreator, NwcInvoiceCreator};
use anyhow::Result;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::get;
use bitcoin_hashes::Sha256;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AppState {
    domain: String,
    users: HashMap<String, Vec<Box<dyn InvoiceCreator>>>,
}

impl AppState {
    pub fn new(config: &Config) -> Result<AppState> {
        let mut state = AppState {
            domain: config.server.domain.clone(),
            users: HashMap::new(),
        };

        for user_config in &config.users {
            let mut invoice_creators: Vec<Box<dyn InvoiceCreator>> = vec![];
            for nwc_str in &user_config.nwcs {
                let nwc_invoice_creator = NwcInvoiceCreator::new(nwc_str)?;
                invoice_creators.push(Box::new(nwc_invoice_creator));
            }
            state
                .users
                .insert(user_config.name.clone(), invoice_creators);
        }

        Ok(state)
    }
}

// lightning address specs:
// - [LUD-16: Paying to static internet identifiers](https://github.com/lnurl/luds/blob/luds/16.md)
// - [LUD-06: payRequest base spec](https://github.com/lnurl/luds/blob/luds/06.md)
pub async fn run_http_server(config: &Config) -> Result<()> {
    let state = Arc::new(AppState::new(&config)?);

    let app = Router::new()
        .route("/.well-known/lnurlp/{username}", get(get_lnurlp_info))
        .route("/lnurlp/{username}", get(create_invoice))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.server.listen_addr).await?;
    tracing::info!("listening on {}", config.server.listen_addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_lnurlp_info(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Json<LnUrlPayInfo>, HttpError> {
    if state.users.get(&username).is_none() {
        let e = Lud06Error::new(format!("user {} not found", username));
        return Err(HttpError::new(StatusCode::BAD_REQUEST, e));
    }

    let metadata = LnUrlPayInfo {
        callback: format!("https://{}/lnurlp/{}", state.domain, username),
        max_sendable: 100_000_000_000, // 1 bitcoin
        min_sendable: 1_000,           // 1 sat
        metadata: generate_metadata(&state, &username)?,
        tag: "payRequest",
    };
    Ok(Json(metadata))
}

fn generate_metadata(state: &AppState, username: &str) -> Result<String> {
    // LUD-16 requires that there must be either a `text/identifier` or a `text/email` metadata entry.
    let v = serde_json::json!([
        [
            "text/identifier".to_string(),
            format!("{}@{}", username, state.domain)
        ],
        [
            "text/plain".to_string(),
            format!("sats for {}@{}", username, state.domain)
        ],
        [
            "text/plain".to_string(),
            "powered by https://github.com/yfaming/thor".to_string()
        ],
    ]);
    let metadata_str = serde_json::to_string(&v)?;
    Ok(metadata_str)
}

#[derive(Debug, Serialize, Deserialize)]
struct LnUrlPayInfo {
    callback: String,
    #[serde(rename = "maxSendable")]
    max_sendable: u64, // msat
    #[serde(rename = "minSendable")]
    min_sendable: u64, // msat
    metadata: String,
    tag: &'static str, // "payRequest"
}

async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Query(amount): Query<Amount>,
) -> Result<Json<InvoiceResponse>, HttpError> {
    if amount.amount == 0 {
        let e = Lud06Error::new("amount must > 0".to_string());
        return Err(HttpError::new(StatusCode::BAD_REQUEST, e));
    }

    let creators = match state.users.get(&username) {
        Some(creators) => {
            let mut creators: Vec<_> = creators.iter().map(|creator| creator.as_ref()).collect();
            creators.shuffle(&mut rand::rng());
            creators
        }
        None => {
            let e = Lud06Error::new(format!("user {} not found", username));
            return Err(HttpError::new(StatusCode::BAD_REQUEST, e));
        }
    };

    // LUD-06 requires that we use the hash of the metadata as `description_hash` of invoice.
    let metadata = generate_metadata(&state, &username)?;
    let description_hash = format!("{}", Sha256::hash(metadata.as_bytes()));

    // attempt at most 3 times
    let mut last_err = None;
    for creator in creators.iter().take(3) {
        match creator
            .create_invoice(amount.amount, &description_hash)
            .await
        {
            Ok(invoice) => {
                tracing::info!(username = username, invoice = invoice, "invoice created.");
                return Ok(Json(InvoiceResponse {
                    pr: invoice,
                    routes: vec![],
                }));
            }
            Err(e) => {
                tracing::warn!(user = username, error = %e, "failed to create invoice.");
                last_err = Some(e);
            }
        };
    }

    match last_err {
        Some(e) => {
            tracing::error!(user = username, error = %e, "failed to create invoice. All attempts failed.");
            Err(e.into())
        }
        None => unreachable!(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pr: String,          // invoice
    routes: Vec<String>, // empty
}

#[derive(Debug, Deserialize)]
struct Amount {
    amount: u64,
}
