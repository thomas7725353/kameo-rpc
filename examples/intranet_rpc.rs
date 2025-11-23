use futures::StreamExt;
use kameo::prelude::*;
use libp2p::{
    noise, quic, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use std::time::Duration;
use tracing::{error, info, warn};

// ============================================================================
// RPC 框架配置
// ============================================================================

/// RPC 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub tcp_port: u16,
    pub quic_port: u16,
    pub name: String,
    pub idle_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub max_concurrent_streams: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            tcp_port: 8020,
            quic_port: 8021,
            name: "server".to_string(),
            idle_timeout_secs: 300,
            request_timeout_secs: 60,
            max_concurrent_streams: 500,
        }
    }
}

/// RPC 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_host: String,
    pub server_tcp_port: u16,
    pub server_peer_id: Option<String>,
    pub name: String,
    pub request_timeout_secs: u64,
    pub max_concurrent_streams: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_host: "127.0.0.1".to_string(),
            server_tcp_port: 8020,
            server_peer_id: None,
            name: "client".to_string(),
            request_timeout_secs: 60,
            max_concurrent_streams: 500,
        }
    }
}

// ============================================================================
// RPC 网络行为定义
// ============================================================================

/// RPC 服务端网络行为
#[derive(NetworkBehaviour)]
pub struct RpcServerBehaviour {
    pub kameo: remote::Behaviour,
}

/// RPC 客户端网络行为
#[derive(NetworkBehaviour)]
pub struct RpcClientBehaviour {
    pub kameo: remote::Behaviour,
}

// ============================================================================
// RPC 服务器构建器
// ============================================================================

pub struct RpcServer {
    swarm: Swarm<RpcServerBehaviour>,
    config: ServerConfig,
}

impl RpcServer {
    /// 创建新的 RPC 服务器
    pub fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔧 初始化 RPC 服务器");

        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default().port_reuse(true).nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key| {
                let peer_id = key.public().to_peer_id();
                info!("🆔 服务器 Peer ID: {}", peer_id);

                let messaging_config = remote::messaging::Config::default()
                    .with_request_timeout(Duration::from_secs(config.request_timeout_secs))
                    .with_max_concurrent_streams(config.max_concurrent_streams);

                let kameo = remote::Behaviour::new(peer_id, messaging_config);

                Ok(RpcServerBehaviour { kameo })
            })?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(config.idle_timeout_secs))
                    .with_max_negotiating_inbound_streams(1024)
            })
            .build();

        // 初始化 Kameo
        swarm.behaviour().kameo.init_global();

        // 监听地址
        let tcp_addr = format!("/ip4/{}/tcp/{}", config.host, config.tcp_port);
        swarm.listen_on(tcp_addr.parse()?)?;

        let quic_addr = format!("/ip4/{}/udp/{}/quic-v1", config.host, config.quic_port);
        swarm.listen_on(quic_addr.parse()?)?;

        Ok(Self { swarm, config })
    }

    /// 获取本地 Peer ID
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// 获取服务器配置
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// 启动事件循环（后台任务）
    pub fn spawn_event_loop(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.swarm.select_next_some().await {
                    SwarmEvent::Behaviour(RpcServerBehaviourEvent::Kameo(
                        remote::Event::Registry(event),
                    )) => {
                        info!("📝 Registry 事件: {:?}", event);
                    }
                    SwarmEvent::Behaviour(RpcServerBehaviourEvent::Kameo(
                        remote::Event::Messaging(event),
                    )) => {
                        info!("📨 Messaging 事件: {:?}", event);
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("✅ 开始监听: {}", address);
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        info!(
                            "🔗 连接建立: {} via {}",
                            peer_id,
                            endpoint.get_remote_address()
                        );
                    }
                    SwarmEvent::ConnectionClosed {
                        peer_id, cause, ..
                    } => {
                        warn!("❌ 连接关闭: {} 原因: {:?}", peer_id, cause);
                    }
                    SwarmEvent::IncomingConnection { .. } => {
                        info!("📥 收到新连接请求");
                    }
                    SwarmEvent::IncomingConnectionError { error, .. } => {
                        error!("❌ 连接错误: {}", error);
                    }
                    _ => {}
                }
            }
        })
    }
}

// ============================================================================
// RPC 客户端构建器
// ============================================================================

pub struct RpcClient {
    swarm: Swarm<RpcClientBehaviour>,
    config: ClientConfig,
}

impl RpcClient {
    /// 创建新的 RPC 客户端
    pub fn new(config: ClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔧 初始化 RPC 客户端");

        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default().port_reuse(true).nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key| {
                let peer_id = key.public().to_peer_id();
                info!("🆔 客户端 Peer ID: {}", peer_id);

                let messaging_config = remote::messaging::Config::default()
                    .with_request_timeout(Duration::from_secs(config.request_timeout_secs))
                    .with_max_concurrent_streams(config.max_concurrent_streams);

                let kameo = remote::Behaviour::new(peer_id, messaging_config);

                Ok(RpcClientBehaviour { kameo })
            })?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(300))
                    .with_max_negotiating_inbound_streams(1024)
            })
            .build();

        // 初始化 Kameo
        swarm.behaviour().kameo.init_global();

        // 客户端也需要监听以建立双向连接
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // 连接到服务器
        let server_addr: Multiaddr = if let Some(peer_id_str) = &config.server_peer_id {
            format!(
                "/ip4/{}/tcp/{}/p2p/{}",
                config.server_host, config.server_tcp_port, peer_id_str
            )
            .parse()?
        } else {
            format!("/ip4/{}/tcp/{}", config.server_host, config.server_tcp_port).parse()?
        };

        info!("🔌 连接服务器: {}", server_addr);
        swarm.dial(server_addr)?;

        Ok(Self { swarm, config })
    }

    /// 获取本地 Peer ID
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// 获取客户端配置
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// 启动事件循环（后台任务）
    pub fn spawn_event_loop(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.swarm.select_next_some().await {
                    SwarmEvent::Behaviour(RpcClientBehaviourEvent::Kameo(
                        remote::Event::Registry(event),
                    )) => {
                        info!("📝 Registry 事件: {:?}", event);
                    }
                    SwarmEvent::Behaviour(RpcClientBehaviourEvent::Kameo(
                        remote::Event::Messaging(event),
                    )) => {
                        info!("📨 Messaging 事件: {:?}", event);
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("✅ 本地监听: {}", address);
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        info!(
                            "🔗 连接建立: {} via {}",
                            peer_id,
                            endpoint.get_remote_address()
                        );
                    }
                    SwarmEvent::ConnectionClosed {
                        peer_id, cause, ..
                    } => {
                        warn!("❌ 连接关闭: {} 原因: {:?}", peer_id, cause);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        error!("❌ 连接服务器失败 {:?}: {}", peer_id, error);
                    }
                    SwarmEvent::Dialing { peer_id, .. } => {
                        info!("📞 正在拨号: {:?}", peer_id);
                    }
                    _ => {}
                }
            }
        })
    }
}
