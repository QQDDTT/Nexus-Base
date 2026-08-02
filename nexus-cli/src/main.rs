use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use nexus_proto::nexus::v1::nexus_knowledge_service_client::NexusKnowledgeServiceClient;
use nexus_proto::nexus::v1::{CommitRequest, IngestRequest, MemoryFragment, RetrieveRequest};
use tonic::Request;

#[derive(Parser)]
#[command(name = "nexus-cli", about = "Nexus-Base Management CLI")]
struct Cli {
    #[arg(short, long, default_value = "http://[::1]:9090")]
    endpoint: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 在本地生成新的 Ed25519 管理密钥对
    Keygen {
        #[arg(short, long, default_value = ".")]
        out_dir: PathBuf,
    },
    /// 摄入新的知识片段
    Ingest {
        #[arg(short, long)]
        text: String,
        #[arg(short, long, default_value = "./private.key")]
        key: PathBuf,
    },
    /// 检索上下文
    Retrieve {
        #[arg(short, long)]
        query: String,
        #[arg(short = 'k', long, default_value_t = 3)]
        top_k: u32,
    },
    /// 强制生成物理快照
    Commit {
        #[arg(short, long, default_value = "./private.key")]
        key: PathBuf,
    },
}

/// 辅助函数：根据私钥文件和时间戳生成鉴权 Metadata，并附加到 gRPC Request
fn sign_request<T>(req: &mut Request<T>, key_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let hex_key = fs::read_to_string(key_path)?;
    let bytes = hex::decode(hex_key.trim())?;
    let signing_key = SigningKey::from_bytes(bytes.as_slice().try_into()?);

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string();
    
    // 签名
    let signature = signing_key.sign(timestamp.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());

    req.metadata_mut().insert("authorization", sig_hex.parse()?);
    req.metadata_mut().insert("x-timestamp", timestamp.parse()?);
    Ok(())
}

/// 从本地 Ollama 获取真实的 Embedding
async fn fetch_embedding(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/embeddings")
        .json(&serde_json::json!({
            "model": "nomic-embed-text",
            "prompt": text
        }))
        .send()
        .await?;
    
    let json: serde_json::Value = res.json().await?;
    let embedding = json["embedding"]
        .as_array()
        .ok_or("Failed to parse embedding array")?
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();
        
    Ok(embedding)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Keygen { out_dir } => {
            let mut csprng = OsRng;
            let signing_key: SigningKey = SigningKey::generate(&mut csprng);
            let verifying_key = signing_key.verifying_key();
            
            let priv_hex = hex::encode(signing_key.to_bytes());
            let pub_hex = hex::encode(verifying_key.to_bytes());
            
            fs::write(out_dir.join("private.key"), priv_hex)?;
            fs::write(out_dir.join("public.key"), pub_hex)?;
            println!("Keys generated in {:?}", out_dir);
            return Ok(());
        }
        _ => {}
    }

    let mut client = NexusKnowledgeServiceClient::connect(cli.endpoint.clone()).await?;

    match &cli.command {
        Commands::Ingest { text, key } => {
            println!("Fetching embedding for input text...");
            let embedding = fetch_embedding(text).await?;
            let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string();
            let mut req = Request::new(IngestRequest {
                fragments: vec![MemoryFragment {
                    id: format!("cli-doc-{}", ts),
                    file_path: "cli-input".to_string(),
                    content_snippet: text.clone(),
                    embedding,
                    metadata_anchors: vec!["cli".to_string()],
                    similarity_score: 0.0,
                }],
            });
            sign_request(&mut req, key)?;
            let response = client.ingest_experience(req).await?;
            println!("Ingest Response: {:?}", response.into_inner());
        }
        Commands::Retrieve { query, top_k } => {
            println!("Fetching embedding for query...");
            let query_tensor = fetch_embedding(query).await?;
            let req = Request::new(RetrieveRequest {
                query_tensor,
                anchors: vec![],
                top_k: *top_k,
            });
            let response = client.retrieve_context(req).await?;
            let resp_inner = response.into_inner();
            println!("Retrieved {} fragments:", resp_inner.fragments.len());
            for f in resp_inner.fragments {
                println!("- [{}]: {}", f.id, f.content_snippet);
            }
        }
        Commands::Commit { key } => {
            let mut req = Request::new(CommitRequest { admin_signature: "".to_string() });
            sign_request(&mut req, key)?;
            let response = client.commit_snapshot(req).await?;
            println!("Commit Response: {:?}", response.into_inner());
        }
        _ => unreachable!(),
    }

    Ok(())
}
