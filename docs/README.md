# Nexus-Base 知识库文档导航 (Documentation Index)

欢迎查阅 `Nexus-Base` 项目文档。本项目旨在提供一个极速、跨平台的个人向量检索引擎，专为本地 Agent 和代码知识沉淀设计。

为避免过度嵌套，所有设计规范均统一存放在平级目录。当文档规模预计控制在 20 份以内时，我们采用编号前缀的方式进行逻辑分组：

## 概览与设计哲学
* **[00_design_overview.md](./00_design_overview.md)** - 顶层基座设计理念与定位演进。
* **[10_project_structure.md](./10_project_structure.md)** - 整个 Cargo Workspace 目录结构布局及模块间的物理依赖流向。

## 底层引擎架构 (Architecture)
* **[01_tensor_engine.md](./01_tensor_engine.md)** - 张量检索引擎设计及“元数据锚点”机制。
* **[02_hardware_performance.md](./02_hardware_performance.md)** - 针对不同规模向量库的内存、I/O 及计算延迟的理论预估。

## 数据存储与接口通信 (Data & Interfaces)
* **[03_storage_cache.md](./03_storage_cache.md)** - “永续缓存树”持久化与热备份分发机制。
* **[04_api_grpc.md](./04_api_grpc.md)** - 服务端 gRPC 通信契约、防穿透请求鉴权机制及面向 Agentic 生态的 MCP 双层适配网关设计。
* **[05_api_transaction.md](./05_api_transaction.md)** - 复杂事务接口、WAL 持久化与 MVCC 无阻塞读设计。

## 平台适配 (Cross-Platform)
* **[06_cross_platform.md](./06_cross_platform.md)** - IPC 混合设计、Mmap 兼容、SIMD 加速抽象及 Cloud Build 矩阵构建方案。

---

## 模块详细设计 (Detailed Module Design)
针对代码工程内各个具体组件的独立设计契约，统一分配 `20+` 编号进行隔离：

* **[20_module_core.md](./20_module_core.md)** - `nexus-core` 纯计算引擎代码模块职责与内部接口规范。
* **[21_module_cli.md](./21_module_cli.md)** - `cli` 模块超级管理员终端命令设计与鉴权工作流。
* **[22_module_proto.md](./22_module_proto.md)** - `nexus-proto` 数据通信契约与生成的 RPC 服务骨架。
* **[23_module_server.md](./23_module_server.md)** - `server` 模块作为 gRPC 网关的安全鉴权与实体转换。
