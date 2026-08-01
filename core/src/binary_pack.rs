

use crate::entities::{EngineError, MemoryFragment};

/// 单个 Fragment 的紧凑二进制布局工具。
/// Layout:
/// - [u32: total_block_len]
/// - [u16: id_len], [bytes: id]
/// - [u16: file_path_len], [bytes: file_path]
/// - [u32: content_len], [bytes: content]
/// - [u16: anchors_count]
///   - for each anchor: [u16: anchor_len], [bytes: anchor]
/// - [f32: similarity_score] (4 bytes)
/// - [u16: tensor_len (elements)], [f32 elements...] (tensor_len * 4 bytes)

pub fn pack_fragment(frag: &MemoryFragment) -> Vec<u8> {
    let mut buf = Vec::new();

    // 预留 total_len (4 bytes) 的位置
    buf.extend_from_slice(&0u32.to_ne_bytes());

    // 1. id
    let id_bytes = frag.id.as_bytes();
    buf.extend_from_slice(&(id_bytes.len() as u16).to_ne_bytes());
    buf.extend_from_slice(id_bytes);

    // 2. file_path
    let fp_bytes = frag.file_path.as_bytes();
    buf.extend_from_slice(&(fp_bytes.len() as u16).to_ne_bytes());
    buf.extend_from_slice(fp_bytes);

    // 3. content
    let ct_bytes = frag.content_snippet.as_bytes();
    buf.extend_from_slice(&(ct_bytes.len() as u32).to_ne_bytes());
    buf.extend_from_slice(ct_bytes);

    // 4. anchors
    buf.extend_from_slice(&(frag.metadata_anchors.len() as u16).to_ne_bytes());
    for anchor in &frag.metadata_anchors {
        let a_bytes = anchor.as_bytes();
        buf.extend_from_slice(&(a_bytes.len() as u16).to_ne_bytes());
        buf.extend_from_slice(a_bytes);
    }

    // 5. score
    buf.extend_from_slice(&frag.similarity_score.to_ne_bytes());

    // 6. tensor
    buf.extend_from_slice(&(frag.embedding.len() as u16).to_ne_bytes());
    for val in &frag.embedding {
        buf.extend_from_slice(&val.to_ne_bytes());
    }

    // 更新 total_len
    let total_len = buf.len() as u32;
    buf[0..4].copy_from_slice(&total_len.to_ne_bytes());

    buf
}

/// 用于在内存映射区域 (Mmap) 上零拷贝滑动读取的游标指针
pub struct BinaryView<'a> {
    pub raw_data: &'a [u8],
}

impl<'a> BinaryView<'a> {
    /// 反序列化出一个游标结构
    pub fn unpack(data: &'a [u8]) -> Result<MemoryFragment, EngineError> {
        let mut offset = 4; // 跳过 total_len

        let id_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
        offset += 2;
        let id = String::from_utf8(data[offset..offset+id_len].to_vec()).unwrap();
        offset += id_len;

        let fp_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
        offset += 2;
        let file_path = String::from_utf8(data[offset..offset+fp_len].to_vec()).unwrap();
        offset += fp_len;

        let ct_len = u32::from_ne_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let content_snippet = String::from_utf8(data[offset..offset+ct_len].to_vec()).unwrap();
        offset += ct_len;

        let anchors_count = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
        offset += 2;
        let mut metadata_anchors = Vec::with_capacity(anchors_count);
        for _ in 0..anchors_count {
            let a_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
            offset += 2;
            let a_str = String::from_utf8(data[offset..offset+a_len].to_vec()).unwrap();
            offset += a_len;
            metadata_anchors.push(a_str);
        }

        let similarity_score = f32::from_ne_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;

        let tensor_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
        offset += 2;
        let mut embedding = Vec::with_capacity(tensor_len);
        for _ in 0..tensor_len {
            let val = f32::from_ne_bytes(data[offset..offset+4].try_into().unwrap());
            offset += 4;
            embedding.push(val);
        }

        Ok(MemoryFragment {
            id,
            file_path,
            content_snippet,
            embedding,
            metadata_anchors,
            similarity_score,
        })
    }
}

/// 快速从裸块中直接提取 Tensor 数组用于计算打分。
/// 这个过程是零分配（不会分配任何 String，也不会进行全反序列化）。
pub fn extract_tensor_from_block(data: &[u8]) -> Vec<f32> {
    let mut offset = 4; // jump total_len

    let id_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
    offset += 2 + id_len;

    let fp_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
    offset += 2 + fp_len;

    let ct_len = u32::from_ne_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    offset += 4 + ct_len;

    let anchors_count = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
    offset += 2;
    for _ in 0..anchors_count {
        let a_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
        offset += 2 + a_len;
    }

    offset += 4; // jump score

    let tensor_len = u16::from_ne_bytes(data[offset..offset+2].try_into().unwrap()) as usize;
    offset += 2;
    
    // 不建议返回 &'a [f32] 是因为平台对齐问题可能会引发指针 panic。
    // 在内存中顺序取出组装是最稳妥且依旧比全量 JSON 解析快千万倍的方案。
    let mut tensor = Vec::with_capacity(tensor_len);
    for _ in 0..tensor_len {
        let val = f32::from_ne_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;
        tensor.push(val);
    }
    tensor
}

/// 迭代 Mmap 中的所有二进制块
pub struct MmapBlockIterator<'a> {
    pub data: &'a [u8],
    pub cursor: usize,
}

impl<'a> Iterator for MmapBlockIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.data.len() {
            return None;
        }

        let total_len = u32::from_ne_bytes(self.data[self.cursor..self.cursor+4].try_into().unwrap()) as usize;
        let block = &self.data[self.cursor..self.cursor+total_len];
        self.cursor += total_len;
        Some(block)
    }
}
