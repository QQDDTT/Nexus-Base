# Nexus-Base 智能体协助规范 (Project Rules)

本文档定义了所有 AI Agent 在维护与扩展 `Nexus-Base` 仓库时，必须严格遵守的项目级准则与约束。

## 1. 极致性能与零拷贝 (Zero-Copy & Performance)
- **内存分配限制**: 在处理大规模 Embedding 向量或日志回放时，严禁使用不必要的 `.clone()` 或大量的堆分配。
- **Mmap 优先**: 对于冷数据的读取、扫描和检索，必须复用 Phase 4 构建的 `memmap2` 零拷贝模型。任何试图将快照文件整体读入内存 (`fs::read`) 的行为都会被视为严重的退化并被拒绝。
- **二进制紧凑布局**: `nexus_core::binary_pack` 定义了专门针对 SIMD/缓存行友好的连续二进制协议。绝不允许在核心存储层引入 JSON、Bincode 等带有额外序列化开销的中间件。

## 2. 系统安全与 ED25519 鉴权 (Security & Auth)
- **写操作必校验**: `IngestExperience`、`CommitSnapshot` 等一切能够修改持久化状态的接口，都必须在拦截器或服务层校验 ED25519 的签名。
- **Agent 测试规范**: 若 Agent 需要测试写入，必须通过 `nexus-cli keygen` 生成 `private.key`，并基于 `SystemTime` 颁发签名。绝不可在未告知用户的情况下试图删去服务端的鉴权代码以图测试方便。

## 3. 架构与编码风格 (Architecture & Style)
- **微服务分层不可逾越**:
  - `core`: 纯逻辑引擎，不含任何网络代码。
  - `server`: gRPC 服务网关，负责鉴权与协议转换。
  - `mcp-adapter`: MCP 协议转换层，供大模型直接调用。
  - `cli`: 本地终端入口，调用远程 LLM（如 Ollama）并转发 gRPC。
- **统一日志**: 禁止在生产库代码中使用 `println!`，必须统一使用 `tracing::info!`、`tracing::warn!` 等结构化日志；仅在 `tests/` 和 `cli` 中允许输出控制台格式。

## 4. 自动化测试驱动 (Test-Driven Maintenance)
- 在完成任何模块逻辑后，必须先在 `tests/` 中进行完整的集成或并发压测，确保系统的性能红线（如检索延迟及内存占用）不倒退，随后才能宣告任务完成。

## 5. Publisher 子项目与前端发布规范 (Publisher & Deployment)
- **独立演进**: `nexus-publisher` 目录是一个独立的 TypeScript 项目，专用于 Nexus-Base 的配套前端、文档站或独立页面发布（特别是向 Cloudflare Pages 等平台部署）。
- **静态构建优先**: 在构建部署至 Cloudflare Pages 的产物时，必须确保配置了正确的构建命令与输出目录（如 `dist/` 或 `out/`），且默认采用静态生成策略，避免在未做 Worker 适配时强行使用 SSR。
- **环境与依赖隔离**: `nexus-publisher` 中的 NPM 依赖（`node_modules`）以及任何 Node.js 脚本，严格限制在该目录下，绝不可污染外部 Rust 工作区的依赖管理与执行流。
