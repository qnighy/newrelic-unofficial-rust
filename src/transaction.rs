// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::apdex::ApdexZone;
use crate::payloads::analytics_events::{
    AnalyticsEvent, AnalyticsEventWithAttrs, TransactionEvent, TransactionShared,
};
use crate::payloads::{AgentAttrs, UserAttrs};
use crate::{AppState, ApplicationInner};

const MAIN_THREAD_ID: usize = 0;

#[derive(Debug)]
pub struct TransactionGuard {
    txn: Transaction,
}

impl std::ops::Deref for TransactionGuard {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}

impl std::ops::Drop for TransactionGuard {
    fn drop(&mut self) {
        self.txn.inner.stop();
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    inner: Arc<TransactionInner>,
    thread_id: usize,
}

impl Transaction {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(
        app: &Arc<ApplicationInner>,
        name: &str,
        web_request: Option<WebRequest>,
    ) -> TransactionGuard {
        let now = Instant::now();
        TransactionGuard {
            txn: Transaction {
                inner: Arc::new(TransactionInner {
                    app: app.clone(),
                    start: now,
                    name: name.to_owned(),
                    web_request,
                    state: Mutex::new(Some(TransactionState::new(now))),
                }),
                thread_id: MAIN_THREAD_ID,
            },
        }
    }
}

#[derive(Debug)]
struct TransactionInner {
    app: Arc<ApplicationInner>,
    start: Instant,
    name: String,
    web_request: Option<WebRequest>,
    state: Mutex<Option<TransactionState>>,
}

impl TransactionInner {
    fn final_name(&self) -> String {
        // TODO: apply URL rules
        let name = if self.name.starts_with('/') {
            &self.name[1..]
        } else {
            &self.name
        };
        let prefix = if self.web_request.is_some() {
            crate::metric_names::WEB_METRIC_PREFIX
        } else {
            crate::metric_names::BACKGROUND_METRIC_PREFIX
        };
        // TODO: apply transaction name rules
        // TODO: apply segment terms
        format!("{}/{}", prefix, name)
    }

    fn stop(&self) {
        let is_web = self.web_request.is_some();
        let mut state = self.app.state.lock();
        if let AppState::Running { run, harvest } = &mut *state {
            // Ensure immutability
            let run = &**run;

            let name = self.final_name();
            let duration = Instant::now()
                .checked_duration_since(self.start)
                .unwrap_or_else(|| Duration::from_secs(0));
            let end = SystemTime::now();
            let start = end - duration;
            let start_from_unix = start.duration_since(UNIX_EPOCH).unwrap_or_default();
            let mut agent_attrs = AgentAttrs::default();
            if let Some(web_request) = &self.web_request {
                agent_attrs.0.insert(
                    "request.method".to_owned(),
                    web_request.method.to_string().into(),
                );
                agent_attrs
                    .0
                    .insert("request.uri".to_owned(), web_request.uri.to_string().into());
                if let Some(host) = web_request.headers.get("Host") {
                    agent_attrs.0.insert(
                        "request.headers.host".to_owned(),
                        String::from_utf8_lossy(host.as_bytes()).into_owned().into(),
                    );
                }
            }
            let attrs = AnalyticsEventWithAttrs {
                event: AnalyticsEvent::Transaction(TransactionEvent {
                    name: name.clone(),
                    timestamp: start_from_unix.as_secs() as i64,
                    apdex_perf_zone: if is_web {
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
                user_attrs: UserAttrs::default(),
                agent_attrs: agent_attrs.clone(),
            };
            harvest.txn_events.push(attrs);
            harvest
                .metric_table
                .add_duration(&name, None, duration, Duration::from_secs(0), true);
            let rollup_name = crate::metric_names::rollup_name(is_web);
            harvest.metric_table.add_duration(
                rollup_name,
                None,
                duration,
                Duration::from_secs(0),
                true,
            );
            if is_web {
                harvest.metric_table.add_duration(
                    crate::metric_names::DISPATCHER_METRIC,
                    None,
                    duration,
                    Duration::from_secs(0),
                    true,
                );
            }
            let total_name = crate::metric_names::total_time_name(&name, is_web);
            let total_rollup_name = crate::metric_names::total_time_rollup_name(is_web);
            harvest
                .metric_table
                .add_duration(&total_name, None, duration, duration, false);
            harvest
                .metric_table
                .add_duration(total_rollup_name, None, duration, duration, true);

            // TODO: check is_synthetics
            // TODO: duration and is_apdex_failing configs
            let should_save_trace = self.app.config.transaction_tracer.enabled
                && duration >= Duration::from_millis(500);
            if should_save_trace {
                use crate::payloads::transaction_trace::{
                    DummyStruct, Intrinsics, Node, NodeAttrs, Properties, TraceData,
                    TransactionTrace,
                };

                let trace = TransactionTrace {
                    start: start_from_unix.as_micros() as i64,
                    duration: duration.as_secs_f64() * 1000.0,
                    name: name.clone(),
                    request_uri: self
                        .web_request
                        .as_ref()
                        .map(|web_request| web_request.uri.to_string()),
                    trace_data: TraceData {
                        unused1: 0.0,
                        unused2: DummyStruct {},
                        unused3: DummyStruct {},
                        node: Node {
                            relative_start_millis: 0,
                            relative_stop_millis: duration.as_millis() as i64,
                            name: "ROOT".to_owned(),
                            attrs: NodeAttrs {
                                exclusive_duration_millis: None,
                            },
                            children: vec![Node {
                                relative_start_millis: 0,
                                relative_stop_millis: duration.as_millis() as i64,
                                name,
                                attrs: NodeAttrs {
                                    exclusive_duration_millis: Some(
                                        duration.as_secs_f64() * 1000.0,
                                    ),
                                },
                                children: vec![],
                            }],
                        },
                        properties: Properties {
                            agent_attributes: agent_attrs,
                            user_attributes: UserAttrs::default(),
                            intrinsics: Intrinsics {
                                total_time: duration.as_secs_f64(),
                            },
                        },
                    },
                    cat_guid: "".to_owned(),
                    reserved1: (),
                    force_persist: false,
                    xray_session: (),
                    synthetics_resource_id: "".to_owned(),
                };
                harvest.txn_traces.push(trace);
            }
        }
    }
}

#[derive(Debug)]
struct TransactionState {
    threads: Vec<Thread>,
}

impl TransactionState {
    fn new(now: Instant) -> Self {
        Self {
            threads: vec![Thread::new(now)],
        }
    }
}

#[derive(Debug)]
struct Thread {
    start: Instant,
    end: Option<Instant>,
}

impl Thread {
    fn new(now: Instant) -> Self {
        Self {
            start: now,
            end: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WebRequest {
    pub version: http::Version,
    pub method: http::Method,
    pub uri: http::Uri,
    pub headers: http::HeaderMap,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl From<http::request::Parts> for WebRequest {
    fn from(req: http::request::Parts) -> Self {
        WebRequest {
            version: req.version,
            method: req.method,
            uri: req.uri,
            headers: req.headers,
            __non_exhaustive: (),
        }
    }
}

impl<'a> From<&'a http::request::Parts> for WebRequest {
    fn from(req: &'a http::request::Parts) -> Self {
        WebRequest {
            version: req.version,
            method: req.method.clone(),
            uri: req.uri.clone(),
            headers: req.headers.clone(),
            __non_exhaustive: (),
        }
    }
}

impl<T> From<http::Request<T>> for WebRequest {
    fn from(req: http::Request<T>) -> Self {
        let (parts, _) = req.into_parts();
        WebRequest::from(parts)
    }
}

impl<'a, T> From<&'a http::Request<T>> for WebRequest {
    fn from(req: &'a http::Request<T>) -> Self {
        WebRequest {
            version: req.version(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            headers: req.headers().clone(),
            __non_exhaustive: (),
        }
    }
}
