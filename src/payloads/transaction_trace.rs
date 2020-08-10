use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::domain_defs::AgentRunId;
use crate::payloads::{AgentAttrs, UserAttrs};

#[derive(Debug, Clone)]
pub(crate) struct CollectorPayload {
    pub(crate) agent_run_id: AgentRunId,
    pub(crate) traces: Vec<TransactionTrace>,
}

impl CollectorPayload {
    pub(crate) fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }
}

impl Serialize for CollectorPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.agent_run_id)?;
        tup.serialize_element(&self.traces)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for CollectorPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _)>::deserialize(deserializer)?;
        Ok(Self {
            agent_run_id: tup.0,
            traces: tup.1,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TransactionTrace {
    // start (nanos)
    pub(crate) start: i64,
    // duration (millis)
    pub(crate) duration: f64,
    // final name
    pub(crate) name: String,
    // request uri
    pub(crate) request_uri: Option<String>,
    pub(crate) trace_data: TraceData,
    // CAT GUID
    pub(crate) cat_guid: String,
    // reserved (null)
    pub(crate) reserved1: (),
    // ForcePersist (false for now)
    pub(crate) force_persist: bool,
    // X-Ray sessions (null for now)
    pub(crate) xray_session: (),
    // Synthetics resource id
    pub(crate) synthetics_resource_id: String,
}

impl Serialize for TransactionTrace {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(10)?;
        tup.serialize_element(&self.start)?;
        tup.serialize_element(&self.duration)?;
        tup.serialize_element(&self.name)?;
        tup.serialize_element(&self.request_uri)?;
        tup.serialize_element(&self.trace_data)?;
        tup.serialize_element(&self.cat_guid)?;
        tup.serialize_element(&self.reserved1)?;
        tup.serialize_element(&self.force_persist)?;
        tup.serialize_element(&self.xray_session)?;
        tup.serialize_element(&self.synthetics_resource_id)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for TransactionTrace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _, _, _, _, _, _, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            start: tup.0,
            duration: tup.1,
            name: tup.2,
            request_uri: tup.3,
            trace_data: tup.4,
            cat_guid: tup.5,
            reserved1: tup.6,
            force_persist: tup.7,
            xray_session: tup.8,
            synthetics_resource_id: tup.9,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TraceData {
    // unused timestamp (0.0)
    pub(crate) unused1: f64,
    // unused: formerly request parameters
    pub(crate) unused2: DummyStruct,
    // unused: formerly custom parameters
    pub(crate) unused3: DummyStruct,
    pub(crate) node: Node,
    pub(crate) properties: Properties,
}

impl Serialize for TraceData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(5)?;
        tup.serialize_element(&self.unused1)?;
        tup.serialize_element(&self.unused2)?;
        tup.serialize_element(&self.unused3)?;
        tup.serialize_element(&self.node)?;
        tup.serialize_element(&self.properties)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for TraceData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            unused1: tup.0,
            unused2: tup.1,
            unused3: tup.2,
            node: tup.3,
            properties: tup.4,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct DummyStruct {}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub(crate) relative_start_millis: i64,
    pub(crate) relative_stop_millis: i64,
    pub(crate) name: String,
    pub(crate) attrs: NodeAttrs,
    pub(crate) children: Vec<Node>,
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(5)?;
        tup.serialize_element(&self.relative_start_millis)?;
        tup.serialize_element(&self.relative_stop_millis)?;
        tup.serialize_element(&self.name)?;
        tup.serialize_element(&self.attrs)?;
        tup.serialize_element(&self.children)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tup = <(_, _, _, _, _)>::deserialize(deserializer)?;
        Ok(Self {
            relative_start_millis: tup.0,
            relative_stop_millis: tup.1,
            name: tup.2,
            attrs: tup.3,
            children: tup.4,
        })
    }
}

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
