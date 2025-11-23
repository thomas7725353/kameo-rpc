# Kameo 内网直连示例

适用于公司内网环境的 Kameo 分布式 Actor 通信示例。不依赖 mDNS、Gossipsub 等发现机制，直接通过 IP:Port 建立连接。

## 文件说明

- `intranet_common.rs` - 共享的 Actor 和消息定义
- `intranet_server.rs` - 服务端程序（监听指定端口）
- `intranet_client.rs` - 客户端程序（连接到服务端）

## 功能特性

### 服务端 (intranet_server)
- ✅ 监听指定 IP 和端口（TCP + QUIC）
- ✅ 提供 CounterActor 服务
- ✅ 定期与远程节点交互
- ✅ 完整的连接状态监控

### 客户端 (intranet_client)
- ✅ 连接到指定服务端 IP:Port
- ✅ 可选启用本地 CounterActor
- ✅ 定期发送增量和查询请求
- ✅ 友好的错误提示

## 快速开始

### 1. 编译项目

```bash
# 编译所有示例
cargo build --examples --features remote

# 或者单独编译
cargo build --example intranet_server --features remote
cargo build --example intranet_client --features remote
```

### 2. 启动服务端

在服务器机器上运行：

```bash
# 基本用法 - 监听所有网卡的 8020 端口
cargo run --example intranet_server --features remote

# 指定监听地址和端口
cargo run --example intranet_server --features remote -- \
  --host 0.0.0.0 \
  --tcp-port 8020 \
  --quic-port 8021 \
  --name "server-node" \
  --initial-count 0

# 监听特定网卡（如内网 IP）
cargo run --example intranet_server --features remote -- \
  --host 192.168.1.100 \
  --tcp-port 8020
```

**服务端参数说明：**
- `--host, -h` - 监听的 IP 地址（默认: 0.0.0.0）
- `--tcp-port, -p` - TCP 端口（默认: 8020）
- `--quic-port, -q` - QUIC(UDP) 端口（默认: 8021）
- `--name, -n` - 节点名称（默认: server）
- `--initial-count` - 计数器初始值（默认: 0）

### 3. 启动客户端

在客户端机器上运行：

```bash
# 基本用法 - 连接到服务端
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100

# 完整配置
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \
  --server-tcp-port 8020 \
  --name "client-node-1" \
  --interval 3

# 启用本地 Counter（可选）
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \
  --enable-local-counter \
  --local-initial-count 100
```

**客户端参数说明：**
- `--server-host, -s` - 服务端 IP 地址（必填）
- `--server-tcp-port, -p` - 服务端 TCP 端口（默认: 8020）
- `--server-peer-id` - 服务端 Peer ID（可选，通常不需要）
- `--name, -n` - 客户端节点名称（默认: client）
- `--interval, -i` - 请求间隔秒数（默认: 3）
- `--enable-local-counter` - 是否启用本地 Counter
- `--local-initial-count` - 本地 Counter 初始值（默认: 100）

## 使用场景

### 场景 1: 简单的 Client-Server 通信

**服务端:**
```bash
cargo run --example intranet_server --features remote -- \
  --name "main-server"
```

**客户端:**
```bash
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \
  --name "client-1"
```

### 场景 2: 多个客户端连接同一服务端

**服务端（一个）:**
```bash
cargo run --example intranet_server --features remote -- \
  --name "central-server"
```

**客户端 1:**
```bash
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \
  --name "client-1"
```

**客户端 2:**
```bash
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \
  --name "client-2"
```

### 场景 3: 双向通信（两个节点都有 Counter）

**节点 A（作为服务端 + 客户端）:**
```bash
# 先启动作为服务端
cargo run --example intranet_server --features remote -- \
  --host 0.0.0.0 \
  --tcp-port 8020 \
  --name "node-a"
```

**节点 B（作为客户端，但也启用本地 Counter）:**
```bash
cargo run --example intranet_client --features remote -- \
  --server-host 192.168.1.100 \
  --name "node-b" \
  --enable-local-counter \
  --local-initial-count 1000
```

这样节点 A 和节点 B 可以相互发送消息。

## 预期输出

### 服务端输出示例

```
🚀 启动 Kameo 内网服务端
📋 节点名称: server
🌐 监听地址: 0.0.0.0:8020 (TCP)
🌐 监听地址: 0.0.0.0:8021 (QUIC/UDP)
🆔 本地 Peer ID: 12D3KooWXXXXXX...
✅ 开始监听: /ip4/192.168.1.100/tcp/8020
✅ CounterActor 已注册为 'counter_service' (初始值: 0)
🔄 每 5s 检查一次远程节点
⏳ 等待客户端连接...
🔗 连接建立: 12D3KooWYYYYYY... via /ip4/192.168.1.101/tcp/xxxxx
[server] 收到来自 client (YYYYYY...) 的增量请求: +10
[server] 当前计数: 10
```

### 客户端输出示例

```
🚀 启动 Kameo 内网客户端
📋 节点名称: client
🎯 目标服务端: 192.168.1.100:8020
🆔 本地 Peer ID: 12D3KooWYYYYYY...
🔌 尝试连接服务端: /ip4/192.168.1.100/tcp/8020
✅ 本地监听地址: /ip4/0.0.0.0/tcp/xxxxx
📞 正在拨号连接: Some(12D3KooWXXXXXX...)
🔗 连接建立: 12D3KooWXXXXXX... via /ip4/192.168.1.100/tcp/8020
🔄 每 3s 与远程节点交互一次
========== 第 1 次请求 ==========
📤 发送增量请求: +10
✅ 增量成功！远程节点计数更新为: 10
📤 查询远程计数
📊 远程节点当前计数: 10
```

## 架构说明

### 网络拓扑

```
┌─────────────────┐           ┌─────────────────┐
│   Server Node   │           │   Client Node   │
│                 │           │                 │
│ CounterActor    │◄─────────►│   (Remote Ref)  │
│ IP: x.x.x.x     │    TCP    │                 │
│ Port: 8020      │           │                 │
└─────────────────┘           └─────────────────┘
```

### 消息流程

1. **连接建立**
   - Client 主动连接 Server 的 IP:Port
   - libp2p 建立加密的双向连接
   - Kameo 初始化远程 Actor 注册表同步

2. **服务发现**
   - Client 通过 `RemoteActorRef::lookup_all("counter_service")` 查找远程 Actor
   - 无需 mDNS 或 Gossipsub，直接通过已建立的连接发现

3. **消息传递**
   - Client 使用 `remote_counter.ask(&Increment{...})` 发送消息
   - Kameo 自动序列化消息（MessagePack）并通过连接发送
   - Server 处理消息并返回结果

## 常见问题

### Q1: 客户端提示 "未找到远程 counter_service"

**可能原因:**
1. 服务端未启动或已崩溃
2. 网络连接问题（防火墙、网段隔离）
3. IP 地址或端口错误

**排查步骤:**
1. 确认服务端正在运行并显示 "✅ CounterActor 已注册"
2. 检查防火墙是否阻止端口（默认 8020）
3. 使用 `telnet server_ip 8020` 测试连接
4. 查看服务端是否显示 "🔗 连接建立"

### Q2: 连接断开后如何处理？

当前实现会自动重试。客户端会持续尝试查找远程 Actor，服务端重启后会自动重新连接。

### Q3: 如何在生产环境使用？

建议配置：
1. 使用固定端口和 IP
2. 配置防火墙规则允许指定端口
3. 启用日志记录（设置 `RUST_LOG=debug`）
4. 考虑添加心跳检测和重连逻辑
5. 实现优雅关闭机制

### Q4: 性能如何？

- TCP 连接延迟：<1ms（局域网）
- QUIC 连接延迟：<2ms（局域网）
- 消息序列化：MessagePack（高效）
- 支持并发请求：默认 500 并发流

## 扩展开发

### 添加新的消息类型

在 `intranet_common.rs` 中添加：

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomMessage {
    pub data: String,
}

#[remote_message]
impl Message<CustomMessage> for CounterActor {
    type Reply = String;

    async fn handle(
        &mut self,
        msg: CustomMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        format!("处理了: {}", msg.data)
    }
}
```

### 添加新的 Actor

```rust
#[derive(Actor, RemoteActor)]
pub struct MyActor {
    // fields
}

// 实现消息处理
#[remote_message]
impl Message<YourMessage> for MyActor {
    // ...
}

// 在 server/client 中注册
let actor = MyActor::spawn(...);
actor.register("my_service").await?;
```

## 技术细节

### 使用的技术栈

- **Kameo** - Actor 框架
- **libp2p** - P2P 网络库
- **Noise** - 加密传输协议
- **Yamux** - 流多路复用
- **QUIC** - UDP 传输（可选）
- **MessagePack** - 消息序列化

### 端口说明

- **8020/TCP** - 默认 TCP 传输端口
- **8021/UDP** - 默认 QUIC 传输端口

你可以通过命令行参数自定义这些端口。

## 日志控制

使用环境变量控制日志级别：

```bash
# 查看所有日志
RUST_LOG=debug cargo run --example intranet_server --features remote

# 只看 info 级别
RUST_LOG=info cargo run --example intranet_server --features remote

# 只看 kameo 的日志
RUST_LOG=kameo=debug cargo run --example intranet_server --features remote
```

## 许可证

遵循 Kameo 项目的 MIT OR Apache-2.0 许可证。
