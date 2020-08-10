use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) mod analytics_events;
pub(crate) mod metrics;
pub(crate) mod transaction_trace;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct UserAttrs {}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct AgentAttrs {
    pub(crate) hash: HashMap<String, serde_json::Value>,
}
