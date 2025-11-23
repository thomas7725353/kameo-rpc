mod intranet_common;
mod intranet_rpc;

use clap::Parser;
use intranet_common::*;
use intranet_rpc::{RpcServer, ServerConfig};
use kameo::prelude::*;
use tracing::info;
use tracing_subscriber::EnvFilter;

// 客户端通知处理器的前置声明(将在 client 中定义)
// 这里我们使用 RemoteActorRef 通过名称查找,所以需要一个占位符类型
#[derive(Actor, RemoteActor)]
pub struct ClientNotificationHandler {
    pub client_name: String,
}

impl ClientNotificationHandler {
    pub fn new(client_name: String) -> Self {
        Self { client_name }
    }
}

// 为 ClientNotificationHandler 实现消息处理
// 注意: 这些实现实际上应该在 client 端,这里仅用于类型完整性
#[remote_message]
impl Message<ServerStatusUpdate> for ClientNotificationHandler {
    type Reply = ();
    async fn handle(&mut self, _msg: ServerStatusUpdate, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {}
}

#[remote_message]
impl Message<TaskCompletionNotice> for ClientNotificationHandler {
    type Reply = ();
    async fn handle(&mut self, _msg: TaskCompletionNotice, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {}
}

#[remote_message]
impl Message<EventBroadcast> for ClientNotificationHandler {
    type Reply = ();
    async fn handle(&mut self, _msg: EventBroadcast, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {}
}

// ============================================================================
// 命令行参数定义
// ============================================================================

/// Kameo RPC 计算器服务端
#[derive(Parser, Debug)]
#[command(name = "Kameo Calculator Server")]
#[command(version = "1.0")]
#[command(about = "提供加减乘除四种运算服务的 RPC 服务器", long_about = None)]
struct Args {
    /// 监听的 IP 地址
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// 监听的 TCP 端口
    #[arg(short = 'p', long, default_value = "8020")]
    tcp_port: u16,

    /// 监听的 QUIC 端口 (UDP)
    #[arg(short = 'q', long, default_value = "8021")]
    quic_port: u16,

    /// 服务器名称（用于标识）
    #[arg(short, long, default_value = "calc-server")]
    name: String,

    /// 空闲连接超时（秒）
    #[arg(long, default_value = "300")]
    idle_timeout: u64,

    /// 请求超时（秒）
    #[arg(long, default_value = "60")]
    request_timeout: u64,

    /// 最大并发流数量
    #[arg(long, default_value = "500")]
    max_streams: usize,
}

// ============================================================================
// 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    let args = Args::parse();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    print_banner(&args);

    // 创建服务器配置
    let config = ServerConfig {
        host: args.host.clone(),
        tcp_port: args.tcp_port,
        quic_port: args.quic_port,
        name: args.name.clone(),
        idle_timeout_secs: args.idle_timeout,
        request_timeout_secs: args.request_timeout,
        max_concurrent_streams: args.max_streams,
    };

    // 创建并启动 RPC 服务器
    let server = RpcServer::new(config)?;
    let _local_peer_id = server.local_peer_id();

    // 启动网络事件循环
    let _event_loop_handle = server.spawn_event_loop();

    // 等待一小段时间让服务器完全初始化
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 注册计算器服务
    register_calculator_service(&args.name).await?;

    // 注册通知推送服务
    let notification_ref = register_notification_service(&args.name).await?;

    // 启动推送服务
    start_push_services(notification_ref, args.name.clone(), _local_peer_id);

    // 保持服务运行
    info!("✅ 服务器已就绪，等待客户端请求...");
    info!("按 Ctrl+C 停止服务");

    // 阻塞主线程
    tokio::signal::ctrl_c().await?;
    info!("🛑 收到停止信号，正在关闭服务器...");

    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 打印启动横幅
fn print_banner(args: &Args) {
    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║          🧮 Kameo RPC 计算器服务器 v1.0                   ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("📋 服务器名称: {}", args.name);
    info!("🌐 监听地址:");
    info!("   - TCP:  {}:{}", args.host, args.tcp_port);
    info!("   - QUIC: {}:{} (UDP)", args.host, args.quic_port);
    info!("⚙️  配置:");
    info!("   - 空闲超时: {}s", args.idle_timeout);
    info!("   - 请求超时: {}s", args.request_timeout);
    info!("   - 最大并发流: {}", args.max_streams);
    info!("════════════════════════════════════════════════════════════");
}

/// 注册计算器服务
async fn register_calculator_service(server_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("📝 正在注册计算器服务...");

    // 创建并启动 CalculatorActor
    let calculator = CalculatorActor::new(server_name.to_string());
    let calculator_ref = CalculatorActor::spawn(calculator);

    // 注册为远程服务
    calculator_ref.register("calculator").await?;

    info!("✅ 计算器服务已注册为 'calculator'");
    info!("   - 支持的操作:");
    info!("     • 加法 (Add)");
    info!("     • 减法 (Subtract)");
    info!("     • 乘法 (Multiply)");
    info!("     • 除法 (Divide)");

    Ok(())
}

/// 注册通知推送服务
async fn register_notification_service(server_name: &str) -> Result<ActorRef<NotificationActor>, Box<dyn std::error::Error>> {
    info!("📝 正在注册通知推送服务...");

    // 创建并启动 NotificationActor
    let notification = NotificationActor::new(server_name.to_string());
    let notification_ref = NotificationActor::spawn(notification);

    // 注册为远程服务
    notification_ref.register("notification").await?;

    info!("✅ 通知推送服务已注册为 'notification'");
    info!("   - 支持的推送类型:");
    info!("     • 服务器状态更新 (ServerStatusUpdate)");
    info!("     • 任务完成通知 (TaskCompletionNotice)");
    info!("     • 实时数据流 (StreamDataItem)");
    info!("     • 系统事件广播 (EventBroadcast)");

    Ok(notification_ref)
}

/// 启动推送服务（后台任务）
fn start_push_services(
    notification_ref: ActorRef<NotificationActor>,
    server_name: String,
    _local_peer_id: libp2p::PeerId,
) {
    info!("📡 启动推送服务...");

    // 任务1: 定期推送服务器状态(每5秒)
    let notification_ref_clone = notification_ref.clone();
    let server_name_clone = server_name.clone();
    tokio::spawn(async move {
        push_server_status_loop(notification_ref_clone, server_name_clone).await;
    });

    // 任务2: 模拟任务完成通知(每10秒)
    let notification_ref_clone = notification_ref.clone();
    let server_name_clone = server_name.clone();
    tokio::spawn(async move {
        push_task_completion_loop(notification_ref_clone, server_name_clone).await;
    });

    // 任务3: 模拟系统事件广播(每15秒)
    tokio::spawn(async move {
        broadcast_system_events_loop(notification_ref, server_name).await;
    });

    info!("✅ 推送服务已启动");
}

/// 定期推送服务器状态
async fn push_server_status_loop(
    _notification_ref: ActorRef<NotificationActor>,
    server_name: String,
) {
    use rand::Rng;

    // 等待5秒让客户端有时间连接
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("[{}] 🔄 服务器状态推送循环已启动", server_name);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // 生成模拟的服务器状态
        let (cpu_usage, memory_usage) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (rng.gen_range(20.0..80.0), rng.gen_range(40.0..75.0))
        };

        let status = ServerStatusUpdate {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            cpu_usage,
            memory_usage,
            active_connections: 1, // 简化
            uptime_seconds: 0,
        };

        info!(
            "[{}] 📤 推送服务器状态: CPU {:.1}%, 内存 {:.1}%",
            server_name, status.cpu_usage, status.memory_usage
        );

        // 尝试查找并推送到客户端通知处理器
        // 简化版:尝试推送到已知的客户端handler名称
        let handler_names = vec!["client_handler"];
        for handler_name in handler_names {
            if let Ok(Some(client_handler)) = RemoteActorRef::<ClientNotificationHandler>::lookup(handler_name.to_string()).await {
                let _ = client_handler.tell(&status).send();
            }
        }
    }
}

/// 模拟任务完成通知
async fn push_task_completion_loop(
    _notification_ref: ActorRef<NotificationActor>,
    server_name: String,
) {
    use rand::Rng;
    let mut task_counter = 1u64;

    // 等待8秒让客户端有时间连接
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    info!("[{}] 🔄 任务完成通知循环已启动", server_name);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let task_types = vec!["计算任务", "数据处理", "文件上传", "报告生成"];
        let (task_type_idx, duration_ms) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (rng.gen_range(0..task_types.len()), rng.gen_range(100..5000))
        };
        let task_type = task_types[task_type_idx];

        let notice = TaskCompletionNotice {
            task_id: format!("task-{:04}", task_counter),
            task_type: task_type.to_string(),
            result: "成功完成".to_string(),
            duration_ms,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        info!(
            "[{}] 📤 推送任务完成通知: {} ({})",
            server_name, notice.task_id, notice.task_type
        );

        // 尝试推送到客户端
        let handler_names = vec!["client_handler"];
        for handler_name in handler_names {
            if let Ok(Some(client_handler)) = RemoteActorRef::<ClientNotificationHandler>::lookup(handler_name.to_string()).await {
                let _ = client_handler.tell(&notice).send();
            }
        }

        task_counter += 1;
    }
}

/// 模拟系统事件广播
async fn broadcast_system_events_loop(
    _notification_ref: ActorRef<NotificationActor>,
    server_name: String,
) {
    use rand::Rng;

    // 等待12秒让客户端有时间连接
    tokio::time::sleep(std::time::Duration::from_secs(12)).await;

    info!("[{}] 🔄 系统事件广播循环已启动", server_name);

    let event_types = vec![
        ("系统启动", Severity::Info),
        ("高负载警告", Severity::Warning),
        ("服务健康检查", Severity::Info),
        ("配置更新", Severity::Info),
        ("性能优化", Severity::Info),
    ];

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;

        let event_idx = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen_range(0..event_types.len())
        };
        let (event_type, severity) = &event_types[event_idx];

        let event = EventBroadcast {
            event_type: event_type.to_string(),
            message: format!("{} 事件已触发", event_type),
            severity: severity.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let severity_icon = match event.severity {
            Severity::Info => "ℹ️",
            Severity::Warning => "⚠️",
            Severity::Error => "❌",
        };

        info!(
            "[{}] 📢 广播系统事件: {} {}",
            server_name, severity_icon, event.event_type
        );

        // 尝试推送到客户端
        let handler_names = vec!["client_handler"];
        for handler_name in handler_names {
            if let Ok(Some(client_handler)) = RemoteActorRef::<ClientNotificationHandler>::lookup(handler_name.to_string()).await {
                let _ = client_handler.tell(&event).send();
            }
        }
    }
}
