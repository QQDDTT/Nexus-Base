use std::sync::Mutex;
use std::path::PathBuf;
use std::fs::OpenOptions;

use memmap2::{Mmap, MmapOptions};

use crate::entities::{EngineError, MemoryFragment};
use crate::storage::{DirtySet, WalLog};
use crate::binary_pack::{pack_fragment, extract_tensor_from_block, BinaryView, MmapBlockIterator};
use crate::math::cosine_similarity;

/// 核心对外门面
pub trait TensorEngine: Send + Sync {
    /// 基于张量和锚点检索相关上下文 (无锁读)
    fn retrieve(
        &self,
        query_tensor: &[f32],
        anchors: &[String],
        top_k: usize,
    ) -> Result<Vec<MemoryFragment>, EngineError>;

    /// 将新的经验元组吸收入库 (写入 WAL)
    fn ingest(&self, fragments: Vec<MemoryFragment>) -> Result<(), EngineError>;

    /// 强制执行重平衡并将脏数据持久化至 .nxpack 物理磁盘快照
    fn commit_snapshot(&self) -> Result<(), EngineError>;
}

pub struct EngineConfig {
    pub wal_path: PathBuf,
    pub snapshot_path: PathBuf,
}

/// 基于 Zero-Copy Mmap 的新一代引擎实现载体
pub struct NexusEngine {
    config: EngineConfig,
    dirty_set: DirtySet,
    // 采用 Mmap 直接映射内存区域，大幅度避免 OOM 与启动时间
    snapshot_mmap: std::sync::RwLock<Option<Mmap>>,
    wal_log: Mutex<WalLog>,
}

impl NexusEngine {
    /// 初始化引擎。如果有历史快照和 WAL 会自动进行崩溃恢复加载
    pub fn new(config: EngineConfig) -> Result<Self, EngineError> {
        // 1. 恢复快照基线数据 (Mmap 零分配)
        let mmap = if config.snapshot_path.exists() {
            let file = OpenOptions::new().read(true).open(&config.snapshot_path)?;
            let meta = file.metadata()?;
            if meta.len() > 0 {
                // 安全警告：外部如果并发修改此文件会导致崩溃。此程序内部写入时是以追加模式，但重载时可能需要注意。
                Some(unsafe { MmapOptions::new().map(&file)? })
            } else {
                None
            }
        } else {
            None
        };

        // 2. 恢复尚未 Commit 的脏数据 (WAL 依然保持 JSON 行格式，以便追加)
        let wal_fragments = crate::storage::load_fragments_from_file(&config.wal_path)?;

        let dirty_set = DirtySet::new();
        dirty_set.insert(wal_fragments);

        // 3. 打开 WAL 文件柄用于后续追加
        let wal_log = WalLog::open(&config.wal_path)?;

        Ok(Self {
            config,
            dirty_set,
            snapshot_mmap: std::sync::RwLock::new(mmap),
            wal_log: Mutex::new(wal_log),
        })
    }
}

impl TensorEngine for NexusEngine {
    fn retrieve(
        &self,
        query_tensor: &[f32],
        anchors: &[String],
        top_k: usize,
    ) -> Result<Vec<MemoryFragment>, EngineError> {
        // 用于存储候选的内存切片和分数
        let mut scored_blocks: Vec<(&[u8], f32)> = Vec::new();
        
        let mmap_lock = self.snapshot_mmap.read().unwrap();
        if let Some(mmap) = &*mmap_lock {
            // 注意：Mmap 返回的结构体能够自动转换为 &[u8]
            let data: &[u8] = mmap;
            let iter = MmapBlockIterator { data, cursor: 0 };
            for block in iter {
                // 在不进行全量反序列化的情况下，直接通过游标强解出 f32 array
                let tensor = extract_tensor_from_block(block);
                
                // 暂时忽略 anchor 过滤的裸读取 (Phase 4 重点优化了张量读取，Anchor 过滤可在二进制层增加)
                // 这里我们直接打分
                let score = cosine_similarity(query_tensor, &tensor);
                scored_blocks.push((block, score));
            }
        }

        // 将 dirty_set 的数据也拉出来打分
        let mut dirty_candidates = self.dirty_set.get_all();
        if !anchors.is_empty() {
            dirty_candidates.retain(|frag| {
                frag.metadata_anchors.iter().any(|a| anchors.contains(a))
            });
        }
        for frag in &mut dirty_candidates {
            frag.similarity_score = cosine_similarity(query_tensor, &frag.embedding);
        }

        // 排序 Mmap 命中的区块
        scored_blocks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_blocks.truncate(top_k);

        // 将 Mmap 中排名前 K 的区块通过 BinaryView 解包成 MemoryFragment
        let mut final_candidates = Vec::new();
        for (block, score) in scored_blocks {
            let mut frag = BinaryView::unpack(block)?;
            frag.similarity_score = score;
            
            // 补做 anchor 过滤
            if anchors.is_empty() || frag.metadata_anchors.iter().any(|a| anchors.contains(a)) {
                final_candidates.push(frag);
            }
        }

        // 合并 dirty 并在整体上做最终 Top-K 截断
        final_candidates.extend(dirty_candidates);
        final_candidates.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        final_candidates.truncate(top_k);

        Ok(final_candidates)
    }

    fn ingest(&self, fragments: Vec<MemoryFragment>) -> Result<(), EngineError> {
        // 1. 持久化保证：序列化并写入 WAL
        {
            let mut wal = self.wal_log.lock().map_err(|_| EngineError::LockAcquisitionFailed)?;
            wal.append(&fragments)?;
        }
        
        // 2. 内存可见性：写入成功后追加到 DirtySet
        self.dirty_set.insert(fragments);
        Ok(())
    }

    fn commit_snapshot(&self) -> Result<(), EngineError> {
        // 1. 获取写锁，防止 commit 时写入新数据
        let mut wal = self.wal_log.lock().map_err(|_| EngineError::LockAcquisitionFailed)?;

        // 2. 取出当前所有脏数据
        let fragments = self.dirty_set.clear_all();
        if fragments.is_empty() {
            return Ok(()); // 无数据需要 commit
        }

        // 3. 将脏数据打包为二进制格式刷入 .nxpack
        let mut snapshot_file = OpenOptions::new()
            .create(true)
            .append(true) // 二进制紧凑追加
            .read(true)
            .open(&self.config.snapshot_path)?;
        
        for frag in &fragments {
            let binary_buf = pack_fragment(frag);
            std::io::Write::write_all(&mut snapshot_file, &binary_buf)?;
        }
        snapshot_file.sync_data()?;

        // 4. 重载 Mmap
        let new_mmap = unsafe { MmapOptions::new().map(&snapshot_file)? };
        *self.snapshot_mmap.write().unwrap() = Some(new_mmap);

        // 5. 清理旧的 WAL
        wal.truncate()?;

        Ok(())
    }
}
