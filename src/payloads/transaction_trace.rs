use serde::{Deserialize, Serialize};

use crate::domain_defs::AgentRunId;
use crate::payloads::analytics_events::{AgentAttrs, UserAttrs};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct CollectorPayload(pub(crate) AgentRunId, pub(crate) Vec<TransactionTrace>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct TransactionTrace(
    pub(crate) i64, // start (nanos)
    // duration (millis)
    pub(crate) f64,
    // final name
    pub(crate) String,
    // request uri
    pub(crate) Option<String>,
    pub(crate) TraceData,
    // CAT GUID
    pub(crate) String,
    // reserved (null)
    pub(crate) (),
    // ForcePersist (false for now)
    pub(crate) bool,
    // X-Ray sessions (null for now)
    pub(crate) (),
    // Synthetics resource id
    pub(crate) String,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct TraceData(
    pub(crate) f64, // unused timestamp (0.0)
    // unused: formerly request parameters
    pub(crate) DummyStruct,
    // unused: formerly custom parameters
    pub(crate) DummyStruct,
    pub(crate) Node,
    pub(crate) Properties,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct DummyStruct {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Node(
    pub(crate) i64, // relativeStartMillis
    // relativeStopMillis
    pub(crate) i64,
    // name
    pub(crate) String,
    pub(crate) NodeAttrs,
    // children
    pub(crate) Vec<Node>,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct NodeAttrs {
    // pub(crate) backtrace: Option<()>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) exclusive_duration_millis: Option<f64>,
    // pub(crate) transaction_guid: Option<String>,
    // #[serde(flatten)]
    // pub(crate) other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Properties {
    pub(crate) agent_attributes: AgentAttrs,
    pub(crate) user_attributes: UserAttrs,
    pub(crate) intrinsics: Intrinsics,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Intrinsics {
    #[serde(rename = "totalTime")]
    pub(crate) total_time: f64,
    // TODO: other intrinsics
}
