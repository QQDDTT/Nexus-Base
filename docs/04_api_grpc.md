# 与 Agent 终端的对接契约 (Agent Connection Interface)

## 1. 协议选择：为什么是 gRPC？
虽然 HTTP RESTful 对人类开发者更加友好，但 `Nexus-Base` 主要服务于本地环境下的其他守护进程（如 `Agent_Platform` 的执行引擎）。
出于对毫秒级延迟与强类型安全的要求，我们放弃 JSON 解析，全面采用 **gRPC + Protocol Buffers** 建立本地 IPC (进程间通信) 管道。

## 2. 核心通信契约 (Proto Definitions)
下面是用于双向交互的核心数据结构：

```protobuf
syntax = "proto3";
package nexus.v1;

service NexusKnowledgeService {
    // 供 Agent 查询上下文片段 (无状态/极速查询)
    rpc RetrieveContext(RetrieveRequest) returns (RetrieveResponse);
    
    // 供 Agent 吸收并沉淀新的开发经验 (触发张量树重平衡)
    rpc IngestExperience(IngestRequest) returns (IngestResponse);
}

message RetrieveRequest {
    string intent_query = 1;     // LLM 生成的自然语言或代码意图
    repeated string anchors = 2; // 元数据锚点过滤，如 ["UI_Component", "Bug_Fix"]
    uint32 top_k = 3;            // 限制返回的最大条数，防止上下文溢出
}

message RetrieveResponse {
    repeated MemoryFragment fragments = 1;
    float compute_latency_ms = 2; // 诊断字段：向量运算耗时
}

message MemoryFragment {
    string id = 1;               // 张量节点 UUID
    string file_path = 2;        // 原始文件溯源路径
    string content_snippet = 3;  // 相关的代码或文档片段
    float similarity_score = 4;  // 绝对余弦相似度得分 (0.0 - 1.0)
}
```

## 3. Rust Tonic 内部实现约束 (gRPC Server)
在 `Nexus-Base` 的服务端实现中，必须使用 `tonic` 框架，并且必须限制最大并发连接数与单次载荷，以防被失控的 Agent 刷爆宿主机内存：

```rust
use tonic::{Request, Response, Status};
use nexus_pb::nexus_knowledge_service_server::NexusKnowledgeService;

#[derive(Debug, Default)]
pub struct NexusEngineImpl {
    // 内部持有的内存映射（Mmap）或并发安全的张量库引用
    pub tensor_tree: Arc<RwLock<HnswGraph>>,
}

#[tonic::async_trait]
impl NexusKnowledgeService for NexusEngineImpl {
    async fn retrieve_context(
        &self,
        request: Request<RetrieveRequest>,
    ) -> Result<Response<RetrieveResponse>, Status> {
        let req = request.into_inner();
        
        // 1. 拦截防御：防止单次查询占用过多显存
        if req.top_k > 50 {
            return Err(Status::invalid_argument("top_k cannot exceed 50 tokens."));
        }
        
        // 2. 将纯文本意图快速编码为本地 Embedding
        let query_tensor = local_embedding_model::encode(&req.intent_query).await?;
        
        // 3. 在 HNSW 图中搜索
        let tree = self.tensor_tree.read().await;
        let matches = tree.search(&query_tensor, req.top_k, req.anchors);
        
        // ... (转换并返回 Response)
        Ok(Response::new(RetrieveResponse { /* ... */ }))
    }
}
```

## 4. 防穿透安全机制 (Anti-Penetration Flow)
作为个人核心资产库，接口层设有只读沙箱。
`RetrieveContext` 与 `IngestExperience` 是无需强授权的默认接口。但是任何试图调用内部未公开的高危操作（如清空整个知识树）时，将触发如下的 Ed25519 物理隔离墙校验：

<div style="font-family: sans-serif; text-align: center; color: #fff; padding: 30px; border: 1px solid #52525b; border-radius: 12px; background-color: #18181b; margin: 2rem 0; box-shadow: inset 0 0 40px rgba(0,0,0,0.8);">
    
    <div style="display: flex; align-items: center; justify-content: center; gap: 30px; margin-bottom: 20px;">
        
        <div style="padding: 15px 25px; background: #27272a; border: 1px solid #71717a; border-radius: 8px;">
            <span style="font-size: 2rem;">🤖</span><br>
            <strong>Agent 试图删除记忆</strong><br>
            <code>rpc DeleteFragment</code>
        </div>
        
        <div style="color: #ef4444; font-size: 2rem; font-weight: bold;">
            ❌ 拦截
        </div>
        
        <div style="padding: 15px 25px; background: rgba(239, 68, 68, 0.1); border: 2px dashed #ef4444; border-radius: 8px;">
            <span style="font-size: 2rem;">🛡️</span><br>
            <strong>Tonic 鉴权中间件</strong><br>
            <span style="font-size: 0.8em; color: #fca5a5;">无 Admin 签名，抛出 gRPC 401 Unauthenticated</span>
        </div>
        
    </div>
    
    <p style="font-size: 0.9em; color: #a1a1aa; max-width: 500px; margin: 0 auto; line-height: 1.6;">
        无论 LLM 发生何种越狱或幻觉，因为 <strong>Worker Agent</strong> 本身在物理内存中就不持有 <code>Super_Admin_Private_Key</code>，因此它在数学层面上绝对不可能绕过这道 gRPC 中间件屏障，保证了十年代码经验库的绝对安全。
    </p>

</div>

## 5. 主流 Agentic 生态适配：MCP 双层网关设计 (Dual-Layer MCP Adapter)

### 5.1 通信架构分层决议
在当今 AI Agent (Agentic) 生态中，通信协议呈现双层分化格局：
* **系统底层 (Engine IPC Level)**：专有向量库与守护进程间要求毫秒级检索与零序列化负担，采用 **gRPC + Unix Domain Socket (UDS)** 是最佳技术方案。
* **工具生态层 (Tool & Context Level)**：主流 Agentic 平台（如 Cursor, Antigravity IDE, Claude Desktop, LangGraph 等）统一使用 **MCP (Model Context Protocol)** 规范进行 Tool Call 和上下文挂载。

为了兼顾性能与生态通用性，`Nexus-Base` 采用 **双层分化通信架构 (Dual-Layer Architecture)**：

```
┌─────────────────────────────────────────────────────────────────────────┐
|                       主流 Agentic 生态平台                             |
|    (Antigravity IDE / Cursor / Claude Desktop / LangGraph / AutoGen)    |
└─────────────────────────────────────────────────────────────────────────┘
         │                                       │
         │ [标准 MCP 协议]                       │ [直接 gRPC 内部调用]
         │ (Stdio / Streamable HTTP)             │ (Unix Domain Socket / TCP)
         ▼                                       ▼
┌───────────────────────────────────┐   ┌─────────────────────────────────┐
|  Nexus MCP 适配网关 (Sidecar/CLI) |   |    Agent_Platform 专用执行器    |
|  - 遵循 MCP 2026-07-28 无状态规范 |   |    (直连 Protobuf 二进制通道)    |
|  - 映射 JSON-RPC 转换为 gRPC 请求 |   └─────────────────────────────────┘
└───────────────────────────────────┘                    │
         │                                               │
         └───────────────────────┬───────────────────────┘
                                 ▼
               ┌───────────────────────────────────┐
               |  Nexus-Base 极速引擎 (Rust Daemon)|
               |  - gRPC Server (Tonic)            |
               |  - HNSW 检索 / WAL / 热快照挂载    |
               └───────────────────────────────────┘
```

### 5.2 MCP 工具映射规范 (Tool Schema Mapping)

`Nexus-Base` 的 MCP 适配网关（可通过 `nexus_daemon --mcp-stdio` 或独立二进制 `nexus-mcp` 启动）向外部 Agent 暴露以下标准 Tool：

#### 1. `search_knowledge_context`
* **说明**：供 Agent 检索关联上下文与历史经验片段。
* **MCP 工具定义**：
  ```json
  {
    "name": "search_knowledge_context",
    "description": "极速检索 Nexus-Base 向量知识库中与当前意图关联的代码片段与长尾经验",
    "inputSchema": {
      "type": "object",
      "properties": {
        "intent_query": { "type": "string", "description": "自然语言或代码意图描述" },
        "anchors": { "type": "array", "items": { "type": "string" }, "description": "元数据锚点过滤" },
        "top_k": { "type": "number", "default": 5, "description": "最大返回条数 (≤50)" }
      },
      "required": ["intent_query"]
    }
  }
  ```
* **转换管道**：接收 JSON-RPC 请求 ➔ 转译为 gRPC `RetrieveContextRequest` ➔ 发送给 `nexus_daemon` ➔ 响应格式化为 MCP `structuredContent`。

#### 2. `ingest_knowledge_experience`
* **说明**：供 Agent 提交并沉淀新的解决经验与模式。
* **MCP 工具定义**：
  ```json
  {
    "name": "ingest_knowledge_experience",
    "description": "向 Nexus-Base 知识库中沉淀新的关键经验或通用模式",
    "inputSchema": {
      "type": "object",
      "properties": {
        "fragments": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "file_path": { "type": "string" },
              "content_snippet": { "type": "string" }
            },
            "required": ["file_path", "content_snippet"]
          }
        }
      },
      "required": ["fragments"]
    }
  }
  ```
* **转换管道**：接收 JSON-RPC 请求 ➔ 转译为 gRPC `IngestExperienceRequest` ➔ 发送给 `nexus_daemon`。

### 5.3 适配最新 MCP (2026-07-28) 协议要点
1. **完全无状态 (Stateless)**：适配层不维护 Session 状态，自动解析请求体中的 `params._meta`（包含 `protocolVersion` 与 `capabilities`）。
2. **多轮往返 (MRTR) 支持**：高危管理操作（如清理或强制重构快照）通过返回 `input_required` 响应要求 Agent 宿主用户交互确认后重新发起。
3. **高效本地传输**：默认提供 Stdio 传输通道，零端口暴露，安全对接本地 IDE 与守护引擎。

