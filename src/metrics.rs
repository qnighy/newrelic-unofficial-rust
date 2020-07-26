use serde::{Deserialize, Serialize};

use crate::connect_reply::AgentRunId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct CollectorPayload(
    pub(crate) AgentRunId,
    /// period start (unix time)
    pub(crate) i64,
    /// period end (unix time)
    pub(crate) i64,
    pub(crate) Vec<(MetricKey, MetricValue)>,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct MetricKey {
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct MetricValue(
    /// count_satisfied
    pub(crate) f64,
    /// total_tolerated
    pub(crate) f64,
    /// exclusive_failed
    pub(crate) f64,
    /// min
    pub(crate) f64,
    /// max
    pub(crate) f64,
    /// sum_squares
    pub(crate) f64,
);
