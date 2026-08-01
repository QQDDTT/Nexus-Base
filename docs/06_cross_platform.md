# 跨平台通信与挂载架构 (Cross-Platform IPC & Mount Design)

`Nexus-Base` 作为高性能的张量检索引擎，需要与外部环境（如 `Agent_Platform` 或第三方 CLI）进行极低延迟的通信，同时保证在不同操作系统上的兼容性。

## 1. 进程间通信 (IPC) 的混合架构
为解决不同操作系统的底层通信机制差异，Nexus-Base 采用 **动态回退 (Dynamic Fallback)** 策略：

* **首选机制 (原生高性能)**：
  * **Linux / macOS**: 使用 `Unix Domain Socket` (UDS)。套接字文件存放于系统标准的运行时目录。
  * **Windows**: 使用 `Named Pipes` (命名管道)。
* **回退/跨设备机制 (通用兼容)**：
  * 当原生 IPC 失败或在特定的隔离容器环境内时，自动降级并启动 **gRPC over TCP (127.0.0.1)**，以此保证核心功能在任意环境下 100% 可用。

## 2. 存储与快照路径标准化
为避免路径硬编码（如 `~/.nexus`），必须引入统一的抽象：
* 所有的元数据存储和张量快照缓存路径，必须通过 Rust 的 `dirs` crate 获取 OS 级别的标准数据目录。
  * **Linux**: `~/.local/share/Nexus/`
  * **macOS**: `~/Library/Application Support/Nexus/`
  * **Windows**: `%APPDATA%\Nexus\`
* 所有路径合并与解析，必须使用 `std::path::PathBuf`，严禁在源码中出现手动的 `/` 或 `\` 拼接。

## 3. 配置文件与日志路径标准化
除了数据快照存储外，针对配置及日志，同样遵循系统级规范，便于系统管理员和监控工具介入：
* **配置文件路径**：
  * **Linux / macOS**: `~/.config/Nexus/` 或 `/etc/nexus/` (系统级)
  * **Windows**: `%APPDATA%\Nexus\config\`
* **日志与诊断文件**：
  * 采用结构化日志记录。
  * **Linux / macOS**: `~/.cache/Nexus/logs/` 或 `/var/log/nexus/` (系统级)
  * **Windows**: `%LOCALAPPDATA%\Nexus\logs\`

## 4. 文件锁与进程互斥 (File Locking)
在多 Agent 或多终端环境中，极易发生同时尝试加载和写入张量树库 (`.nxpack`) 的冲突。
* 必须在底层封装统一的跨平台 **排他性文件锁 (Exclusive File Lock)** 抽象：
  * **Linux / macOS**: 依赖 `flock` 或 `fcntl`。
  * **Windows**: 依赖 `LockFileEx`。
* 获取不到锁的进程必须实施合理的退让机制 (Exponential Backoff)，或者降级以只读模式启动连接，确保知识库在任何情况下不被写损坏。

## 5. 内存映射文件 (Memory-Mapped Files, Mmap) 跨平台兼容
为实现张量库极速冷启动并节约内存，Nexus-Base 的核心引擎高度依赖 Mmap 技术：
* **Linux / macOS (`mmap` / `munmap`)**：支持零拷贝的数据加载。
* **Windows (`CreateFileMapping` / `MapViewOfFile`)**：通过 Rust 的底层抽象库（如 `memmap2`）隔离底层差异。
* **边界条件**：设计需考虑跨平台在超出系统内存限制时的 Page Fault 处理策略，以及如何优雅卸载不再活跃的区块。

## 6. 硬件架构兼容与指令集加速 (Hardware Architectures & SIMD)
`Nexus-Base` 的张量计算（如 HNSW 图的余弦相似度计算）对 CPU 指令集极为敏感，需要针对不同硬件架构进行差异化的底层优化：
* **x86_64 (Intel / AMD)**：
  * 在 Linux 和 Windows 环境，优先利用 `AVX2` 或 `AVX-512` 指令集实现向量化加速，以成倍缩短高维张量距离的运算耗时。
* **ARM64 (Apple Silicon M1/M2/M3, AWS Graviton 等)**：
  * 在 macOS 与 ARM 架构的 Linux 下，必须适配并切换至 `NEON` 硬件加速指令集。
* **运行时动态分发 (Dynamic Dispatch)**：
  * 为避免编译好的二进制在未支持特定指令集的老旧 CPU 上运行发生 `Illegal Instruction (SIGILL)` 崩溃，Rust 引擎代码必须在运行时检测 CPU Feature。根据检测结果自动动态路由到对应的 SIMD 实现路径，若不支持则安全降级到标量计算 (Scalar Fallback)。

## 7. 基于 Google Cloud 的云端交叉编译方案 (Cross-Compilation via GCP)
由于各平台和架构对底层链接（如 glibc 版本、Windows SDK、macOS SDK）的依赖错综复杂，纯本地环境难以输出全平台产物。我们设计了依托 **Google Cloud Build** 的自动化构建矩阵方案：

* **核心构建矩阵 (Build Matrix)**：
  * `x86_64-unknown-linux-gnu` (主流 Linux)
  * `aarch64-unknown-linux-gnu` (ARM64 Linux)
  * `x86_64-pc-windows-msvc` (Windows 64位)
  * `aarch64-apple-darwin` (macOS Apple Silicon)
  * `x86_64-apple-darwin` (macOS Intel)

* **Cloud Build 执行管线 (Pipeline)**：
  1. **事件触发**：合并至 Main 分支或打 Tag 时自动触发 `cloudbuild.yaml` 工作流。
  2. **基于 Docker 的交叉编译 (针对 Linux)**：
     使用定制的 Rust 编译镜像，利用 `cross` (rust-embedded) 工具集在容器内利用 QEMU 模拟构建跨架构（如 x86_64 编译 aarch64）的 Linux 二进制文件。
  3. **原生环境池 (针对 Windows / macOS)**：
     借助于 Google Cloud 提供的 Windows Server 容器池执行 `msvc` 构建。针对 macOS 则利用 Google Cloud 专属的 Mac 裸金属节点（Bare Metal）或借助第三方 Mac CI Runner，确保 Apple 签名的合法性。
  4. **工件统一分发**：
     构建成功的各平台二进制将被打包压缩（`.tar.gz` / `.zip`），并附带 SHA256 校验和，最终推送到 **Google Cloud Storage (GCS)**。客户端通过 `Install Portal` 或 `Download Center` 基站拉取更新。
