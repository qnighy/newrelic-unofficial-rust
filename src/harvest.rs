// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use std::time::{Duration, Instant};

use crate::app_run::AppRun;
use crate::collector::{collector_request, RpmError};
use crate::metrics::MetricTable;
use crate::payloads::analytics_events::AnalyticsEventWithAttrs;
use crate::transaction_trace::HarvestTraces;

#[derive(Debug)]
pub(crate) struct Harvest {
    metrics_traces_timer: HarvestTimer,
    span_events_timer: HarvestTimer,
    custom_events_timer: HarvestTimer,
    txn_events_timer: HarvestTimer,
    error_events_timer: HarvestTimer,
    pub(crate) txn_events: Vec<AnalyticsEventWithAttrs>,
    pub(crate) metric_table: MetricTable,
    pub(crate) txn_traces: HarvestTraces,
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
            txn_events: vec![],
            metric_table: MetricTable::new(),
            txn_traces: HarvestTraces::new(),
        }
    }

    // TODO: _run may be totally unnecessary.
    pub(crate) fn ready(&mut self, _run: &AppRun, force: bool) -> HarvestReady {
        let now = Instant::now();
        let mut ready = HarvestReady::default();
        if self.metrics_traces_timer.ready(now, force) {
            eprintln!("Processing metrics traces...");
            self.metric_table
                .add_count(crate::metric_names::INSTANCE_REPORTING, None, 1.0, true);
            ready.metric_table = Some(std::mem::replace(
                &mut self.metric_table,
                MetricTable::new(),
            ));
            ready.txn_traces = Some(std::mem::replace(
                &mut self.txn_traces,
                HarvestTraces::new(),
            ));
        }
        if self.span_events_timer.ready(now, force) {
            eprintln!("Processing span events...");
        }
        if self.custom_events_timer.ready(now, force) {
            eprintln!("Processing custom events...");
        }
        if self.txn_events_timer.ready(now, force) {
            eprintln!("Processing txn events...");
            ready.txn_events = Some(std::mem::replace(&mut self.txn_events, vec![]));
        }
        if self.error_events_timer.ready(now, force) {
            eprintln!("Processing error events...");
        }
        ready
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

#[derive(Debug, Default)]
pub(crate) struct HarvestReady {
    pub(crate) txn_events: Option<Vec<AnalyticsEventWithAttrs>>,
    pub(crate) metric_table: Option<MetricTable>,
    pub(crate) txn_traces: Option<HarvestTraces>,
}

impl HarvestReady {
    pub(crate) fn harvest(self, run: &AppRun) -> Result<(), RpmError> {
        if let Some(metric_table) = self.metric_table {
            eprintln!("Sending metrics traces...");
            let payload = metric_table.payload(&run.agent_run_id);
            // TODO: ignore specific errors & save harvest data when appropriate
            collector_request(run, "metric_data", &payload)?;
        }
        if let Some(txn_traces) = self.txn_traces {
            eprintln!("Sending transaction traces...");
            let payload = txn_traces.into_payload(&run.agent_run_id);
            if !payload.is_empty() {
                // TODO: ignore specific errors & save harvest data when appropriate
                collector_request(run, "transaction_sample_data", &payload)?;
            }
        }
        if let Some(txn_events) = self.txn_events {
            use crate::payloads::analytics_events::{CollectorPayload, Properties};

            eprintln!("Sending txn events...");
            // TODO: ignore specific errors & save harvest data when appropriate
            collector_request(
                run,
                "analytic_event_data",
                &CollectorPayload {
                    agent_run_id: run.agent_run_id.clone(),
                    properties: Properties {
                        reservoir_size: txn_events.capacity() as i32,
                        events_seen: txn_events.len() as i32,
                    },
                    events: txn_events,
                },
            )?;
        }

        Ok(())
    }
}
