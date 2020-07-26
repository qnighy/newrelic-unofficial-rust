use std::collections::HashMap;
use std::time::Duration;

use crate::connect_reply::{AgentRunId, ConnectReply, PreconnectReply};
use crate::limits::{DEFAULT_CONFIGURABLE_EVENT_HARVEST, FIXED_HARVEST_PERIOD};

#[derive(Debug)]
pub(crate) struct AppRun {
    pub(crate) host: String,
    // TODO: dedup with config
    pub(crate) license: String,

    pub(crate) agent_run_id: AgentRunId,
    pub(crate) request_headers_map: HashMap<String, String>,
    pub(crate) metrics_traces_period: Duration,
    pub(crate) span_events_period: Duration,
    pub(crate) custom_events_period: Duration,
    pub(crate) txn_events_period: Duration,
    pub(crate) error_events_period: Duration,
}

impl AppRun {
    pub(crate) fn new(license: &str, reply_pre: &PreconnectReply, reply: &ConnectReply) -> AppRun {
        let configurable_period = if let Some(ms) = reply.event_harvest_config.report_period_ms {
            Duration::from_millis(u64::from(ms))
        } else {
            DEFAULT_CONFIGURABLE_EVENT_HARVEST
        };
        let select_period = |x: Option<u32>| {
            if x.is_some() {
                configurable_period
            } else {
                FIXED_HARVEST_PERIOD
            }
        };
        Self {
            host: reply_pre.redirect_host.clone(),
            license: license.to_owned(),

            agent_run_id: reply.agent_run_id.clone(),
            request_headers_map: reply.request_headers_map.clone(),
            metrics_traces_period: FIXED_HARVEST_PERIOD,
            span_events_period: select_period(
                reply.event_harvest_config.harvest_limits.span_event_data,
            ),
            custom_events_period: select_period(
                reply.event_harvest_config.harvest_limits.custom_event_data,
            ),
            txn_events_period: select_period(
                reply
                    .event_harvest_config
                    .harvest_limits
                    .analytic_event_data,
            ),
            error_events_period: select_period(
                reply.event_harvest_config.harvest_limits.error_event_data,
            ),
        }
    }
}
