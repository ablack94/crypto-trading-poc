use axum::{http::StatusCode, response::IntoResponse, Json};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_urlencoded::ser::Error as UrlEncodedError;
use sha2::{Digest, Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::models::{AddOrderRequest, AddOrderResponse, CancelOrderResponse, QueryOrdersResponse};

type HmacSha512 = Hmac<Sha512>;

#[derive(Clone)]
pub struct KrakenClient {
    api_key: String,
    secret: Vec<u8>,
    base_url: String,
    http: Client,
}

impl KrakenClient {
    pub fn new(
        api_key: String,
        api_secret: String,
        base_url: String,
    ) -> Result<Self, KrakenClientError> {
        let secret = BASE64_STANDARD
            .decode(api_secret.as_bytes())
            .map_err(KrakenClientError::SecretDecode)?;

        let http = Client::builder()
            .user_agent("crypto-trading-poc/0.1")
            .build()
            .map_err(KrakenClientError::HttpClient)?;

        Ok(Self {
            api_key,
            secret,
            base_url,
            http,
        })
    }

    pub async fn add_order(
        &self,
        payload: AddOrderRequest,
    ) -> Result<AddOrderResponse, KrakenClientError> {
        self.private_post("AddOrder", &payload).await
    }

    pub async fn cancel_order(&self, txid: &str) -> Result<CancelOrderResponse, KrakenClientError> {
        #[derive(Serialize)]
        struct CancelPayload<'a> {
            txid: &'a str,
        }

        self.private_post("CancelOrder", &CancelPayload { txid })
            .await
    }

    pub async fn query_order(
        &self,
        txid: &str,
        trades: Option<bool>,
    ) -> Result<QueryOrdersResponse, KrakenClientError> {
        #[derive(Serialize)]
        struct QueryPayload<'a> {
            txid: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            trades: Option<bool>,
        }

        self.private_post("QueryOrders", &QueryPayload { txid, trades })
            .await
    }

    async fn private_post<T, R>(&self, endpoint: &str, payload: &T) -> Result<R, KrakenClientError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let nonce = current_nonce();
        let encoded_payload =
            serde_urlencoded::to_string(payload).map_err(KrakenClientError::Serialize)?;
        let post_body = if encoded_payload.is_empty() {
            format!("nonce={nonce}")
        } else {
            format!("nonce={nonce}&{encoded_payload}")
        };

        let path = format!("/0/private/{endpoint}");
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let signature = self.sign(&path, &nonce, &post_body)?;

        let response = self
            .http
            .post(url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .body(post_body)
            .send()
            .await
            .map_err(KrakenClientError::Http)?;

        if !response.status().is_success() {
            return Err(KrakenClientError::HttpStatus(response.status()));
        }

        let parsed: KrakenResponse<R> = response.json().await.map_err(KrakenClientError::Http)?;
        if !parsed.error.is_empty() {
            return Err(KrakenClientError::Kraken(parsed.error));
        }

        Ok(parsed.result)
    }

    fn sign(&self, path: &str, nonce: &str, body: &str) -> Result<String, KrakenClientError> {
        let mut sha = Sha256::new();
        sha.update(nonce.as_bytes());
        sha.update(body.as_bytes());
        let hash = sha.finalize();

        let mut mac = HmacSha512::new_from_slice(&self.secret).map_err(KrakenClientError::Hmac)?;
        mac.update(path.as_bytes());
        mac.update(&hash);
        let signature = mac.finalize().into_bytes();
        Ok(BASE64_STANDARD.encode(signature))
    }
}

fn current_nonce() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    now.as_millis().to_string()
}

#[derive(Debug, Deserialize)]
struct KrakenResponse<T> {
    error: Vec<String>,
    result: T,
}

#[derive(Debug, Error)]
pub enum KrakenClientError {
    #[error("missing Kraken credentials: {0}")]
    Config(&'static str),
    #[error("could not build HTTP client: {0}")]
    HttpClient(reqwest::Error),
    #[error("request failed: {0}")]
    Http(reqwest::Error),
    #[error("unexpected HTTP status: {0}")]
    HttpStatus(StatusCode),
    #[error("failed to serialize request: {0}")]
    Serialize(UrlEncodedError),
    #[error("kraken returned errors: {0:?}")]
    Kraken(Vec<String>),
    #[error("invalid API secret: {0}")]
    SecretDecode(base64::DecodeError),
    #[error("failed to initialize signer: {0}")]
    Hmac(hmac::digest::InvalidLength),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl KrakenClientError {
    pub fn missing_key() -> Self {
        Self::Config("KRAKEN_API_KEY")
    }

    pub fn missing_secret() -> Self {
        Self::Config("KRAKEN_API_SECRET")
    }
}

impl IntoResponse for KrakenClientError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorBody {
            error: String,
        }

        let status = match self {
            KrakenClientError::Config(_) => StatusCode::BAD_REQUEST,
            KrakenClientError::Kraken(_) => StatusCode::BAD_GATEWAY,
            KrakenClientError::HttpStatus(code) => code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

pub fn build_client_from_env() -> Result<KrakenClient, KrakenClientError> {
    let api_key = std::env::var("KRAKEN_API_KEY").map_err(|_| KrakenClientError::missing_key())?;
    let api_secret =
        std::env::var("KRAKEN_API_SECRET").map_err(|_| KrakenClientError::missing_secret())?;
    let base_url = std::env::var("KRAKEN_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.kraken.com".to_string());

    KrakenClient::new(api_key, api_secret, base_url)
}
