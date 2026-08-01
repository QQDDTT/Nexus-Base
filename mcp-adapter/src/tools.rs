use serde_json::Value;
use tonic::transport::Channel;
use nexus_proto::nexus::v1::{nexus_knowledge_service_client::NexusKnowledgeServiceClient, RetrieveRequest, MemoryFragment, IngestRequest};
use tracing::{info, warn};
use serde::Deserialize;

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

async fn fetch_embedding(query: &str) -> Result<Vec<f32>, String> {
    let api_url = std::env::var("OLLAMA_API_URL").unwrap_or_else(|_| "http://localhost:11434/api/embeddings".to_string());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());

    let client = reqwest::Client::new();
    let res = client.post(&api_url)
        .json(&serde_json::json!({
            "model": model,
            "prompt": query
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Ollama API returned status: {}", res.status()));
    }

    let parsed: OllamaEmbeddingResponse = res.json().await.map_err(|e| format!("Parse error: {}", e))?;
    Ok(parsed.embedding)
}

pub async fn handle_tools_call(
    mut client: NexusKnowledgeServiceClient<Channel>,
    params: Option<Value>,
) -> Result<Value, Value> {
    let params = params.unwrap_or(Value::Null);

    // MCP 2026-07-28 规范：_meta 包含了协议版本与客户端声明，无状态架构核心
    if let Some(_meta) = params.get("_meta") {
        info!("Processing stateless request with _meta: {:?}", _meta);
    }

    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| serde_json::json!({"code": -32602, "message": "Missing 'name' in params"}))?;

    let arguments = params.get("arguments").unwrap_or(&Value::Null);

    match name {
        "search_knowledge_context" => {
            let intent_query = arguments.get("intent_query").and_then(|q| q.as_str()).unwrap_or("");
            let top_k = arguments.get("top_k").and_then(|k| k.as_u64()).unwrap_or(5) as u32;
            let anchors: Vec<String> = arguments
                .get("anchors")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            // 调用远程 Embedding 服务
            let query_tensor = match fetch_embedding(intent_query).await {
                Ok(tensor) => tensor,
                Err(e) => {
                    warn!("Failed to fetch embedding for intent: {}. Error: {}", intent_query, e);
                    return Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Embedding API Error: {}", e)
                            }
                        ],
                        "isError": true
                    }));
                }
            };

            let req = RetrieveRequest {
                query_tensor,
                anchors,
                top_k,
            };

            match client.retrieve_context(req).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    
                    let mut text_output = String::new();
                    for frag in &resp.fragments {
                        text_output.push_str(&format!("[{}] {}\n", frag.file_path, frag.content_snippet));
                    }

                    // 遵循 MCP 2026-06-18 及之后的 structuredContent 返回格式
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": text_output
                            }
                        ],
                        "structuredContent": {
                            "fragments": resp.fragments.iter().map(|f| {
                                serde_json::json!({
                                    "id": f.id,
                                    "file_path": f.file_path,
                                    "similarity_score": f.similarity_score
                                })
                            }).collect::<Vec<_>>(),
                            "compute_latency_ms": resp.compute_latency_ms
                        },
                        "isError": false
                    }))
                }
                Err(e) => {
                    warn!("gRPC error: {:?}", e);
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Backend Error: {}", e.message())
                            }
                        ],
                        "isError": true
                    }))
                }
            }
        }
        "ingest_knowledge_experience" => {
            let frags = arguments.get("fragments").and_then(|f| f.as_array()).ok_or_else(|| {
                serde_json::json!({"code": -32602, "message": "Missing 'fragments' array"})
            })?;

            let mut grpc_frags = Vec::new();
            for f in frags {
                let file_path = f.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let snippet = f.get("content_snippet").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let embedding = fetch_embedding(&snippet).await.unwrap_or_default();
                grpc_frags.push(MemoryFragment {
                    id: uuid::Uuid::new_v4().to_string(),
                    file_path,
                    content_snippet: snippet,
                    embedding,
                    metadata_anchors: vec!["mcp".to_string()],
                    similarity_score: 1.0,
                });
            }

            let req = IngestRequest { fragments: grpc_frags };

            match client.ingest_experience(req).await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "content": [{"type": "text", "text": "Successfully ingested experience."}],
                        "isError": false
                    }))
                }
                Err(e) => {
                    Ok(serde_json::json!({
                        "content": [{"type": "text", "text": format!("Ingest Error: {}", e.message())}],
                        "isError": true
                    }))
                }
            }
        }
        _ => Err(serde_json::json!({
            "code": -32601,
            "message": format!("Tool {} not found", name)
        })),
    }
}
