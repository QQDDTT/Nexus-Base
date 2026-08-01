#!/bin/bash
set -e

echo "==========================================="
echo "   Nexus-Base 打包管线 (Installers Build)  "
echo "==========================================="

echo ">> 1. 编译全平台 Release 二进制"
cargo build --workspace --release

echo ">> 2. Linux (.deb) 构建"
if ! command -v cargo-deb &> /dev/null; then
    echo ">> [Info] 正在安装 cargo-deb..."
    cargo install cargo-deb
fi

# 在 cli 目录下触发打包，因为配置文件挂载在 cli/Cargo.toml
cd cli
cargo deb --no-build
cd ..

if [ -f "target/debian/"*.deb ]; then
    echo "✅ Linux 打包成功！产物位置: target/debian/"
else
    echo "❌ Linux 打包失败！未找到 .deb 产物"
fi

# 注：Windows msi 和 macOS pkg 通常由跨平台 CI 或原生机器执行
# 这里仅预留输出
echo "==========================================="
echo "        🎉 打包脚本执行完毕！🎉          "
echo "==========================================="
