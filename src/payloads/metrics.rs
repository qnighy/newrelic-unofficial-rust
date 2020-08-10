// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use serde::{Deserialize, Serialize};

use crate::domain_defs::AgentRunId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct CollectorPayload(
    pub(crate) AgentRunId,
    /// period start (unix time)
    pub(crate) i64,
    /// period end (unix time)
    pub(crate) i64,
    pub(crate) Vec<(MetricId, MetricValue)>,
);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct MetricId {
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
