// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use crate::domain_defs::AgentRunId;
use crate::payloads::transaction_trace::{CollectorPayload, TransactionTrace};

#[derive(Debug, Clone)]
pub(crate) struct HarvestTraces {
    // We don't use VecDeque because the number of elements is reasonably low.
    regular: Vec<TransactionTrace>,
    // synthetics: Vec<TransactionTrace>,
}

impl HarvestTraces {
    pub(crate) fn new() -> Self {
        Self {
            regular: Vec::with_capacity(crate::limits::MAX_REGULAR_TRACES),
        }
    }

    pub(crate) fn push(&mut self, trace: TransactionTrace) {
        if self.regular.len() >= crate::limits::MAX_REGULAR_TRACES {
            self.regular.remove(0);
        }
        self.regular.push(trace);
    }

    pub(crate) fn into_payload(self, agent_run_id: &AgentRunId) -> CollectorPayload {
        let traces = self.regular;
        CollectorPayload {
            agent_run_id: agent_run_id.clone(),
            traces,
        }
    }
}
