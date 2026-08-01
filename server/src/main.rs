use std::sync::Arc;
use tonic::transport::Server;
use nexus_core::{NexusEngine, EngineConfig};
use nexus_proto::nexus::v1::nexus_knowledge_service_server::NexusKnowledgeServiceServer;

use server::NexusService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:9090".parse()?;
    
    // 初始化底层引擎
    let config = EngineConfig {
        wal_path: std::env::temp_dir().join("nexus_main.log"),
        snapshot_path: std::env::temp_dir().join("nexus_snapshot.nxpack"),
    };
    let engine = NexusEngine::new(config)?;
    let engine_arc = Arc::new(engine);

    // 尝试读取 public.key (如果存在)
    let pub_key_path = std::env::current_dir()?.join("public.key");
    let public_key = if pub_key_path.exists() {
        let hex_str = std::fs::read_to_string(pub_key_path)?;
        let bytes = hex::decode(hex_str.trim())?;
        let pk = ed25519_dalek::VerifyingKey::from_bytes(bytes.as_slice().try_into().unwrap())?;
        tracing::info!("Loaded admin public key for authentication.");
        Some(Arc::new(pk))
    } else {
        tracing::warn!("No public.key found in current directory. Writes will be rejected.");
        None
    };

    // 实例化 gRPC 处理器
    let nexus_service = NexusService::new(engine_arc, public_key);

    tracing::info!("Nexus Server listening on {}", addr);

    // 装配并启动
    Server::builder()
        .add_service(NexusKnowledgeServiceServer::new(nexus_service))
        .serve(addr)
        .await?;

    Ok(())
}
