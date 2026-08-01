use std::sync::Arc;
use tonic::{Request, Response, Status};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

use nexus_core::{MemoryFragment, NexusEngine, TensorEngine};
use nexus_proto::nexus::v1::nexus_knowledge_service_server::NexusKnowledgeService;
use nexus_proto::nexus::v1::{
    CommitRequest, CommitResponse, IngestRequest, IngestResponse,
    RetrieveRequest, RetrieveResponse, MemoryFragment as ProtoMemoryFragment,
    RollbackRequest, RollbackResponse
};

pub struct NexusService {
    engine: Arc<NexusEngine>,
    public_key: Option<Arc<VerifyingKey>>,
}

impl NexusService {
    pub fn new(engine: Arc<NexusEngine>, public_key: Option<Arc<VerifyingKey>>) -> Self {
        Self { engine, public_key }
    }

    fn verify_auth<T>(&self, request: &Request<T>) -> Result<(), Status> {
        if let Some(pub_key) = &self.public_key {
            let auth_header = request.metadata().get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;
            
            let timestamp = request.metadata().get("x-timestamp")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| Status::unauthenticated("Missing x-timestamp header"))?;

            let message = timestamp.as_bytes();
            let sig_bytes = hex::decode(auth_header).map_err(|_| Status::unauthenticated("Invalid hex signature"))?;
            if sig_bytes.len() != 64 {
                return Err(Status::unauthenticated("Invalid signature length"));
            }
            let signature = Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap());
            pub_key.verify(message, &signature).map_err(|_| Status::unauthenticated("Signature verification failed"))?;
            Ok(())
        } else {
            Err(Status::unauthenticated("Server not configured for authentication"))
        }
    }
}

#[tonic::async_trait]
impl NexusKnowledgeService for NexusService {
    async fn ingest_experience(
        &self,
        request: Request<IngestRequest>,
    ) -> Result<Response<IngestResponse>, Status> {
        self.verify_auth(&request)?;
        let req = request.into_inner();
        
        // 映射 Proto 对象到底层实体
        let fragments: Vec<MemoryFragment> = req.fragments.into_iter().map(|f| {
            MemoryFragment {
                id: f.id,
                file_path: f.file_path,
                content_snippet: f.content_snippet,
                embedding: f.embedding,
                metadata_anchors: f.metadata_anchors,
                similarity_score: f.similarity_score,
            }
        }).collect();

        self.engine.ingest(fragments)
            .map_err(|e| Status::internal(format!("Engine error: {:?}", e)))?;

        Ok(Response::new(IngestResponse { success: true, message: "Ingested".to_string() }))
    }

    async fn retrieve_context(
        &self,
        request: Request<RetrieveRequest>,
    ) -> Result<Response<RetrieveResponse>, Status> {
        let req = request.into_inner();
        
        let results = self.engine.retrieve(
            &req.query_tensor,
            &req.anchors,
            req.top_k as usize,
        ).map_err(|e| Status::internal(format!("Engine error: {:?}", e)))?;

        // 映射结果为 Proto 对象
        let proto_fragments = results.into_iter().map(|f| {
            ProtoMemoryFragment {
                id: f.id,
                file_path: f.file_path,
                content_snippet: f.content_snippet,
                similarity_score: f.similarity_score,
                embedding: f.embedding,
                metadata_anchors: f.metadata_anchors,
            }
        }).collect();

        Ok(Response::new(RetrieveResponse {
            fragments: proto_fragments,
            compute_latency_ms: 0.0,
        }))
    }

    async fn commit_snapshot(
        &self,
        request: Request<CommitRequest>,
    ) -> Result<Response<CommitResponse>, Status> {
        self.verify_auth(&request)?;
        
        self.engine.commit_snapshot()
            .map_err(|e| Status::internal(format!("Engine error: {:?}", e)))?;

        Ok(Response::new(CommitResponse { success: true, message: "OK".to_string() }))
    }

    async fn rollback_state(
        &self,
        _request: Request<RollbackRequest>,
    ) -> Result<Response<RollbackResponse>, Status> {
        // 尚未实现回滚逻辑
        Ok(Response::new(RollbackResponse { success: false, message: "Not implemented".to_string() }))
    }
}
