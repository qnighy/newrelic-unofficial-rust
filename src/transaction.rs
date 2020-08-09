// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::analytics_events::{
    AgentAttrs, AnalyticsEvent, AnalyticsEventWithAttrs, TransactionEvent, TransactionShared,
    UserAttrs,
};
use crate::apdex::ApdexZone;
use crate::{AppState, ApplicationInner};

#[derive(Debug)]
pub struct Transaction {
    app: Arc<ApplicationInner>,
    start: Instant,
    name: String,
    web_request: Option<http::Request<()>>,
}

impl Transaction {
    pub(crate) fn new(
        app: &Arc<ApplicationInner>,
        name: &str,
        web_request: Option<http::Request<()>>,
    ) -> Self {
        Transaction {
            app: app.clone(),
            start: Instant::now(),
            name: name.to_owned(),
            web_request,
        }
    }

    fn final_name(&self) -> String {
        // TODO: apply URL rules
        let name = if self.name.starts_with('/') {
            &self.name[1..]
        } else {
            &self.name
        };
        let prefix = if self.web_request.is_some() {
            "WebTransaction/Go"
        } else {
            "OtherTransaction/Go"
        };
        // TODO: apply transaction name rules
        // TODO: apply segment terms
        format!("{}/{}", prefix, name)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        let mut state = self.app.state.lock();
        if let AppState::Running { run, harvest } = &mut *state {
            // Ensure immutability
            let run = &**run;

            let name = self.final_name();
            let name_without_first_segment = if let Some(pos) = name.find('/') {
                &name[pos + 1..]
            } else {
                &name
            };
            let (rollup_name, total_time) = if self.web_request.is_some() {
                ("WebTransaction", "WebTransactionTotalTime")
            } else {
                ("OtherTransaction/all", "OtherTransactionTotalTime")
            };
            let duration = Instant::now()
                .checked_duration_since(self.start)
                .unwrap_or(Duration::from_secs(0));
            let end = SystemTime::now();
            let start = end - duration;
            let mut agent_attrs = AgentAttrs {
                hash: HashMap::new(),
            };
            if let Some(web_request) = &self.web_request {
                agent_attrs.hash.insert(
                    "request.method".to_owned(),
                    web_request.method().to_string().into(),
                );
                agent_attrs.hash.insert(
                    "request.uri".to_owned(),
                    web_request.uri().to_string().into(),
                );
                if let Some(host) = web_request.headers().get("Host") {
                    agent_attrs.hash.insert(
                        "request.headers.host".to_owned(),
                        String::from_utf8_lossy(host.as_bytes()).into_owned().into(),
                    );
                }
            }
            let attrs = AnalyticsEventWithAttrs(
                AnalyticsEvent::Transaction(TransactionEvent {
                    name: name.clone(),
                    timestamp: start.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
                    apdex_perf_zone: if self.web_request.is_some() {
                        // TODO: Apdex T may depend on transaction name
                        Some(ApdexZone::calculate(duration, run.apdex_t))
                    } else {
                        None
                    },
                    error: false,
                    shared: TransactionShared {
                        duration: duration.as_secs_f64(),
                        queue_duration: None,
                        external_call_count: None,
                        external_duration: None,
                        database_call_count: None,
                        database_duration: None,
                        synthetics_resource_id: None,
                        synthetics_job_id: None,
                        synthetics_monitor_id: None,
                    },
                    total_time: duration.as_secs_f64(),
                }),
                UserAttrs {},
                agent_attrs,
            );
            harvest.txn_events.push(attrs);
            harvest
                .metric_table
                .add_duration(&name, None, duration, Duration::from_secs(0), true);
            harvest.metric_table.add_duration(
                rollup_name,
                None,
                duration,
                Duration::from_secs(0),
                true,
            );
            let total_name = format!("{}/{}", total_time, name_without_first_segment);
            harvest
                .metric_table
                .add_duration(&total_name, None, duration, duration, false);
            harvest
                .metric_table
                .add_duration(total_time, None, duration, duration, true);
        }
    }
}
