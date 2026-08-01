# API 设计与数据库事务机制 (API & Transaction Design)

## 1. 核心 API 扩展设计 (gRPC & HTTP Fallback)

基于 `Nexus-Base` 的定位，我们不仅需要前置的只读和写入接口，还需要支持复杂生命周期的管理操作。接口层面默认提供 gRPC，同时可以通过 sidecar 或内置轻量 HTTP 模块提供 RESTful 降级（供 curl 或简单脚本测试）。

```protobuf
syntax = "proto3";
package nexus.v1;

service NexusKnowledgeService {
    // ---------------- 读取操作 ----------------
    // 供 Agent 极速查询上下文片段 (无锁、MVCC 隔离读)
    rpc RetrieveContext(RetrieveRequest) returns (RetrieveResponse);
    
    // ---------------- 写入操作 ----------------
    // Agent 吸收新经验，开启隐式事务，将写入暂存至 WAL
    rpc IngestExperience(IngestRequest) returns (IngestResponse);
    
    // ---------------- 事务与管理操作 (需要 Admin 签名) ----------------
    // 强制将当前内存的变更刷入磁盘树，生成新的快照点
    rpc CommitSnapshot(CommitRequest) returns (CommitResponse);
    
    // 发生错误时，回滚到上一个持久化快照，丢弃未 Commit 的 WAL 记录
    rpc RollbackState(RollbackRequest) returns (RollbackResponse);
}
```

## 2. 数据库事务机制 (Transactions for Tensor Engine)

不同于传统的行级关系型数据库，Nexus-Base 的底层是张量图（HNSW）。由于张量图的节点相连错综复杂，部分写入会导致图结构断裂。因此必须引入 **ACID 事务保障**：

### 2.1 MVCC (多版本并发控制) 与无锁读
* **隔离级别：Snapshot Isolation (快照隔离)**
* 当 Agent 调用 `RetrieveContext` 时，检索引擎分配一个当前的全局读视图（Read View）。
* 即使此时后台有大量 `IngestExperience` 写入操作在调整图的连通性，读操作永远只在旧的、完整的指针树上执行，**实现零等待无锁读取**。

### 2.2 WAL (预写日志) 与原子性写入
* 当调用 `IngestExperience` 时，数据首先被串行化追加到磁盘的 `wal.log` 中。
* 内存中生成一个新的“脏节点”集合（Dirty Set）。
* 只有当积攒到一定阈值，或者调用 `CommitSnapshot` 时，才会执行原子的**树重平衡（Tree Rebalancing）**，并将原子指针翻转，使其对新的查询可见。如果中途宕机，重启时会通过 `wal.log` 恢复脏集合，避免数据半写。

## 3. 与 Agent 的交互工作流 (Agent Interaction Flow)

`Nexus-Base` 与终端 Agent 协同工作时，通常遵循以下标准化交互流：

### 场景 A：需求理解与代码生成 (Context Hydration)
1. **Agent 意图产生**：Agent 接收到用户的开发指令（例如：“重构这里的权限校验逻辑”）。
2. **触发检索**：Agent 构建 `RetrieveRequest`，提取意图张量和当前文件的路径作为 `anchors`。
3. **极速返回**：`Nexus-Base` 在 10ms 内返回最相关的 5 条历史经验或架构规范（MVCC 保证读不阻塞）。
4. **生成代码**：Agent 将这些记忆作为 Prompt Context，生成更符合本地架构的代码。

### 场景 B：踩坑经验沉淀 (Experience Ingestion)
1. **调试闭环**：Agent 发现了一处导致编译失败的错误，并经过几轮尝试后终于修复。
2. **知识提取**：Agent 自动将“错误表现”与“正确的解决方案代码段”打包。
3. **发起写入**：调用 `IngestExperience` 接口将这组张量对提交给 `Nexus-Base`。
4. **事务挂起**：`Nexus-Base` 返回成功，数据进入 WAL，此时不阻塞 Agent 接下来的任务。
5. **异步落盘**：在系统空闲期，`Nexus-Base` 的后台线程自动完成图结构的原子更新合并（Commit）。
