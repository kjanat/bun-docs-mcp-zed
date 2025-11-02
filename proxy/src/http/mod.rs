use anyhow::{Context, Result};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info, warn};

const BUN_DOCS_API: &str = "https://bun.com/docs/mcp";
const REQUEST_TIMEOUT_SECS: u64 = 5;

pub struct BunDocsClient {
    client: Client,
}

impl BunDocsClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn forward_request(&self, request: Value) -> Result<Value> {
        debug!("Forwarding request to Bun Docs API");

        // Send HTTP POST with JSON-RPC request
        let response = self
            .client
            .post(BUN_DOCS_API)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&request)
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .context("Failed to send request to Bun Docs API")?;

        let status = response.status();
        info!("Bun Docs API response status: {}", status);

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("Bun Docs API error: {} - {}", status, error_text);
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Parse SSE stream
        if content_type.contains("text/event-stream") {
            debug!("Parsing SSE stream");
            return self.parse_sse_response(response).await;
        }

        // Fallback to regular JSON
        debug!("Parsing regular JSON response");
        response
            .json()
            .await
            .context("Failed to parse JSON response")
    }

    async fn parse_sse_response(&self, response: reqwest::Response) -> Result<Value> {
        let mut event_stream = response.bytes_stream().eventsource();
        let mut json_response: Option<Value> = None;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(event) => {
                    debug!("SSE event type: {:?}", event.event);

                    let data = event.data;
                    if !data.is_empty() {
                        match serde_json::from_str::<Value>(&data) {
                            Ok(parsed) => {
                                debug!("Parsed SSE data successfully");

                                // Based on protocol analysis, the SSE data contains
                                // the complete JSON-RPC response
                                if parsed.get("result").is_some() || parsed.get("error").is_some() {
                                    json_response = Some(parsed);
                                    // Found the JSON-RPC response, we can stop
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse SSE data as JSON: {}", e);
                                debug!("SSE data: {}", &data[..data.len().min(200)]);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("SSE stream error: {}", e);
                    break;
                }
            }
        }

        json_response.ok_or_else(|| anyhow::anyhow!("No valid JSON-RPC response in SSE stream"))
    }
}
