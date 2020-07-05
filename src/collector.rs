use attohttpc::body::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreconnectRequest {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    security_policies_token: String,
    high_security: bool,
}

pub(crate) fn connect_attempt(license: &str) -> anyhow::Result<()> {
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
    eprintln!("compressed = {:?}", compressed);
    let resp = attohttpc::post("https://collector.newrelic.com/agent_listener/invoke_raw_method")
        .param("marshal_format", "json")
        .param("protocol_version", "17")
        .param("method", "preconnect")
        .param("license_key", license)
        .header("Accept-Encoding", "identity, deflate")
        .header("Content-Type", "application/octet-stream")
        .header("User-Agent", "NewRelic-Rust-Agent-Unofficial/0.1.0")
        .header("Content-Encoding", "gzip")
        .body(Bytes(compressed))
        .send()?;

    eprintln!("resp = {:#?}", resp);
    let body = resp.text()?;
    eprintln!("body = {:?}", body);

    Ok(())
}
