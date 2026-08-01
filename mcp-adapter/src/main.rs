
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use mcp_adapter::mcp_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr) // MCP requires stdout for JSON-RPC, logs MUST go to stderr
        .init();

    info!("Starting Nexus MCP Adapter on stdio...");

    let grpc_addr = std::env::var("NEXUS_GRPC_ADDR").unwrap_or_else(|_| "http://[::1]:9090".to_string());
    
    // 连接到 gRPC 引擎
    let grpc_client = match nexus_proto::nexus::v1::nexus_knowledge_service_client::NexusKnowledgeServiceClient::connect(grpc_addr.clone()).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to Nexus gRPC backend at {}: {}", grpc_addr, e);
            std::process::exit(1);
        }
    };
    info!("Connected to Nexus gRPC backend at {}", grpc_addr);

    let server = mcp_server::McpServer::new(grpc_client);

    // 标准输入处理 JSON-RPC 消息
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        match server.handle_request(&line).await {
            Ok(response_json) => {
                if let Some(resp) = response_json {
                    println!("{}", resp); // 必须写到 stdout
                }
            }
            Err(e) => {
                error!("Error handling request: {:?}", e);
            }
        }
    }

    Ok(())
}
