use serde::{Deserialize, Serialize};
use serde_json::Value;
use tonic::transport::Channel;
use nexus_proto::nexus::v1::nexus_knowledge_service_client::NexusKnowledgeServiceClient;
use tracing::{debug, warn};
use crate::tools;

#[derive(Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>, // 可能是数字或字符串，通知可能没有id
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct McpServer {
    pub grpc_client: NexusKnowledgeServiceClient<Channel>,
}

impl McpServer {
    pub fn new(grpc_client: NexusKnowledgeServiceClient<Channel>) -> Self {
        Self { grpc_client }
    }

    pub async fn handle_request(&self, line: &str) -> anyhow::Result<Option<String>> {
        debug!("Received: {}", line);
        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => return Ok(None), // 非JSON或格式不对忽略
        };

        if req.jsonrpc != "2.0" {
            return Ok(None);
        }

        let id = req.id.unwrap_or(Value::Null);

        // 如果没有 id 则是通知，直接忽略，因为 MCP 2026-07-28 的调用需要响应
        if id.is_null() {
            return Ok(None);
        }

        let result = match req.method.as_str() {
            "tools/call" => {
                // MCP 无状态的 tools/call
                tools::handle_tools_call(self.grpc_client.clone(), req.params).await
            }
            _ => {
                warn!("Unsupported method: {}", req.method);
                Err(serde_json::json!({
                    "code": -32601,
                    "message": "Method not found"
                }))
            }
        };

        let resp = match result {
            Ok(res) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(res),
                error: None,
            },
            Err(err) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(err),
            },
        };

        Ok(Some(serde_json::to_string(&resp)?))
    }
}
