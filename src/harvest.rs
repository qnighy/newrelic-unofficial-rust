use std::time::{Duration, Instant};

use crate::app_run::AppRun;
use crate::collector::collector_request;

#[derive(Debug)]
pub(crate) struct Harvest {
    metrics_traces_timer: HarvestTimer,
    span_events_timer: HarvestTimer,
    custom_events_timer: HarvestTimer,
    txn_events_timer: HarvestTimer,
    error_events_timer: HarvestTimer,
}

impl Harvest {
    pub(crate) fn new(run: &AppRun) -> Self {
        let last_harvest = Instant::now();
        let new_timer = |duration: Duration| HarvestTimer {
            duration,
            last_harvest,
        };
        Self {
            metrics_traces_timer: new_timer(run.metrics_traces_period),
            span_events_timer: new_timer(run.span_events_period),
            custom_events_timer: new_timer(run.custom_events_period),
            txn_events_timer: new_timer(run.txn_events_period),
            error_events_timer: new_timer(run.error_events_period),
        }
    }

    pub(crate) fn harvest(&mut self, run: &AppRun, force: bool) {
        let now = Instant::now();
        if self.metrics_traces_timer.ready(now, force) {
            eprintln!("Processing metrics traces...");
        }
        if self.span_events_timer.ready(now, force) {
            eprintln!("Processing span events...");
        }
        if self.custom_events_timer.ready(now, force) {
            eprintln!("Processing custom events...");
        }
        if self.txn_events_timer.ready(now, force) {
            use crate::analytics_events::{
                AgentAttrs, AnalyticsEvent, AnalyticsEventWithAttrs, CollectorPayload, Properties,
                TransactionEvent, TransactionShared, UserAttrs,
            };
            use std::time::{SystemTime, UNIX_EPOCH};
            eprintln!("Processing txn events...");
            // TODO: handle errors
            collector_request(
                run,
                "analytic_event_data",
                &CollectorPayload(
                    run.agent_run_id.clone(),
                    Properties {
                        // TODO: use cap
                        reservoir_size: 833,
                        events_seen: 1,
                    },
                    vec![AnalyticsEventWithAttrs(
                        AnalyticsEvent::Transaction(TransactionEvent {
                            name: "OtherTransaction/Go/test".to_owned(),
                            timestamp: (SystemTime::now() - Duration::from_secs(20))
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64,
                            apdex_perf_zone: None,
                            error: false,
                            shared: TransactionShared {
                                duration: 20.0,
                                queue_duration: None,
                                external_call_count: None,
                                external_duration: None,
                                database_call_count: None,
                                database_duration: None,
                                synthetics_resource_id: None,
                                synthetics_job_id: None,
                                synthetics_monitor_id: None,
                            },
                            total_time: 20.0,
                        }),
                        UserAttrs {},
                        AgentAttrs {},
                    )],
                ),
            )
            .unwrap();
        }
        if self.error_events_timer.ready(now, force) {
            eprintln!("Processing error events...");
        }
    }
}

#[derive(Debug)]
struct HarvestTimer {
    duration: Duration,
    last_harvest: Instant,
}

impl HarvestTimer {
    fn ready(&mut self, now: Instant, force: bool) -> bool {
        if force || now >= self.last_harvest + self.duration {
            self.last_harvest = now;
            true
        } else {
            false
        }
    }
}
