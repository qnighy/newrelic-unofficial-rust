// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::apdex::ApdexZone;
use crate::domain_defs::AgentRunId;
use crate::payloads::{AgentAttrs, UserAttrs};

#[derive(Debug, Clone)]
pub(crate) struct CollectorPayload {
    pub(crate) agent_run_id: AgentRunId,
    pub(crate) properties: Properties,
    pub(crate) events: Vec<AnalyticsEventWithAttrs>,
}

impl Serialize for CollectorPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.agent_run_id)?;
        tup.serialize_element(&self.properties)?;
        tup.serialize_element(&self.events)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for CollectorPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            agent_run_id: tup.0,
            properties: tup.1,
            events: tup.2,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Properties {
    pub(crate) reservoir_size: i32,
    pub(crate) events_seen: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct AnalyticsEventWithAttrs {
    pub(crate) event: AnalyticsEvent,
    pub(crate) user_attrs: UserAttrs,
    pub(crate) agent_attrs: AgentAttrs,
}

impl Serialize for AnalyticsEventWithAttrs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.event)?;
        tup.serialize_element(&self.user_attrs)?;
        tup.serialize_element(&self.agent_attrs)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for AnalyticsEventWithAttrs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            event: tup.0,
            user_attrs: tup.1,
            agent_attrs: tup.2,
        })
    }
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
