// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::apdex::ApdexZone;
use crate::connect_reply::AgentRunId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct CollectorPayload(
    pub(crate) AgentRunId,
    pub(crate) Properties,
    pub(crate) Vec<AnalyticsEventWithAttrs>,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Properties {
    pub(crate) reservoir_size: i32,
    pub(crate) events_seen: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct AnalyticsEventWithAttrs(
    pub(crate) AnalyticsEvent,
    pub(crate) UserAttrs,
    pub(crate) AgentAttrs,
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct UserAttrs {}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct AgentAttrs {
    pub(crate) hash: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum AnalyticsEvent {
    Transaction(TransactionEvent),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct TransactionEvent {
    pub(crate) name: String,
    pub(crate) timestamp: i64,
    #[serde(rename = "nr.apdexPerfZone")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) apdex_perf_zone: Option<ApdexZone>,
    pub(crate) error: bool,
    #[serde(flatten)]
    pub(crate) shared: TransactionShared,
    #[serde(rename = "totalTime")]
    pub(crate) total_time: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct TransactionShared {
    pub(crate) duration: f64,

    #[serde(rename = "queueDuration")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) queue_duration: Option<f64>,

    #[serde(rename = "externalCallCount")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) external_call_count: Option<u64>,
    #[serde(rename = "externalDuration")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) external_duration: Option<f64>,

    #[serde(rename = "databaseCallCount")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) database_call_count: Option<u64>,
    #[serde(rename = "databaseDuration")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) database_duration: Option<f64>,

    #[serde(rename = "nr.syntheticsResourceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) synthetics_resource_id: Option<String>,
    #[serde(rename = "nr.syntheticsJobId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) synthetics_job_id: Option<String>,
    #[serde(rename = "nr.syntheticsMonitorId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) synthetics_monitor_id: Option<String>,
}
