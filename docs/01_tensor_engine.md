# 张量引擎与元数据锚点 (Tensor Engine & Metadata Anchors)

## 1. 为什么抛弃全量向量化？
原 Aura Substrate 的设计是对所有源码和文档进行无差别的 Chunk 切分并提取 Embedding。这导致：
- 检索噪音极大，大模型经常提取到不相关的工具类代码。
- 计算开销过高，宿主机的 CPU 常常被用于计算无用的 `node_modules` 向量。

## 2. 元数据锚点 (Metadata Anchors) 机制
`Nexus-Base` 作为专属的个人知识库，引入了**元数据锚点过滤 (Metadata Anchor Filtering)**。
只有包含特定头部标识（如 `[Design/Architect]`, `[Core/Rule]`）的文件或代码片段，才会被提取入库。

<div style="font-family: sans-serif; text-align: center; color: #fff; line-height: 1.5; padding: 25px; border: 1px solid #444; border-radius: 12px; background-color: #1a1a1a; max-width: 700px; margin: 2rem auto;">
    <h3 style="color: #4ade80;">🤖 Agent 检索漏斗图</h3>
    <div style="padding: 10px; background-color: #27272a; border-radius: 8px; margin-bottom: 10px; font-size: 0.9em;">
        <strong>1. 意图缺失 (Context Miss)</strong><br>
        Agent 在编写代码时遇到未知框架，触发 <code>Nexus Retrieval Skill</code>
    </div>
    <div style="font-size: 1.5rem; color: #a1a1aa; margin-bottom: 10px;">⬇️</div>
    <div style="padding: 10px; background-color: #3f3f46; border-radius: 8px; margin-bottom: 10px; width: 80%; margin: 0 auto;">
        <strong>2. 元数据预筛 (Pre-filter by Anchor)</strong><br>
        仅在标记为 <code>[React/UI_Lib]</code> 的锚点区间内检索
    </div>
    <div style="font-size: 1.5rem; color: #a1a1aa; margin-bottom: 10px;">⬇️</div>
    <div style="padding: 10px; background-color: #52525b; border-radius: 8px; width: 60%; margin: 0 auto; border: 2px solid #4ade80;">
        <strong>3. 张量余弦碰撞 (Cosine Similarity)</strong><br>
        极速命中相关代码段并返回补齐
    </div>
</div>

## 3. 极速 Rust 内核
为了不占用 Agent 的推理模型内存，Nexus-Base 采用纯 Rust 编写，其底层的 HNSW (分层导航小世界) 算法经过了极简裁剪。不提供任何图形管理界面，唯一的交互手段即是通过 gRPC 数据管道。
