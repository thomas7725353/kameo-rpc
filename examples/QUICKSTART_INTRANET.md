# Kameo 内网示例快速开始

适用于公司内网环境的 Kameo 分布式 Actor 通信示例 - 10 分钟快速上手。

## 特点

✅ **无需服务发现** - 直接 IP:Port 连接，去除 mDNS 和 Gossipsub
✅ **简单配置** - 使用 clap 命令行工具，灵活指定连接参数
✅ **内网友好** - 专为公司内网环境设计，无需复杂的网络配置
✅ **双向通信** - 支持 Client-Server 和点对点通信模式

## 最简单用法

### 终端 1 - 启动服务端

```bash
cargo run --example intranet_server --features remote
```

服务端会监听 `0.0.0.0:8020` (TCP) 和 `0.0.0.0:8021` (QUIC)

### 终端 2 - 启动客户端

```bash
# 如果服务端在本机
cargo run --example intranet_client --features remote -- --server-host 127.0.0.1

# 如果服务端在其他机器（替换为实际 IP）
cargo run --example intranet_client --features remote -- --server-host 192.168.1.100
```

## 预期效果

### 服务端输出
```
🚀 启动 Kameo 内网服务端
📋 节点名称: server
🌐 监听地址: 0.0.0.0:8020 (TCP)
🆔 本地 Peer ID: 12D3KooW...
✅ 开始监听: /ip4/0.0.0.0/tcp/8020
✅ CounterActor 已注册为 'counter_service' (初始值: 0)
🔗 连接建立: 12D3KooW...
[server] 收到来自 client 的增量请求: +10
[server] 当前计数: 10
```

### 客户端输出
```
🚀 启动 Kameo 内网客户端
🎯 目标服务端: 127.0.0.1:8020
🔗 连接建立: 12D3KooW...
========== 第 1 次请求 ==========
📤 发送增量请求: +10
✅ 增量成功！远程节点计数更新为: 10
📊 远程节点当前计数: 10
```

## 常用参数

### 服务端参数

```bash
cargo run --example intranet_server --features remote -- \
  --host 0.0.0.0 \          # 监听地址
  --tcp-port 8020 \         # TCP 端口
  --quic-port 8021 \        # QUIC 端口
  --name "my-server" \      # 节点名称
  --initial-count 0         # 初始计数值
```

### 客户端参数

```bash
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \  # 服务端 IP (必填)
  --server-tcp-port 8020 \       # 服务端端口
  --name "my-client" \           # 客户端名称
  --interval 3                   # 请求间隔（秒）
```

## 查看帮助

```bash
# 服务端帮助
cargo run --example intranet_server --features remote -- --help

# 客户端帮助
cargo run --example intranet_client --features remote -- --help
```

## 多客户端示例

可以同时启动多个客户端连接到同一个服务端：

```bash
# 终端 1 - 服务端
cargo run --example intranet_server --features remote

# 终端 2 - 客户端 1
cargo run --example intranet_client --features remote -- \
  --server-host 127.0.0.1 --name "client-1"

# 终端 3 - 客户端 2
cargo run --example intranet_client --features remote -- \
  --server-host 127.0.0.1 --name "client-2"
```

## 故障排查

### 问题：客户端提示"未找到远程 counter_service"

**解决方法：**
1. 确认服务端正在运行
2. 检查 IP 地址是否正确
3. 测试网络连接：`ping <server_ip>` 或 `telnet <server_ip> 8020`
4. 检查防火墙是否阻止了 8020 端口

### 问题：连接后立即断开

**解决方法：**
1. 检查服务端日志中是否有错误信息
2. 确认客户端和服务端的 Kameo 版本一致
3. 尝试增加 `--interval` 参数的值

## 文件说明

- `examples/intranet_common.rs` - 共享 Actor 和消息定义
- `examples/intranet_server.rs` - 服务端程序
- `examples/intranet_client.rs` - 客户端程序
- `examples/INTRANET_EXAMPLES.md` - 完整使用文档

## 下一步

查看完整文档了解更多功能：
```bash
cat examples/INTRANET_EXAMPLES.md
```

或访问 Kameo 官方文档：https://github.com/tqwewe/kameo
