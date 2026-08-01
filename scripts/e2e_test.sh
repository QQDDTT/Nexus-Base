#!/bin/bash
set -e

echo "==========================================="
echo "   Nexus-Base E2E 联合测试 (Automated)   "
echo "==========================================="

echo ">> 1. 环境清理"
rm -f public.key private.key fake_private.key
rm -f /tmp/nexus_main.log /tmp/nexus_snapshot.nxpack

echo ">> 2. 生成管理密钥 (Keygen)"
cargo run -q -p cli -- keygen

# 故意生成一个假密钥用于异常测试 (非法验签)
mkdir -p fake_keys
cargo run -q -p cli -- keygen --out-dir fake_keys
mv fake_keys/private.key ./fake_private.key
rm -rf fake_keys

echo ">> 3. 启动 Nexus Server (携带公钥)"
cargo run -q -p server &
SERVER_PID=$!
sleep 2

echo ">> 4. 运行基于 JSON 的数据驱动测试用例"
python3 << 'EOF'
import json
import subprocess
import sys

with open("scripts/test_data.json", "r", encoding="utf-8") as f:
    data = json.load(f)

success = True

print("\n--- [Ingest] 写操作边界与安全测试 ---")
for case in data["ingest_cases"]:
    print(f"[*] 正在运行: {case['name']}")
    key_file = "./fake_private.key" if case["use_fake_key"] else "./private.key"
    
    cmd = ["cargo", "run", "-q", "-p", "cli", "--", "ingest", "--text", case["text"], "--key", key_file]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if case["expect_success"]:
        if result.returncode != 0:
            print(f"  ❌ [FAIL] 预期成功，但发生了报错: {result.stderr}")
            success = False
        else:
            print("  ✅ [PASS] 摄入成功")
    else:
        if result.returncode == 0:
            print("  ❌ [FAIL] 预期被安全网关拦截，但竟然成功了！")
            success = False
        else:
            print("  ✅ [PASS] 成功被 AuthInterceptor 安全拦截")

print("\n--- [Retrieve] 读操作查询测试 ---")
for case in data["retrieve_cases"]:
    print(f"[*] 正在运行: {case['name']}")
    cmd = ["cargo", "run", "-q", "-p", "cli", "--", "retrieve", "--query", case["query"], "--top-k", str(case["top_k"])]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"  ❌ [FAIL] 查询失败: {result.stderr}")
        success = False
    else:
        print("  ✅ [PASS] 成功返回结果")

if not success:
    sys.exit(1)
EOF

echo ">> 5. 执行持久化快照 (Commit) - 需要签名"
cargo run -q -p cli -- commit

echo ">> 6. 验证物理快照落盘状态"
if [ -f "/tmp/nexus_snapshot.nxpack" ]; then
    echo "[PASS] 发现物理快照文件: /tmp/nexus_snapshot.nxpack"
else
    echo "[FAIL] 未找到物理快照文件!"
    kill $SERVER_PID
    exit 1
fi

echo ">> 7. 清理并关闭 Server"
kill $SERVER_PID
rm -f fake_private.key

echo "==========================================="
echo "        🎉 联合测试全线通关！🎉          "
echo "==========================================="
