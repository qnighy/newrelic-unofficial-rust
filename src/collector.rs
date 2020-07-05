use attohttpc::body::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

const MAX_PAYLOAD_SIZE: usize = 1000 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreconnectRequest {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    security_policies_token: String,
    high_security: bool,
}

pub(crate) fn connect_attempt(license: &str) -> anyhow::Result<()> {
    let resp: PreconnectReply = collector_request_internal(Request {
        method: "preconnect",
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
) -> anyhow::Result<U> {
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
    anyhow::ensure!(
        compressed.len() < req.max_payload_size,
        "Payload size for {} too large: {} greater than {}",
        req.method,
        compressed.len(),
        req.max_payload_size
    );

    let url = format!("https://{}/agent_listener/invoke_raw_method", req.host);
    let resp = attohttpc::post(url)
        .param("marshal_format", "json")
        .param("protocol_version", "17")
        .param("method", "preconnect")
        .param("license_key", req.license)
        .header("Accept-Encoding", "identity, deflate")
        .header("Content-Type", "application/octet-stream")
        .header("User-Agent", "NewRelic-Rust-Agent-Unofficial/0.1.0")
        .header("Content-Encoding", "gzip")
        .body(Bytes(compressed))
        .send()?;

    anyhow::ensure!(
        [200, 202].contains(&resp.status().as_u16()),
        "response code: {}",
        resp.status().as_u16(),
    );

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
