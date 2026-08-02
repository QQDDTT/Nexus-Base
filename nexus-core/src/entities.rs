//! 核心领域实体与错误定义

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 引擎级错误枚举
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Corrupted snapshot file: {0}")]
    CorruptedSnapshot(String),
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("Failed to acquire exclusive file lock")]
    LockAcquisitionFailed,
    #[error("I/O error: {0}")]
    IoFailure(#[from] std::io::Error),
}

/// 张量图节点的数据承载：记忆碎片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    pub id: String,
    pub file_path: String,
    pub content_snippet: String,
    pub embedding: Vec<f32>,
    pub metadata_anchors: Vec<String>,
    #[serde(default)]
    pub similarity_score: f32,
}


