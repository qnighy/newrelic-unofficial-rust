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

use crate::app_run::AppRun;
use crate::config::Config;
use crate::connect_reply::{ConnectReply, EventHarvestConfig, HarvestLimits, PreconnectReply};
use crate::limits::{
    DEFAULT_REPORT_PERIOD_MS, MAX_CUSTOM_EVENTS, MAX_ERROR_EVENTS, MAX_PAYLOAD_SIZE, MAX_TXN_EVENTS,
};
use crate::utilization::UtilizationData;

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
struct Settings {
    #[serde(rename = "AppName")]
    app_name: String,
    #[serde(flatten)]
    remain: HashMap<String, serde_json::Value>,
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

pub(crate) fn collector_request<T>(run: &AppRun, command: &str, payload: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    eprintln!("payload = {}", serde_json::to_string(payload).unwrap());
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

pub(crate) fn connect_attempt(config: &Config) -> anyhow::Result<AppRun> {
    let resp_pre: PreconnectReply = collector_request_json(Request {
        method: "preconnect",
        // TODO: config.host
        // TODO: preconnectRegionLicenseRegex (collector.xxxx.nr-data.net)
        host: "collector.newrelic.com",
        run_id: None,
        max_payload_size: MAX_PAYLOAD_SIZE,
        license: &config.license,
        request_headers_map: &HashMap::new(),
        data: &vec![PreconnectRequest {
            security_policies_token: "".to_owned(),
            high_security: false,
        }],
    })?;

    let utilization = UtilizationData::gather();
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
            settings: Settings {
                app_name: config.app_name.clone(),
                remain: vec![
                    (
                        "Attributes".to_owned(),
                        serde_json::json!({
                            "Enabled": true,
                            "Exclude": null,
                            "Include": null
                        }),
                    ),
                    (
                        "BrowserMonitoring".to_owned(),
                        serde_json::json!({
                          "Attributes": {
                            "Enabled": false,
                            "Exclude": null,
                            "Include": null
                          },
                          "Enabled": true
                        }),
                    ),
                    (
                        "CrossApplicationTracer".to_owned(),
                        serde_json::json!({
                          "Enabled": true
                        }),
                    ),
                    (
                        "CustomInsightsEvents".to_owned(),
                        serde_json::json!({
                          "Enabled": true
                        }),
                    ),
                    (
                        "DatastoreTracer".to_owned(),
                        serde_json::json!({
                          "DatabaseNameReporting": {
                            "Enabled": true
                          },
                          "InstanceReporting": {
                            "Enabled": true
                          },
                          "QueryParameters": {
                            "Enabled": true
                          },
                          "SlowQuery": {
                            "Enabled": true,
                            "Threshold": 10000000
                          }
                        }),
                    ),
                    (
                        "DistributedTracer".to_owned(),
                        serde_json::json!({
                          "Enabled": false,
                          "ExcludeNewRelicHeader": false
                        }),
                    ),
                    ("Enabled".to_owned(), serde_json::json!(true)),
                    ("Error".to_owned(), serde_json::json!(null)),
                    (
                        "ErrorCollector".to_owned(),
                        serde_json::json!({
                          "Attributes": {
                            "Enabled": true,
                            "Exclude": null,
                            "Include": null
                          },
                          "CaptureEvents": true,
                          "Enabled": true,
                          "IgnoreStatusCodes": [
                            0,
                            5,
                            404
                          ],
                          "RecordPanics": false
                        }),
                    ),
                    (
                        "Heroku".to_owned(),
                        serde_json::json!({
                          "DynoNamePrefixesToShorten": [
                            "scheduler",
                            "run"
                          ],
                          "UseDynoNames": true
                        }),
                    ),
                    ("HighSecurity".to_owned(), serde_json::json!(false)),
                    ("Host".to_owned(), serde_json::json!("")),
                    ("HostDisplayName".to_owned(), serde_json::json!("")),
                    (
                        "InfiniteTracing".to_owned(),
                        serde_json::json!({
                          "SpanEvents": {
                            "QueueSize": 10000
                          },
                          "TraceObserver": {
                            "Host": "",
                            "Port": 443
                          }
                        }),
                    ),
                    ("Labels".to_owned(), serde_json::json!({})),
                    ("Logger".to_owned(), serde_json::json!(null)),
                    (
                        "RuntimeSampler".to_owned(),
                        serde_json::json!({
                          "Enabled": true
                        }),
                    ),
                    ("SecurityPoliciesToken".to_owned(), serde_json::json!("")),
                    (
                        "ServerlessMode".to_owned(),
                        serde_json::json!({
                          "AccountID": "",
                          "ApdexThreshold": 500000000,
                          "Enabled": false,
                          "PrimaryAppID": "",
                          "TrustedAccountKey": ""
                        }),
                    ),
                    (
                        "SpanEvents".to_owned(),
                        serde_json::json!({
                          "Attributes": {
                            "Enabled": true,
                            "Exclude": null,
                            "Include": null
                          },
                          "Enabled": true
                        }),
                    ),
                    (
                        "TransactionEvents".to_owned(),
                        serde_json::json!({
                          "Attributes": {
                            "Enabled": true,
                            "Exclude": null,
                            "Include": null
                          },
                          "Enabled": true,
                          "MaxSamplesStored": 10000
                        }),
                    ),
                    (
                        "TransactionTracer".to_owned(),
                        serde_json::json!({
                          "Attributes": {
                            "Enabled": true,
                            "Exclude": null,
                            "Include": null
                          },
                          "Enabled": true,
                          "Segments": {
                            "Attributes": {
                              "Enabled": true,
                              "Exclude": null,
                              "Include": null
                            },
                            "StackTraceThreshold": 500000000,
                            "Threshold": 2000000
                          },
                          "Threshold": {
                            "Duration": 500000000,
                            "IsApdexFailing": true
                          }
                        }),
                    ),
                    ("Transport".to_owned(), serde_json::json!(null)),
                    (
                        "Utilization".to_owned(),
                        serde_json::json!({
                          "BillingHostname": "",
                          "DetectAWS": true,
                          "DetectAzure": true,
                          "DetectDocker": true,
                          "DetectGCP": true,
                          "DetectKubernetes": true,
                          "DetectPCF": true,
                          "LogicalProcessors": 0,
                          "TotalRAMMIB": 0
                        }),
                    ),
                    (
                        "browser_monitoring.loader".to_owned(),
                        serde_json::json!("rum"),
                    ),
                ]
                .into_iter()
                .collect(),
            },
            app_name: config.app_name.split(";").map(|s| s.to_owned()).collect(),
            high_security: false,
            labels: vec![],
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
    // eprintln!("resp = {:#?}", resp);

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
