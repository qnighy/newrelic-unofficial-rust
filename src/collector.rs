// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use attohttpc::body::Bytes;
use attohttpc::header::HeaderName;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use self::settings::Settings;
use crate::app_run::AppRun;
use crate::config::Config;
use crate::connect_reply::{ConnectReply, EventHarvestConfig, HarvestLimits, PreconnectReply};
use crate::limits::{
    DEFAULT_REPORT_PERIOD_MS, MAX_CUSTOM_EVENTS, MAX_ERROR_EVENTS, MAX_PAYLOAD_SIZE, MAX_TXN_EVENTS,
};
use crate::utilization::UtilizationData;

mod settings;

#[derive(Error, Debug)]
pub(crate) enum RpmError {
    #[error("HTTP Error: {0}")]
    HttpError(#[from] attohttpc::Error),
    #[error(
        "Payload size for {method} too large: {compressed_len} greater than {max_payload_size}"
    )]
    PayloadTooLarge {
        method: String,
        compressed_len: usize,
        max_payload_size: usize,
    },
    #[error("response code: {status}: {body}")]
    StatusError { status: u16, body: String },
    #[error("shutdown")]
    Shutdown(#[from] crate::sync_util::ShutdownError),
}

impl RpmError {
    pub(crate) fn is_disconnect(&self) -> bool {
        if let RpmError::StatusError { status, .. } = self {
            *status == 410
        } else if let RpmError::Shutdown(..) = self {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_restart_exception(&self) -> bool {
        if let RpmError::StatusError { status, .. } = self {
            *status == 401 || *status == 409
        } else {
            false
        }
    }

    // pub(crate) fn should_save_harvest_data(&self) -> bool {
    //     if let RpmError::StatusError { status, .. } = self {
    //         *status == 408 || *status == 429 || *status == 500 || *status == 503
    //     } else {
    //         false
    //     }
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectRequest {
    pid: u32,
    language: String,
    agent_version: String,
    host: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display_host: Option<String>,
    settings: Settings,
    app_name: Vec<String>,
    high_security: bool,
    labels: Vec<Label>,
    environment: Vec<(String, serde_json::Value)>,
    identifier: String,
    utilization: UtilizationData,
    // #[serde(default, skip_serializing_if="Option::is_none")]
    // security_policies: Option<SecurityPolicies>,
    metadata: HashMap<String, String>,
    event_harvest_config: EventHarvestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Label {
    label_type: String,
    label_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreconnectRequest {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    security_policies_token: String,
    high_security: bool,
}

pub(crate) fn collector_request<T>(run: &AppRun, command: &str, payload: &T) -> Result<(), RpmError>
where
    T: Serialize,
{
    log::debug!("payload = {}", serde_json::to_string(payload).unwrap());
    collector_request_internal(Request {
        method: command,
        host: &run.host,
        run_id: Some(&run.agent_run_id.0),
        max_payload_size: MAX_PAYLOAD_SIZE,
        license: &run.license,
        request_headers_map: &run.request_headers_map,
        data: payload,
    })?;
    Ok(())
}

pub(crate) fn connect_attempt(config: &Config) -> Result<AppRun, RpmError> {
    let resp_pre: PreconnectReply = collector_request_json(Request {
        method: "preconnect",
        host: &preconnect_host(config),
        run_id: None,
        max_payload_size: MAX_PAYLOAD_SIZE,
        license: &config.license,
        request_headers_map: &HashMap::new(),
        data: &vec![PreconnectRequest {
            security_policies_token: "".to_owned(),
            high_security: false,
        }],
    })?;

    let utilization = UtilizationData::gather(config);
    let resp: ConnectReply = collector_request_json(Request {
        method: "connect",
        host: &resp_pre.redirect_host,
        run_id: None,
        max_payload_size: MAX_PAYLOAD_SIZE,
        license: &config.license,
        request_headers_map: &HashMap::new(),
        data: &vec![ConnectRequest {
            pid: std::process::id(),
            // TODO
            language: "go".to_owned(),
            // TODO
            agent_version: "3.8.0".to_owned(),
            host: utilization.hostname().to_owned(),
            display_host: config.host_display_name.clone(),
            settings: Settings::new(config),
            app_name: config.app_name.split(';').map(|s| s.to_owned()).collect(),
            high_security: false,
            labels: config
                .labels
                .iter()
                .map(|(k, v)| Label {
                    label_type: k.clone(),
                    label_value: v.clone(),
                })
                .collect(),
            environment: vec![
                // TODO
                ("runtime.Compiler".to_owned(), "gc".to_owned().into()),
                // TODO
                ("runtime.GOARCH".to_owned(), "amd64".to_owned().into()),
                // TODO
                ("runtime.GOOS".to_owned(), "linux".to_owned().into()),
                // TODO
                ("runtime.Version".to_owned(), "go1.14.2".to_owned().into()),
                // TODO
                ("runtime.NumCPU".to_owned(), 4.into()),
            ],
            identifier: config.app_name.clone(),
            utilization,
            metadata: HashMap::new(),
            event_harvest_config: EventHarvestConfig {
                report_period_ms: Some(DEFAULT_REPORT_PERIOD_MS),
                harvest_limits: HarvestLimits {
                    analytic_event_data: Some(MAX_TXN_EVENTS),
                    custom_event_data: Some(MAX_CUSTOM_EVENTS),
                    error_event_data: Some(MAX_ERROR_EVENTS),
                    span_event_data: None,
                },
            },
        }],
    })?;
    log::debug!("resp = {:#?}", resp);

    Ok(AppRun::new(&config.license, &resp_pre, &resp))
}

#[derive(Debug)]
struct Request<'a, T> {
    method: &'a str,
    host: &'a str,
    run_id: Option<&'a str>,
    max_payload_size: usize,
    license: &'a str,
    request_headers_map: &'a HashMap<String, String>,
    data: &'a T,
}

fn collector_request_json<T: Serialize, U: DeserializeOwned>(
    req: Request<'_, T>,
) -> Result<U, RpmError> {
    let resp = collector_request_internal(req)?;

    Ok(resp.json::<ResponseContainer<U>>()?.return_value)
}

fn collector_request_internal<T: Serialize>(
    req: Request<'_, T>,
) -> Result<attohttpc::Response, RpmError> {
    let compressed = {
        let mut stream = GzEncoder::new(Vec::<u8>::new(), Compression::default());
        serde_json::to_writer(&mut stream, req.data).unwrap();
        stream.finish().unwrap()
    };
    if compressed.len() > req.max_payload_size {
        return Err(RpmError::PayloadTooLarge {
            method: req.method.to_owned(),
            compressed_len: compressed.len(),
            max_payload_size: req.max_payload_size,
        });
    }

    let url = format!("https://{}/agent_listener/invoke_raw_method", req.host);
    let mut collector_req = attohttpc::post(url)
        .param("license_key", req.license)
        .param("marshal_format", "json")
        .param("method", req.method)
        .param("protocol_version", "17")
        .header("Content-Type", "application/octet-stream")
        .header("User-Agent", "NewRelic-Rust-Agent-Unofficial/0.1.0")
        .header("Content-Encoding", "gzip");
    if let Some(run_id) = req.run_id {
        collector_req = collector_req.param("run_id", run_id);
    }
    for (header, value) in req.request_headers_map {
        let header = header.parse::<HeaderName>().unwrap();
        collector_req = collector_req.header(header, value);
    }
    let resp = collector_req.body(Bytes(compressed)).send()?;

    if ![200, 202].contains(&resp.status().as_u16()) {
        return Err(RpmError::StatusError {
            status: resp.status().as_u16(),
            body: resp
                .text()
                .unwrap_or_else(|e| format!("<body failed: {}>", e)),
        });
    }

    Ok(resp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResponseContainer<T> {
    return_value: T,
}

fn preconnect_host(config: &Config) -> String {
    if let Some(host) = &config.host {
        return host.clone();
    }
    if let Some(pos) = config.license.find('x') {
        return format!("collector.{}.nr-data.net", &config.license[..pos]);
    }
    "collector.newrelic.com".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preconnect_host_base_case() {
        #[derive(Debug)]
        struct TestCase {
            license: &'static str,
            override_: Option<&'static str>,
            expect: &'static str,
        }
        const TEST_CASES: &[TestCase] = &[
            // non-region license
            TestCase {
                license: "0123456789012345678901234567890123456789",
                override_: None,
                expect: "collector.newrelic.com",
            },
            // override present
            TestCase {
                license: "0123456789012345678901234567890123456789",
                override_: Some("other-collector.newrelic.com"),
                expect: "other-collector.newrelic.com",
            },
            // four letter region
            TestCase {
                license: "eu01xx6789012345678901234567890123456789",
                override_: None,
                expect: "collector.eu01.nr-data.net",
            },
            // five letter region
            TestCase {
                license: "gov01x6789012345678901234567890123456789",
                override_: None,
                expect: "collector.gov01.nr-data.net",
            },
            // six letter region
            TestCase {
                license: "foo001x789012345678901234567890123456789",
                override_: None,
                expect: "collector.foo001.nr-data.net",
            },
        ];
        for test_case in TEST_CASES {
            let mut config = Config::new("test", test_case.license);
            config.host = test_case.override_.map(|s| s.to_owned());
            config.validate().unwrap();
            assert_eq!(preconnect_host(&config), test_case.expect);
        }
    }
}
