use nexus_core::{NexusEngine, TensorEngine, MemoryFragment};
use nexus_core::math::cosine_similarity;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_cosine_similarity() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

    let c = vec![0.0, 1.0, 0.0];
    assert!(cosine_similarity(&a, &c).abs() < 1e-6);

    let d = vec![0.5, 0.5, 0.0];
    // 相似度为 1*0.5 / (1 * sqrt(0.5^2+0.5^2)) = 0.5 / sqrt(0.5) = sqrt(0.5) = 0.707106
    assert!((cosine_similarity(&a, &d) - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
}

#[test]
fn test_engine_ingest_flow() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("test_wal.log");
    let snapshot_path = dir.path().join("test_snap.nxpack");
    let config = nexus_core::EngineConfig { wal_path: wal_path.clone(), snapshot_path };
    let engine = NexusEngine::new(config).expect("Failed to init engine");
    
    let frag = MemoryFragment {
        id: "test-id-1".to_string(),
        file_path: "/src/main.rs".to_string(),
        content_snippet: "fn main() {}".to_string(),
        embedding: vec![1.0, 0.0, 0.0], // 用真实的张量
        metadata_anchors: vec!["rust".to_string()],
        similarity_score: 0.0,
    };
    
    let frag2 = MemoryFragment {
        id: "test-id-2".to_string(),
        file_path: "/src/main.rs".to_string(),
        content_snippet: "fn extra() {}".to_string(),
        embedding: vec![0.5, 0.5, 0.0],
        metadata_anchors: vec!["rust".to_string()],
        similarity_score: 0.0,
    };

    // 摄入
    engine.ingest(vec![frag.clone(), frag2.clone()]).expect("Failed to ingest");
    
    // 验证内存脏集合的精确检索
    let query = vec![1.0, 0.0, 0.0];
    let results = engine.retrieve(&query, &[], 1).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-id-1"); // 最匹配的是 frag1
    assert!(results[0].similarity_score > 0.99); // 应该为 1.0

    // 验证物理 WAL 文件生成
    let content = fs::read_to_string(&wal_path).expect("Failed to read wal file");
    assert!(content.contains("test-id-1"));
    
    // 测试快照提交 (Commit Snapshot)
    engine.commit_snapshot().expect("Failed to commit");
    
    // WAL 应该被清空
    let content_after = fs::read_to_string(&wal_path).unwrap();
    assert!(content_after.is_empty());
    
    // 脏集合数据被移入快照集合，依然可以检索
    let post_commit_results = engine.retrieve(&query, &[], 10).unwrap();
    assert_eq!(post_commit_results.len(), 2);
}

#[test]
fn test_engine_recovery() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("recovery_wal.log");
    let snapshot_path = dir.path().join("recovery_snap.nxpack");
    
    // 1. 初始化并写入数据
    {
        let config = nexus_core::EngineConfig { wal_path: wal_path.clone(), snapshot_path: snapshot_path.clone() };
        let engine = NexusEngine::new(config).unwrap();
        
        let frag = MemoryFragment {
            id: "recovery-id-1".to_string(),
            file_path: "/test.rs".to_string(),
            content_snippet: "test_recovery".to_string(),
            embedding: vec![0.8, 0.6, 0.0],
            metadata_anchors: vec![],
            similarity_score: 0.0,
        };
        engine.ingest(vec![frag]).unwrap();
        
        // 生成快照
        engine.commit_snapshot().unwrap();
    } // 引擎在此销毁
    
    // 2. 利用相同的配置重新创建引擎
    {
        let config = nexus_core::EngineConfig { wal_path, snapshot_path };
        let engine = NexusEngine::new(config).unwrap();
        
        // 应该能从快照中恢复出数据
        let query = vec![0.8, 0.6, 0.0];
        let results = engine.retrieve(&query, &[], 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "recovery-id-1");
    }
}
