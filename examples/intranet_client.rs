mod intranet_common;
mod intranet_rpc;

use clap::Parser;
use futures::TryStreamExt;
use intranet_common::*;
use intranet_rpc::{ClientConfig, RpcClient};
use kameo::prelude::*;
use libp2p::PeerId;
use std::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// ============================================================================
// 命令行参数定义
// ============================================================================

/// Kameo RPC 计算器客户端
#[derive(Parser, Debug)]
#[command(name = "Kameo Calculator Client")]
#[command(version = "1.0")]
#[command(about = "调用远程计算器服务进行加减乘除运算", long_about = None)]
struct Args {
    /// 要连接的服务端 IP 地址
    #[arg(short = 's', long)]
    server_host: String,

    /// 服务端 TCP 端口
    #[arg(short = 'p', long, default_value = "8020")]
    server_port: u16,

    /// 客户端节点名称
    #[arg(short, long, default_value = "calc-client")]
    name: String,

    /// 请求间隔（秒）
    #[arg(short, long, default_value = "3")]
    interval: u64,

    /// 请求超时（秒）
    #[arg(long, default_value = "60")]
    request_timeout: u64,

    /// 演示模式：执行预定义的计算示例
    #[arg(long, default_value = "true")]
    demo_mode: bool,
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

    // 创建客户端配置
    let config = ClientConfig {
        server_host: args.server_host.clone(),
        server_tcp_port: args.server_port,
        server_peer_id: None,
        name: args.name.clone(),
        request_timeout_secs: args.request_timeout,
        max_concurrent_streams: 500,
    };

    // 创建并启动 RPC 客户端
    let client = RpcClient::new(config)?;
    let local_peer_id = client.local_peer_id();

    // 启动网络事件循环
    let _event_loop_handle = client.spawn_event_loop();

    // 等待连接建立
    info!("⏳ 等待连接建立...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 注册客户端通知处理器并订阅推送服务
    let _notification_handler = subscribe_to_push_services(&args.name, local_peer_id).await?;

    // 运行客户端逻辑
    if args.demo_mode {
        run_demo_mode(&args, local_peer_id).await?;
    } else {
        run_interactive_mode(&args, local_peer_id).await?;
    }

    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 打印启动横幅
fn print_banner(args: &Args) {
    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║          🧮 Kameo RPC 计算器客户端 v1.0                   ║");
    info!("╚════════════════════════════════════════════════════════════╝");
    info!("📋 客户端名称: {}", args.name);
    info!("🎯 目标服务器: {}:{}", args.server_host, args.server_port);
    info!("⚙️  配置:");
    info!("   - 请求间隔: {}s", args.interval);
    info!("   - 请求超时: {}s", args.request_timeout);
    info!("   - 运行模式: {}", if args.demo_mode { "演示模式" } else { "交互模式" });
    info!("════════════════════════════════════════════════════════════");
}

/// 演示模式 - 自动执行预定义的计算
async fn run_demo_mode(args: &Args, local_peer_id: PeerId) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎬 启动演示模式");
    info!("🔄 每 {}s 执行一轮计算", args.interval);

    let interval = Duration::from_secs(args.interval);
    let mut round = 0u64;

    // 预定义的计算示例
    let calculations = vec![
        ("加法", 15.0, 25.0),
        ("减法", 100.0, 35.0),
        ("乘法", 12.5, 8.0),
        ("除法", 144.0, 12.0),
        ("除法", 100.0, 0.0), // 故意触发除零错误
    ];

    loop {
        tokio::time::sleep(interval).await;
        round += 1;

        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  第 {} 轮计算", round);
        info!("╚══════════════════════════════════════════════════════════╝");

        // 查找远程计算器服务
        let calculator = match find_calculator_service(local_peer_id).await {
            Some(calc) => calc,
            None => {
                warn!("⚠️  未找到远程计算器服务，请检查服务器是否运行");
                continue;
            }
        };

        // 执行所有计算
        for (i, (op_name, a, b)) in calculations.iter().enumerate() {
            info!("────────────────────────────────────────────────────────");
            info!("📊 示例 {}/{}: {}", i + 1, calculations.len(), op_name);

            match op_name {
                &"加法" => execute_add(&calculator, *a, *b, &args.name, local_peer_id).await,
                &"减法" => execute_subtract(&calculator, *a, *b, &args.name, local_peer_id).await,
                &"乘法" => execute_multiply(&calculator, *a, *b, &args.name, local_peer_id).await,
                &"除法" => execute_divide(&calculator, *a, *b, &args.name, local_peer_id).await,
                _ => {}
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        info!("════════════════════════════════════════════════════════════");
    }
}

/// 交互模式 - 等待用户输入（未实现）
async fn run_interactive_mode(_args: &Args, _local_peer_id: PeerId) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎮 交互模式暂未实现");
    info!("💡 提示: 使用 --demo-mode true 启动演示模式");
    Ok(())
}

/// 查找远程计算器服务
async fn find_calculator_service(local_peer_id: PeerId) -> Option<RemoteActorRef<CalculatorActor>> {
    let mut calculators = RemoteActorRef::<CalculatorActor>::lookup_all("calculator");

    while let Ok(Some(calculator)) = calculators.try_next().await {
        // 跳过本地服务（如果有）
        if calculator.id().peer_id() == Some(&local_peer_id) {
            continue;
        }

        info!("✅ 找到远程计算器服务");
        return Some(calculator);
    }

    None
}

/// 订阅服务器的推送服务
async fn subscribe_to_push_services(
    client_name: &str,
    local_peer_id: PeerId,
) -> Result<ActorRef<ClientNotificationHandler>, Box<dyn std::error::Error>> {
    info!("📡 正在订阅服务器推送服务...");

    // 1. 创建并启动客户端通知处理器
    let handler = ClientNotificationHandler::new(client_name.to_string());
    let handler_ref = ClientNotificationHandler::spawn(handler);

    // 2. 注册为远程服务（使用固定名称以便服务器能找到）
    handler_ref.register("client_handler").await?;

    info!("✅ 客户端通知处理器已注册为 'client_handler'");

    // 3. 查找服务器的 NotificationActor
    info!("🔍 正在查找服务器的通知服务...");
    let notification_actor = match RemoteActorRef::<NotificationActor>::lookup("notification").await? {
        Some(actor) => {
            info!("✅ 找到服务器通知服务");
            actor
        }
        None => {
            warn!("⚠️  未找到服务器通知服务,推送功能将不可用");
            return Ok(handler_ref);
        }
    };

    // 4. 订阅实时数据流
    info!("📝 正在订阅数据流...");
    let subscription_id = notification_actor
        .ask(&SubscribeDataStream {
            client_peer: local_peer_id,
            client_name: client_name.to_string(),
            stream_type: StreamType::ServerMetrics,
        })
        .await?;

    info!("✅ 成功订阅推送服务");
    info!("   订阅ID: {}", subscription_id);
    info!("   数据流类型: ServerMetrics");
    info!("════════════════════════════════════════════════════════════");

    Ok(handler_ref)
}

// ============================================================================
// 计算操作函数
// ============================================================================

/// 执行加法运算
async fn execute_add(
    calculator: &RemoteActorRef<CalculatorActor>,
    a: f64,
    b: f64,
    client_name: &str,
    peer_id: PeerId,
) {
    info!("➕ 加法: {} + {}", a, b);

    match calculator
        .ask(&AddRequest {
            a,
            b,
            from_peer: peer_id,
            from_name: client_name.to_string(),
        })
        .await
    {
        Ok((result, operation, server_name)) => {
            info!(
                "   ✅ 结果: {} = {} (来自: {})",
                operation, result, server_name
            );
        }
        Err(err) => {
            error!("   ❌ 加法运算失败: {}", err);
        }
    }
}

/// 执行减法运算
async fn execute_subtract(
    calculator: &RemoteActorRef<CalculatorActor>,
    a: f64,
    b: f64,
    client_name: &str,
    peer_id: PeerId,
) {
    info!("➖ 减法: {} - {}", a, b);

    match calculator
        .ask(&SubtractRequest {
            a,
            b,
            from_peer: peer_id,
            from_name: client_name.to_string(),
        })
        .await
    {
        Ok((result, operation, server_name)) => {
            info!(
                "   ✅ 结果: {} = {} (来自: {})",
                operation, result, server_name
            );
        }
        Err(err) => {
            error!("   ❌ 减法运算失败: {}", err);
        }
    }
}

/// 执行乘法运算
async fn execute_multiply(
    calculator: &RemoteActorRef<CalculatorActor>,
    a: f64,
    b: f64,
    client_name: &str,
    peer_id: PeerId,
) {
    info!("✖️  乘法: {} × {}", a, b);

    match calculator
        .ask(&MultiplyRequest {
            a,
            b,
            from_peer: peer_id,
            from_name: client_name.to_string(),
        })
        .await
    {
        Ok((result, operation, server_name)) => {
            info!(
                "   ✅ 结果: {} = {} (来自: {})",
                operation, result, server_name
            );
        }
        Err(err) => {
            error!("   ❌ 乘法运算失败: {}", err);
        }
    }
}

/// 执行除法运算
async fn execute_divide(
    calculator: &RemoteActorRef<CalculatorActor>,
    a: f64,
    b: f64,
    client_name: &str,
    peer_id: PeerId,
) {
    info!("➗ 除法: {} ÷ {}", a, b);

    match calculator
        .ask(&DivideRequest {
            a,
            b,
            from_peer: peer_id,
            from_name: client_name.to_string(),
        })
        .await
    {
        Ok(result) => match result {
            Some((value, operation, server_name)) => {
                info!(
                    "   ✅ 结果: {} = {} (来自: {})",
                    operation, value, server_name
                );
            }
            None => {
                warn!("   ⚠️  除法运算失败：除数为零或结果无效");
            }
        },
        Err(err) => {
            error!("   ❌ 除法运算失败: {}", err);
        }
    }
}

// ============================================================================
// 客户端通知处理器 - 接收服务器推送
// ============================================================================

/// 客户端通知处理器 Actor - 接收服务器的各种推送通知
#[derive(Actor, RemoteActor)]
pub struct ClientNotificationHandler {
    pub client_name: String,
    pub notification_count: u64,
}

impl ClientNotificationHandler {
    pub fn new(client_name: String) -> Self {
        Self {
            client_name,
            notification_count: 0,
        }
    }
}

// ============================================================================
// 消息处理实现 - 服务器状态更新
// ============================================================================

#[remote_message]
impl Message<ServerStatusUpdate> for ClientNotificationHandler {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ServerStatusUpdate,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.notification_count += 1;

        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  📊 服务器状态推送 #{}  ", self.notification_count);
        info!("╚══════════════════════════════════════════════════════════╝");
        info!("   🖥️  CPU 使用率: {:.1}%", msg.cpu_usage);
        info!("   💾 内存使用率: {:.1}%", msg.memory_usage);
        info!("   🔗 活跃连接数: {}", msg.active_connections);
        info!("   ⏱️  运行时间: {}s", msg.uptime_seconds);
        info!("════════════════════════════════════════════════════════════");
    }
}

// ============================================================================
// 消息处理实现 - 任务完成通知
// ============================================================================

#[remote_message]
impl Message<TaskCompletionNotice> for ClientNotificationHandler {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: TaskCompletionNotice,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.notification_count += 1;

        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  ✅ 任务完成通知 #{}  ", self.notification_count);
        info!("╚══════════════════════════════════════════════════════════╝");
        info!("   🆔 任务ID: {}", msg.task_id);
        info!("   📦 任务类型: {}", msg.task_type);
        info!("   📝 执行结果: {}", msg.result);
        info!("   ⏱️  耗时: {}ms", msg.duration_ms);
        info!("════════════════════════════════════════════════════════════");
    }
}

// ============================================================================
// 消息处理实现 - 系统事件广播
// ============================================================================

#[remote_message]
impl Message<EventBroadcast> for ClientNotificationHandler {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: EventBroadcast,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.notification_count += 1;

        let severity_icon = match msg.severity {
            Severity::Info => "ℹ️ ",
            Severity::Warning => "⚠️ ",
            Severity::Error => "❌",
        };

        let severity_text = match msg.severity {
            Severity::Info => "信息",
            Severity::Warning => "警告",
            Severity::Error => "错误",
        };

        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  {} 系统事件广播 #{}  ", severity_icon, self.notification_count);
        info!("╚══════════════════════════════════════════════════════════╝");
        info!("   🏷️  事件类型: {}", msg.event_type);
        info!("   📢 事件消息: {}", msg.message);
        info!("   🔴 严重程度: {}", severity_text);
        info!("════════════════════════════════════════════════════════════");
    }
}

// ============================================================================
// 消息处理实现 - 流式数据项
// ============================================================================

#[remote_message]
impl Message<StreamDataItem> for ClientNotificationHandler {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: StreamDataItem,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.notification_count += 1;

        let stream_icon = match msg.stream_type {
            StreamType::ServerMetrics => "📊",
            StreamType::CalculationHistory => "🧮",
            StreamType::SystemEvents => "🔔",
        };

        info!(
            "{} 流式数据 #{}: {} (序列: {})",
            stream_icon, self.notification_count, msg.data, msg.sequence
        );
    }
}
