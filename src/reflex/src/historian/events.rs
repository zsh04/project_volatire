// Directive-82: Zero-Copy Binary Events
// All events must be fixed-size to fit into the Ring Buffer slots deterministically.

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum LogEvent {
    MarketTick(MarketTickEvent),
    Signal(SignalEvent),
    Order(OrderEvent),
    Fill(FillEvent),
    Veto(VetoEvent),
    Info(InfoEvent), // Fallback for generic messages (fixed size char array)
    Sentinel(SentinelEvent),
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MarketTickEvent {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SignalEvent {
    pub timestamp: u64,
    pub model_id: u8, // 1=Feynman, 2=Simons, 3=Brain
    pub sentiment: f64, // -1.0 to 1.0
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OrderEvent {
    pub timestamp: u64,
    pub order_id: u64, // Sequence ID
    pub side: u8, // 1=Buy, 2=Sell
    pub price: f64,
    pub qty: f64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FillEvent {
    pub timestamp: u64,
    pub order_id: u64,
    pub fill_price: f64,
    pub fill_qty: f64,
    pub fee: f64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VetoEvent {
    pub timestamp: u64,
    pub module_id: u8, // 1=Risk, 2=Compliance, 3=Origin
    pub reason_code: u16, 
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SentinelEvent {
    pub timestamp: u64,
    pub latency_us: f64,
    pub jitter_us: f64,
    pub status_code: u8, // 0=OK, 1=Degraded, 2=Critical
}

// Fixed size string simulation for Info logs
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InfoEvent {
    pub timestamp: u64,
    pub module_id: u8,
    pub msg_len: u8,
    pub msg: [u8; 32], 
}
