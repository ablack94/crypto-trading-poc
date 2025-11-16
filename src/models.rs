use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct AddOrderRequest {
    pub pair: String,
    #[serde(rename = "type")]
    pub side: OrderSide,
    #[serde(rename = "ordertype")]
    pub order_kind: OrderKind,
    pub volume: String,
    pub price: Option<String>,
    pub price2: Option<String>,
    pub leverage: Option<String>,
    pub oflags: Option<String>,
    pub starttm: Option<String>,
    pub expiretm: Option<String>,
    pub userref: Option<i32>,
    pub validate: Option<bool>,
    pub timeinforce: Option<TimeInForce>,
    #[serde(flatten)]
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddOrderResponse {
    pub descr: OrderDescription,
    pub txid: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub count: u32,
    pub pending: Option<bool>,
}

pub type QueryOrdersResponse = BTreeMap<String, OrderInfo>;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderInfo {
    pub refid: Option<String>,
    pub userref: Option<i32>,
    pub status: OrderStatus,
    pub opentm: f64,
    pub starttm: f64,
    pub expiretm: f64,
    pub descr: OrderDescription,
    #[serde(default)]
    pub vol: String,
    #[serde(default)]
    pub vol_exec: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub fee: String,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub stopprice: String,
    #[serde(default)]
    pub limitprice: String,
    pub misc: Option<String>,
    pub oflags: Option<String>,
    pub trades: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDescription {
    pub pair: Option<String>,
    pub position: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<OrderSide>,
    #[serde(rename = "ordertype")]
    pub order_kind: Option<OrderKind>,
    pub price: Option<String>,
    pub price2: Option<String>,
    pub leverage: Option<String>,
    pub order: Option<String>,
    pub close: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum OrderKind {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    SettlePosition,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Gtd,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Open,
    Closed,
    Canceled,
    Expired,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct QueryOrderQuery {
    #[serde(default)]
    pub trades: Option<bool>,
}
