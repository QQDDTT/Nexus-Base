use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::RwLock;

use crate::entities::{EngineError, MemoryFragment};

/// 从按行分隔的 JSON 序列化文件加载所有分片数据 (适用于恢复 Snapshot 或 WAL)
pub fn load_fragments_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<MemoryFragment>, EngineError> {
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut fragments = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }
        let frag: MemoryFragment = serde_json::from_str(&line).map_err(|e| {
            EngineError::CorruptedSnapshot(format!("JSON Parse Error: {}", e))
        })?;
        fragments.push(frag);
    }
    
    Ok(fragments)
}

/// 内存脏数据集合，存储尚未持久化进 nxpack 的数据
pub struct DirtySet {
    fragments: RwLock<Vec<MemoryFragment>>,
}

impl DirtySet {
    pub fn new() -> Self {
        Self {
            fragments: RwLock::new(Vec::new()),
        }
    }

    pub fn insert(&self, fragments: Vec<MemoryFragment>) {
        if let Ok(mut lock) = self.fragments.write() {
            lock.extend(fragments);
        }
    }

    pub fn get_all(&self) -> Vec<MemoryFragment> {
        if let Ok(lock) = self.fragments.read() {
            lock.clone()
        } else {
            Vec::new()
        }
    }

    pub fn clear_all(&self) -> Vec<MemoryFragment> {
        if let Ok(mut lock) = self.fragments.write() {
            // 返回被清理的数据，供写入快照
            std::mem::take(&mut *lock)
        } else {
            Vec::new()
        }
    }
}

/// 预写日志 (Write-Ahead Log)
pub struct WalLog {
    file: File,
}

impl WalLog {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, EngineError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn truncate(&mut self) -> Result<(), EngineError> {
        self.file.set_len(0)?;
        Ok(())
    }

    pub fn append(&mut self, fragments: &[MemoryFragment]) -> Result<(), EngineError> {
        for frag in fragments {
            let mut json_bytes = serde_json::to_vec(frag).map_err(|e| {
                EngineError::CorruptedSnapshot(format!("JSON Serialization error: {}", e))
            })?;
            json_bytes.push(b'\n'); // 换行分隔
            self.file.write_all(&json_bytes)?;
        }
        self.file.sync_data()?;
        Ok(())
    }
}
