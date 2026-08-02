# Nexus-Base 项目目录结构指南 (Project Structure)

本文件详细描述了 `Nexus-Base` 工程的代码组织架构。整个工程作为一个标准的 Rust Cargo Workspace 进行管理，各个模块职责具有严格的物理隔离。

## 目录树概览

```text
Nexus-Base/
├── Cargo.toml                  # Cargo Workspace 顶层配置，管理子项目和全局属性
├── rustfmt.toml                # 强制的代码格式化规范 (如缩进和行宽)
├── .clippy.toml                # 严格的 Clippy Linter 性能与语法约束规则
│
├── docs/                       # 核心文档库 (全扁平化结构)
│   ├── README.md               # 内部全部文档的导航索引
│   ├── 00_design_overview.md   # 顶层设计哲学
│   ├── 01_... ~ 08_...         # 系统级架构、存储、接口与跨平台宏观设计
│   └── 20_... ~ 21_...         # 具体子模块 (core/cli) 的局部详细抽象设计
│
├── nexus-proto/                      # gRPC 契约与自动生成模块 (Crate: nexus-proto)
│   ├── Cargo.toml
│   ├── build.rs                # 编译脚本：调用 tonic-build 编译 proto
│   ├── nexus.proto             # 跨语言、跨进程的核心 Protobuf 接口定义文件
│   └── src/
│       └── lib.rs              # 导出由 proto 自动生成的原生 Rust 结构体与 Trait
│
├── nexus-core/                       # 底层张量引擎与计算模块 (Crate: nexus-core)
│   ├── Cargo.toml              
│   └── src/
│       ├── lib.rs              # 引擎对外的 API 门面暴露点
│       └── entities.rs         # 核心领域实体 (MemoryFragment, HnswGraph, DirtySet等)
│
├── nexus-server/                     # 网络通信与状态挂起模块 (Crate: nexus-server)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs              # 包含基于 Tokio + Tonic 的异步 gRPC 服务端实现 (待填充)
│
└── nexus-cli/                        # 物理管控与终端应用模块 (Crate: nexus-cli)
    ├── Cargo.toml
    └── src/
        └── main.rs             # 基于 Clap 的命令行可执行程序入口，管理超级管理员操作
```

## 模块间的物理依赖流向

由于采用了 Workspace 设计，子项目间的依赖呈现单向数据流的形式，杜绝了循环引用，增强了引擎底座的安全性：

1. **`nexus-proto`**：最底层。它仅仅依赖公共的 `prost`，对外提供纯粹的数据结构，**不依赖**项目中的任何其他子库。
2. **`nexus-core`**：纯算力层。它直接操作 OS 文件系统和内存映射，**不感知网络，不依赖 proto**。
3. **`nexus-server`**：通信网关层。它同时依赖并拉拢 `nexus-core` 和 `nexus-proto`，负责将 Protobuf 结构转化为 Core 所需的实体，将结果封装后通过 gRPC 吐出。
4. **`nexus-cli`**：顶层执行壳。它既可以挂载 `nexus-server` 以守护进程运行，也可以在离线时依赖 `nexus-proto` 组装结构并发起管理指令。

任何新增加的文件或功能，均应按照以上四层的逻辑，放入最合适的物理目录中。
