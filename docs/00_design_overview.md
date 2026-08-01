---
title: "Nexus-Base 向量知识库设计文档 (基于 Aura Substrate 优化版)"
type: "design"
status: "draft"
---

# Nexus-Base 核心基座设计 (Aura Substrate Refactoring)

## 1. 定位演进
原 `Aura` 系统的物理基座 (Substrate) 尽管性能强劲，但包含了过多与通用 LLM 对话或图形界面的耦合代码。
现将其提纯、重构并更名为 **`Nexus-Base`**，专职作为开发终端 Agent 的**个人向量知识引擎 (Personal Vector DB)**。

## 2. 优化与重构方向 (Optimization Strategies)

### [剥离冗余模块]
*   **移除**：原有的前台 UI 渲染总线、通用的聊天会话管理 (Chat Session API)、多模态生成管线。
*   **保留并强化**：底层基于 Rust 的极速本地磁盘 I/O、张量化 (Tensor) 并发存取核心算法。

### [检索效率革命 (Retrieval for Agents)]
普通的向量库面向人类问答设计，而 `Nexus-Base` 只面向代码和 Agent 的上下文机制设计：
*   **元数据锚点 (Metadata Anchors)**：不再切碎代码片段，而是要求入库的所有长尾开发经验均需绑定 `Platform_Rules` 或 `Foundations` 的顶层引用哈希。
*   **永续缓存树 (Persistent Cache Tree)**：支持热备份和冷加载。当新的开发终端启动时，可直接读取 `Download Center` 下发的 `Nexus-Base` 数据快照，无需重新构建数百万代码的 Embeddings，一秒进入拥有“十年开发经验”的记忆状态。

## 3. 技术规范与部署方案
*   **语言架构**：Rust (保证与原 Aura Substrate 的平滑过渡，并在资源占用上做到极致节省，不与开发终端的 LLM 抢夺算力)。
*   **对接通道**：采用双层分化通信架构 (Dual-Layer Communication Architecture)。底层通过二进制 gRPC + Unix Domain Socket 保持 <1ms 级本地 IPC 检索极速；上层提供可选的轻量级 MCP (Model Context Protocol) 适配层，无缝接入主流 Agentic 平台（如 Cursor, Antigravity IDE, Claude Desktop 等）。
*   **安全性**：引入单向沙箱屏障。由于其存储了整个开发终端积累的核心机密，数据流仅允许 Agent 查询和新增，删除或重置必须获得“超级管理员”终端的数字签名授权。
