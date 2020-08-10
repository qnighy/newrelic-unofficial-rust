// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::domain_defs::AgentRunId;

#[derive(Debug, Clone)]
pub(crate) struct CollectorPayload {
    pub(crate) agent_run_id: AgentRunId,
    /// period start (unix time)
    pub(crate) start: i64,
    /// period end (unix time)
    pub(crate) end: i64,
    pub(crate) metrics: Vec<(MetricId, MetricValue)>,
}

impl Serialize for CollectorPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.agent_run_id)?;
        tup.serialize_element(&self.start)?;
        tup.serialize_element(&self.end)?;
        tup.serialize_element(&self.metrics)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for CollectorPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            agent_run_id: tup.0,
            start: tup.1,
            end: tup.2,
            metrics: tup.3,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct MetricId {
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MetricValue {
    pub(crate) count_satisfied: f64,
    pub(crate) total_tolerated: f64,
    pub(crate) exclusive_failed: f64,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) sum_squares: f64,
}

impl Serialize for MetricValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(6)?;
        tup.serialize_element(&self.count_satisfied)?;
        tup.serialize_element(&self.total_tolerated)?;
        tup.serialize_element(&self.exclusive_failed)?;
        tup.serialize_element(&self.min)?;
        tup.serialize_element(&self.max)?;
        tup.serialize_element(&self.sum_squares)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for MetricValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _, _, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            count_satisfied: tup.0,
            total_tolerated: tup.1,
            exclusive_failed: tup.2,
            min: tup.3,
            max: tup.4,
            sum_squares: tup.5,
        })
    }
}
