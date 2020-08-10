use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) mod analytics_events;
pub(crate) mod metrics;
pub(crate) mod transaction_trace;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub(crate) struct UserAttrs(pub(crate) HashMap<String, serde_json::Value>);
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub(crate) struct AgentAttrs(pub(crate) HashMap<String, serde_json::Value>);
