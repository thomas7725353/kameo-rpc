use kameo::prelude::*;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use tracing::info;

// ============================================================================
// 数学运算服务定义 - 模拟 4 个 gRPC 方法
// ============================================================================

/// 加法运算请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddRequest {
    pub a: f64,
    pub b: f64,
    pub from_peer: PeerId,
    pub from_name: String,
}

/// 减法运算请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubtractRequest {
    pub a: f64,
    pub b: f64,
    pub from_peer: PeerId,
    pub from_name: String,
}

/// 乘法运算请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultiplyRequest {
    pub a: f64,
    pub b: f64,
    pub from_peer: PeerId,
    pub from_name: String,
}

/// 除法运算请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DivideRequest {
    pub a: f64,
    pub b: f64,
    pub from_peer: PeerId,
    pub from_name: String,
}

// 移除自定义响应结构，使用基本类型 tuple
// (result, operation, server_name)
pub type CalcResponse = (f64, String, String);

// ============================================================================
// 计算器服务 Actor
// ============================================================================

/// 计算器服务 - 提供四种基本运算
#[derive(Actor, RemoteActor)]
pub struct CalculatorActor {
    pub server_name: String,
    pub request_count: u64,
}

impl CalculatorActor {
    pub fn new(server_name: String) -> Self {
        Self {
            server_name,
            request_count: 0,
        }
    }
}

// ============================================================================
// 远程消息处理实现 - 加法
// ============================================================================

#[remote_message]
impl Message<AddRequest> for CalculatorActor {
    type Reply = CalcResponse;

    async fn handle(
        &mut self,
        msg: AddRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.request_count += 1;
        let result = msg.a + msg.b;

        info!(
            "[{}] 📥 加法请求 #{} | 来自: {} | {} + {} = {}",
            self.server_name, self.request_count, msg.from_name, msg.a, msg.b, result
        );

        (result, format!("{} + {}", msg.a, msg.b), self.server_name.clone())
    }
}

// ============================================================================
// 远程消息处理实现 - 减法
// ============================================================================

#[remote_message]
impl Message<SubtractRequest> for CalculatorActor {
    type Reply = CalcResponse;

    async fn handle(
        &mut self,
        msg: SubtractRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.request_count += 1;
        let result = msg.a - msg.b;

        info!(
            "[{}] 📥 减法请求 #{} | 来自: {} | {} - {} = {}",
            self.server_name, self.request_count, msg.from_name, msg.a, msg.b, result
        );

        (result, format!("{} - {}", msg.a, msg.b), self.server_name.clone())
    }
}

// ============================================================================
// 远程消息处理实现 - 乘法
// ============================================================================

#[remote_message]
impl Message<MultiplyRequest> for CalculatorActor {
    type Reply = CalcResponse;

    async fn handle(
        &mut self,
        msg: MultiplyRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.request_count += 1;
        let result = msg.a * msg.b;

        info!(
            "[{}] 📥 乘法请求 #{} | 来自: {} | {} × {} = {}",
            self.server_name, self.request_count, msg.from_name, msg.a, msg.b, result
        );

        (result, format!("{} × {}", msg.a, msg.b), self.server_name.clone())
    }
}

// ============================================================================
// 远程消息处理实现 - 除法
// ============================================================================

#[remote_message]
impl Message<DivideRequest> for CalculatorActor {
    type Reply = Option<CalcResponse>;

    async fn handle(
        &mut self,
        msg: DivideRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.request_count += 1;

        // 检查除数是否为零
        if msg.b == 0.0 {
            info!(
                "[{}] ❌ 除法请求 #{} | 来自: {} | {} ÷ {} = 错误（除数为零）",
                self.server_name, self.request_count, msg.from_name, msg.a, msg.b
            );
            return None;
        }

        let result = msg.a / msg.b;

        // 检查结果是否有效
        if result.is_infinite() || result.is_nan() {
            info!(
                "[{}] ❌ 除法请求 #{} | 来自: {} | {} ÷ {} = 错误（无效结果）",
                self.server_name, self.request_count, msg.from_name, msg.a, msg.b
            );
            return None;
        }

        info!(
            "[{}] 📥 除法请求 #{} | 来自: {} | {} ÷ {} = {}",
            self.server_name, self.request_count, msg.from_name, msg.a, msg.b, result
        );

        Some((result, format!("{} ÷ {}", msg.a, msg.b), self.server_name.clone()))
    }
}

// ============================================================================
// 推送通知系统 - 服务器主动推送消息定义
// ============================================================================

/// 服务器状态更新推送
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerStatusUpdate {
    pub timestamp: u64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub active_connections: usize,
    pub uptime_seconds: u64,
}

/// 任务完成通知
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCompletionNotice {
    pub task_id: String,
    pub task_type: String,
    pub result: String,
    pub duration_ms: u64,
    pub timestamp: u64,
}

/// 数据流订阅请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeDataStream {
    pub client_peer: PeerId,
    pub client_name: String,
    pub stream_type: StreamType,
}

/// 数据流类型
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StreamType {
    ServerMetrics,
    CalculationHistory,
    SystemEvents,
}

/// 流式数据项
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamDataItem {
    pub timestamp: u64,
    pub stream_type: StreamType,
    pub data: String,
    pub sequence: u64,
}

/// 事件广播消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventBroadcast {
    pub event_type: String,
    pub message: String,
    pub severity: Severity,
    pub timestamp: u64,
}

/// 事件严重程度
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// 客户端信息
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub peer_id: PeerId,
    pub name: String,
    pub actor_id: ActorId,
    pub subscribed_streams: Vec<StreamType>,
    pub connected_at: std::time::SystemTime,
}

// ============================================================================
// 通知服务 Actor
// ============================================================================

/// 通知服务 - 负责向客户端推送各种类型的通知
#[derive(Actor, RemoteActor)]
pub struct NotificationActor {
    pub server_name: String,
    pub connected_clients: std::collections::HashMap<PeerId, ClientInfo>,
    pub event_count: u64,
    pub start_time: std::time::SystemTime,
}

impl NotificationActor {
    pub fn new(server_name: String) -> Self {
        Self {
            server_name,
            connected_clients: std::collections::HashMap::new(),
            event_count: 0,
            start_time: std::time::SystemTime::now(),
        }
    }
}

// ============================================================================
// 通知服务消息处理 - 订阅数据流
// ============================================================================

#[remote_message]
impl Message<SubscribeDataStream> for NotificationActor {
    type Reply = String; // 返回订阅ID

    async fn handle(
        &mut self,
        msg: SubscribeDataStream,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let subscription_id = format!("sub-{}-{}", msg.client_peer, self.event_count);
        self.event_count += 1;

        // 记录客户端信息
        // 注意: ActorId 从 peer_id 生成,这里简化处理,实际应该从 client 传递 ActorId
        let client_info = ClientInfo {
            peer_id: msg.client_peer,
            name: msg.client_name.clone(),
            actor_id: ActorId::new(0), // 简化:使用占位符
            subscribed_streams: vec![msg.stream_type.clone()],
            connected_at: std::time::SystemTime::now(),
        };

        self.connected_clients.insert(msg.client_peer, client_info);

        info!(
            "[{}] 📡 客户端 '{}' 订阅了数据流: {:?}",
            self.server_name, msg.client_name, msg.stream_type
        );
        info!(
            "[{}] 📊 当前连接客户端数: {}",
            self.server_name,
            self.connected_clients.len()
        );

        subscription_id
    }
}
