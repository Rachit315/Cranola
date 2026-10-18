//! Ollama (local models) discovery.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use sw_core::AppError;
use ts_rs::TS;

const DETECT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OllamaModel {
    pub name: String,
    pub size_bytes: u64,
    /// e.g. "8.0B"; empty when Ollama does not report it.
    pub parameter_size: String,
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OllamaStatus {
    pub base_url: String,
    pub version: String,
    pub models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct VersionResponse {
    version: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    details: TagDetails,
}

#[derive(Deserialize, Default)]
struct TagDetails {
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    family: String,
}

fn unreachable(base_url: &str, detail: impl std::fmt::Display) -> AppError {
    AppError::Network {
        service: format!("Ollama at {base_url}"),
        detail: detail.to_string(),
    }
}

/// Checks whether an Ollama server is running and lists its installed models.
pub async fn detect(base_url: &str) -> Result<OllamaStatus, AppError> {
    let base = base_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(DETECT_TIMEOUT)
        .build()
        .map_err(|e| AppError::internal(format!("http client: {e}")))?;

    let version: VersionResponse = client
        .get(format!("{base}/api/version"))
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| unreachable(base, e))?
        .json()
        .await
        .map_err(|e| unreachable(base, format!("unexpected response: {e}")))?;

    let tags: TagsResponse = client
        .get(format!("{base}/api/tags"))
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| unreachable(base, e))?
        .json()
        .await
        .map_err(|e| unreachable(base, format!("unexpected response: {e}")))?;

    let mut models: Vec<OllamaModel> = tags
        .models
        .into_iter()
        .map(|m| OllamaModel {
            name: m.name,
            size_bytes: m.size,
            parameter_size: m.details.parameter_size,
            family: m.details.family,
        })
        .collect();
    models.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(OllamaStatus {
        base_url: base.to_owned(),
        version: version.version,
        models,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn lists_models_from_a_running_server() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"version": "0.40.3"})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "models": [
                    {"name": "qwen3:8b", "size": 5_200_000_000_u64,
                     "details": {"parameter_size": "8.2B", "family": "qwen3"}},
                    {"name": "llama3.2:3b", "size": 2_000_000_000_u64,
                     "details": {"parameter_size": "3.2B", "family": "llama"}}
                ]
            })))
            .mount(&server)
            .await;

        let status = detect(&format!("{}/", server.uri())).await.unwrap();
        assert_eq!(status.version, "0.40.3");
        assert_eq!(status.base_url, server.uri());
        let names: Vec<_> = status.models.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["llama3.2:3b", "qwen3:8b"]);
        assert_eq!(status.models[1].parameter_size, "8.2B");
    }

    #[tokio::test]
    async fn tolerates_missing_details() {
        let server = MockServer::start().await;
        Mock::given(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"version": "1"})))
            .mount(&server)
            .await;
        Mock::given(path("/api/tags"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"models": [{"name": "m"}]})),
            )
            .mount(&server)
            .await;
        let status = detect(&server.uri()).await.unwrap();
        assert_eq!(status.models[0].size_bytes, 0);
        assert_eq!(status.models[0].family, "");
    }

    #[tokio::test]
    async fn reports_unreachable_server() {
        // Port 9 (discard) on localhost is essentially never an HTTP server.
        let err = detect("http://127.0.0.1:9").await.unwrap_err();
        assert_eq!(err.code(), "net.unreachable");
        assert!(err.retryable());
    }

    #[tokio::test]
    async fn reports_http_errors() {
        let server = MockServer::start().await;
        Mock::given(path("/api/version"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let err = detect(&server.uri()).await.unwrap_err();
        assert_eq!(err.code(), "net.unreachable");
        assert!(err.detail().unwrap().contains("500"));
    }
}
