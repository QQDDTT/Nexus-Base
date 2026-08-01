# 🧠 Nexus-Base

**The High-Performance Personal Vector Tensor Engine**

专属极客开发者的长尾经验挂载点。由原 `Aura Substrate` 剥离前端冗余后的纯血 Rust 张量检索核心。
支持热快照、冷加载、防穿透 gRPC 通信。

## 📖 架构设计索引 (Design Index)

所有的算法原理与数据结构契约详见 `docs/`：

- [01_Architecture (元数据锚点与检索漏斗)](docs/01_Architecture/01_TENSOR_ENGINE.md)
- [02_Data_Flow (永续树缓存快照机制)](docs/02_Data_Flow/01_PERSISTENT_CACHE.md)
- [03_Interfaces (gRPC 安全通信契约)](docs/03_Interfaces/01_AGENT_CONNECTION.md)

## 🚀 编译与挂载 (Mounting)

> [!TIP]
> `Nexus-Base` 不对外提供 HTTP 端口，仅通过 Unix Domain Socket 或 gRPC 响应 `Agent_Platform` 的 IPC 请求。

```bash
# 1. 编译张量检索守护进程
cargo build --release --features="avx2,zstd"

# 2. 从 Web_Matrix 下载基站极速拉取最新张量快照
curl -sL https://matrix.local/api/sync_snapshot > ~/.nexus/latest.nxpack

# 3. 挂载快照并拉起服务
./target/release/nexus_daemon --load ~/.nexus/latest.nxpack --socket /tmp/nexus.sock
```

---
*Built for the Divergent Production Architecture.*
