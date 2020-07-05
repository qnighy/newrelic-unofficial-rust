use attohttpc::body::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

const DEFAULT_REPORT_PERIOD_MS: u32 = 60 * 1000;
const MAX_PAYLOAD_SIZE: usize = 1000 * 1000;
const MAX_CUSTOM_EVENTS: u32 = 10 * 1000;
const MAX_TXN_EVENTS: u32 = 10 * 1000;
const MAX_ERROR_EVENTS: u32 = 100;

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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    display_host: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Label {
    label_type: String,
    label_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UtilizationData {
    metadata_version: i32,
    logical_processors: Option<i32>,
    total_ram_mib: Option<u64>,
    hostname: String,
    // #[serde(default, skip_serializing_if = "String::is_empty")]
    // full_hostname: String,
    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    // ip_address: Vec<String>,
    // #[serde(default, skip_serializing_if = "String::is_empty")]
    // boot_id: String,
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // config: Option<ConfigOverride>,
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // vendors: Option<Vendors>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventHarvestConfig {
    #[serde(default, skip_serializing_if = "u32_is_zero")]
    report_period_ms: u32,
    harvest_limits: HarvestLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarvestLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    analytic_event_data: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    custom_event_data: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_event_data: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span_event_data: Option<u32>,
}

fn u32_is_zero(x: &u32) -> bool {
    *x == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreconnectRequest {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    security_policies_token: String,
    high_security: bool,
}

pub(crate) fn connect_attempt(name: &str, license: &str) -> anyhow::Result<()> {
    let resp: PreconnectReply = collector_request_internal(Request {
        method: "preconnect",
        // TODO: config.host
        // TODO: preconnectRegionLicenseRegex (collector.xxxx.nr-data.net)
        host: "collector.newrelic.com",
        run_id: None,
        max_payload_size: MAX_PAYLOAD_SIZE,
        license: license,
        data: &vec![PreconnectRequest {
            security_policies_token: "".to_owned(),
            high_security: false,
        }],
    })?;
    eprintln!("resp = {:?}", resp);

    let resp: serde_json::Value = collector_request_internal(Request {
        method: "connect",
        host: &resp.redirect_host,
        run_id: None,
        max_payload_size: MAX_PAYLOAD_SIZE,
        license: license,
        data: &vec![ConnectRequest {
            pid: std::process::id(),
            language: "c".to_owned(),             // TODO
            agent_version: "0.1.0".to_owned(),    // TODO
            host: "localhost".to_owned(),         // TODO
            display_host: "localhost".to_owned(), // TODO
            settings: Settings {
                app_name: name.to_owned(),
            },
            app_name: name.split(";").map(|s| s.to_owned()).collect(),
            high_security: false,
            labels: vec![],
            environment: vec![],
            identifier: name.to_owned(),
            utilization: UtilizationData {
                metadata_version: 5,
                logical_processors: None,
                total_ram_mib: None,
                hostname: "localhost".to_owned(),
            },
            metadata: HashMap::new(),
            event_harvest_config: EventHarvestConfig {
                report_period_ms: DEFAULT_REPORT_PERIOD_MS,
                harvest_limits: HarvestLimits {
                    analytic_event_data: Some(MAX_TXN_EVENTS),
                    custom_event_data: Some(MAX_CUSTOM_EVENTS),
                    error_event_data: Some(MAX_ERROR_EVENTS),
                    span_event_data: None,
                },
            },
        }],
    })?;
    eprintln!("resp = {:?}", resp);

    Ok(())
}

#[derive(Debug)]
struct Request<'a, T> {
    method: &'a str,
    host: &'a str,
    run_id: Option<&'a str>,
    max_payload_size: usize,
    license: &'a str,
    // request_headers_map: HashMap<String, String>,
    data: &'a T,
}

fn collector_request_internal<T: Serialize, U: DeserializeOwned>(
    req: Request<'_, T>,
) -> Result<U, RpmError> {
    let compressed = {
        let mut stream = GzEncoder::new(Vec::<u8>::new(), Compression::default());
        serde_json::to_writer(
            &mut stream,
            &vec![PreconnectRequest {
                security_policies_token: "".to_owned(),
                high_security: false,
            }],
        )
        .unwrap();
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
    let resp = attohttpc::post(url)
        .param("marshal_format", "json")
        .param("protocol_version", "17")
        .param("method", req.method)
        .param("license_key", req.license)
        .header("Accept-Encoding", "identity, deflate")
        .header("Content-Type", "application/octet-stream")
        .header("User-Agent", "NewRelic-Rust-Agent-Unofficial/0.1.0")
        .header("Content-Encoding", "gzip")
        .body(Bytes(compressed))
        .send()?;

    if ![200, 202].contains(&resp.status().as_u16()) {
        return Err(RpmError::StatusError {
            status: resp.status().as_u16(),
            body: resp
                .text()
                .unwrap_or_else(|e| format!("<body failed: {}>", e)),
        });
    }

    Ok(resp.json::<ResponseContainer<U>>()?.return_value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResponseContainer<T> {
    return_value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PreconnectReply {
    redirect_host: String,
    // security_policies: SecurityPolicies,
}
