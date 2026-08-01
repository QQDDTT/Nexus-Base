use nexus_core::NexusEngine;
use server::NexusService;
use nexus_proto::nexus::v1::nexus_knowledge_service_server::NexusKnowledgeService;
use nexus_proto::nexus::v1::{CommitRequest, IngestRequest, RetrieveRequest, MemoryFragment as ProtoMemoryFragment};
use std::sync::Arc;
use tempfile::tempdir;
use tonic::Request;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};

fn sign_request<T>(req: &mut Request<T>, key: &SigningKey) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let signature = key.sign(timestamp.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());
    req.metadata_mut().insert("authorization", sig_hex.parse().unwrap());
    req.metadata_mut().insert("x-timestamp", timestamp.parse().unwrap());
}

#[tokio::test]
async fn test_nexus_service_flow() {
    let dir = tempdir().unwrap();
    let config = nexus_core::EngineConfig {
        wal_path: dir.path().join("test_server.log"),
        snapshot_path: dir.path().join("test_server.nxpack"),
    };
    let engine = Arc::new(NexusEngine::new(config).unwrap());

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    let service = NexusService::new(engine, Some(Arc::new(verifying_key)));

    // 1. Ingest
    let mut ingest_req = Request::new(IngestRequest {
        fragments: vec![ProtoMemoryFragment {
            id: "test-grpc-1".to_string(),
            file_path: "/src/main.rs".to_string(),
            content_snippet: "async fn main()".to_string(),
            similarity_score: 0.0,
        }],
    });
    sign_request(&mut ingest_req, &signing_key);
    
    let ingest_resp = service.ingest_experience(ingest_req).await.unwrap().into_inner();
    assert!(ingest_resp.success);

    // 2. Retrieve
    let retrieve_req = Request::new(RetrieveRequest {
        query_tensor: vec![0.1, 0.2],
        anchors: vec![],
        top_k: 5,
    });

    let retrieve_resp = service.retrieve_context(retrieve_req).await.unwrap().into_inner();
    assert_eq!(retrieve_resp.fragments.len(), 1);
    assert_eq!(retrieve_resp.fragments[0].id, "test-grpc-1");

    // 3. Commit Snapshot
    let mut commit_req = Request::new(CommitRequest { admin_signature: "".to_string() });
    sign_request(&mut commit_req, &signing_key);
    let commit_resp = service.commit_snapshot(commit_req).await.unwrap().into_inner();
    assert!(commit_resp.success);
}

#[tokio::test]
async fn test_nexus_service_auth_failure() {
    let dir = tempdir().unwrap();
    let wal_path = std::env::temp_dir().join(format!("test_nexus_wal_{}.log", std::process::id()));
    let snapshot_path = std::env::temp_dir().join(format!("test_nexus_snap_{}.nxpack", std::process::id()));
    let config = nexus_core::EngineConfig { wal_path, snapshot_path };
    let engine = Arc::new(NexusEngine::new(config).unwrap());

    // 生成服务端真正的公钥和私钥
    let mut csprng = OsRng;
    let server_signing_key = SigningKey::generate(&mut csprng);
    let server_verifying_key = server_signing_key.verifying_key();
    let service = NexusService::new(engine, Some(Arc::new(server_verifying_key)));

    // 伪造一个恶意的客户端私钥
    let evil_signing_key = SigningKey::generate(&mut csprng);

    // 1. 无签名的恶意 Commit 尝试
    let commit_req_no_auth = Request::new(CommitRequest { admin_signature: "".to_string() });
    let resp1 = service.commit_snapshot(commit_req_no_auth).await;
    assert!(resp1.is_err(), "Unsigned request should be rejected");
    assert_eq!(resp1.unwrap_err().code(), tonic::Code::Unauthenticated);

    // 2. 签名错误（用恶意私钥签名）的 Ingest 尝试
    let mut ingest_req_evil = Request::new(IngestRequest {
        fragments: vec![ProtoMemoryFragment {
            id: "evil-1".to_string(),
            file_path: "/evil.rs".to_string(),
            content_snippet: "panic!".to_string(),
            similarity_score: 0.0,
        }],
    });
    sign_request(&mut ingest_req_evil, &evil_signing_key);
    
    let resp2 = service.ingest_experience(ingest_req_evil).await;
    assert!(resp2.is_err(), "Request with invalid signature should be rejected");
    assert_eq!(resp2.unwrap_err().code(), tonic::Code::Unauthenticated);
}

