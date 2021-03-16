use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub type UtcTimeStamp = DateTime<Utc>;
pub type Price = f32;
pub type Quantity = f32;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    pub header: Header,
    pub body: Body,
    pub trailer: Trailer,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Header {
    pub begin_string: BeginString,
    pub body_length: Option<usize>,
    pub msg_type: MessageType,
    #[serde(rename = "SenderCompID")]
    pub sender_comp_id: String,
    #[serde(rename = "TargetCompID")]
    pub target_comp_id: String,
    pub msg_seq_num: u64,
    pub sending_time: UtcTimeStamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "0")]
    HeartBeat,
    #[serde(rename = "1")]
    TestRequest,
    #[serde(rename = "2")]
    ResendRequest,
    #[serde(rename = "3")]
    Reject,
    #[serde(rename = "4")]
    SequenceReset,
    #[serde(rename = "5")]
    Logout,
    #[serde(rename = "6")]
    IndicationOfInterest,
    #[serde(rename = "7")]
    Advertisement,
    #[serde(rename = "8")]
    ExecutionReport,
    #[serde(rename = "9")]
    OrderCancelReject,
    #[serde(rename = "A")]
    Logon,
    #[serde(rename = "B")]
    News,
    #[serde(rename = "C")]
    Email,
    #[serde(rename = "D")]
    NewOrderSingle,
    #[serde(rename = "E")]
    NewOrderList,
    #[serde(rename = "F")]
    OrderCancelRequest,
    #[serde(rename = "G")]
    OrderCancelReplaceRequest,
    #[serde(rename = "H")]
    OrderStatusRequest,
    #[serde(rename = "J")]
    Allocation,
    #[serde(rename = "K")]
    ListCancelRequest,
    #[serde(rename = "L")]
    ListExecute,
    #[serde(rename = "M")]
    ListStatusRequest,
    #[serde(rename = "N")]
    ListStatus,
    #[serde(rename = "P")]
    AllocationAck,
    #[serde(rename = "Q")]
    DontKnowTrade,
    #[serde(rename = "R")]
    QuoteRequest,
    #[serde(rename = "S")]
    Quote,
    #[serde(rename = "T")]
    SettlementInstructions,
    #[serde(rename = "V")]
    MarketDataRequest,
    #[serde(rename = "W")]
    MarketDataSnapshotFullRefresh,
    #[serde(rename = "X")]
    MarketDataIncrementRefresh,
    #[serde(rename = "Y")]
    MarketDataRquestReject,
    #[serde(rename = "Z")]
    QuoteCancel,
    #[serde(rename = "a")]
    QuoteStatusReject,
    #[serde(rename = "b")]
    QuoteAcknowledgement,
    #[serde(rename = "c")]
    SecurityDefinitionRequest,
    #[serde(rename = "d")]
    SecurityDefinition,
    #[serde(rename = "e")]
    SecurityStatusRequest,
    #[serde(rename = "f")]
    SecurityStatus,
    #[serde(rename = "g")]
    TradingSessionStatusRequest,
    #[serde(rename = "h")]
    TradingSessionStatus,
    #[serde(rename = "i")]
    MassQuote,
    #[serde(rename = "j")]
    BusinessMessageReject,
    #[serde(rename = "k")]
    BidRequest,
    #[serde(rename = "l")]
    BidResponse,
    #[serde(rename = "m")]
    ListStrikePrice,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BeginString {
    #[serde(rename = "FIX.4.2")]
    #[allow(non_camel_case_types)]
    Fix_4_4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HandlInst {
    #[serde(rename = "1")]
    AutomatedExecutionOrderPrivateNoBrokerIntervention,
    #[serde(rename = "2")]
    AutomatedExecutionOrderPublicBrokerInterventionOk,
    #[serde(rename = "3")]
    ManualOrderBestExecution,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Body {
    #[serde(rename = "ClOrdID")]
    pub cl_ord_id: Option<String>,
    #[serde(rename = "OrderID")]
    pub order_id: Option<String>,
    pub ord_status: Option<OrderStatus>,
    #[serde(rename = "ExecID")]
    pub exec_id: Option<String>,
    pub exec_trans_type: Option<ExecTransType>,
    pub exec_type: Option<ExecType>,
    pub leaves_qty: Option<Quantity>,
    pub cum_qty: Option<Quantity>,
    pub avg_px: Option<Price>,
    pub symbol: Option<String>,
    pub side: Option<Side>,
    pub transact_time: Option<UtcTimeStamp>,
    pub handl_inst: Option<HandlInst>,
    pub ord_type: Option<OrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_qty: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecTransType {
    #[serde(rename = "0")]
    New,
    #[serde(rename = "1")]
    Cancel,
    #[serde(rename = "2")]
    Correct,
    #[serde(rename = "3")]
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecType {
    #[serde(rename = "0")]
    New,
    #[serde(rename = "1")]
    PartialFill,
    #[serde(rename = "2")]
    Fill,
    #[serde(rename = "3")]
    DoneForDay,
    #[serde(rename = "4")]
    Canceled,
    #[serde(rename = "5")]
    Replaced,
    #[serde(rename = "6")]
    PendingCancel,
    #[serde(rename = "7")]
    Stopped,
    #[serde(rename = "8")]
    Rejected,
    #[serde(rename = "9")]
    Suspended,
    #[serde(rename = "A")]
    PendingNew,
    #[serde(rename = "B")]
    Calculated,
    #[serde(rename = "C")]
    Expired,
    #[serde(rename = "D")]
    Restated,
    #[serde(rename = "E")]
    PendingReplace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    #[serde(rename = "0")]
    New,
    #[serde(rename = "1")]
    PartiallyFilled,
    #[serde(rename = "2")]
    Filled,
    #[serde(rename = "3")]
    DoneForDay,
    #[serde(rename = "4")]
    Canceled,
    #[serde(rename = "5")]
    Replaced,
    #[serde(rename = "6")]
    PendingCancel,
    #[serde(rename = "7")]
    Stopped,
    #[serde(rename = "8")]
    Rejected,
    #[serde(rename = "9")]
    Suspended,
    #[serde(rename = "A")]
    PendingNew,
    #[serde(rename = "B")]
    Calculated,
    #[serde(rename = "C")]
    Expired,
    #[serde(rename = "D")]
    AcceptedForBidding,
    #[serde(rename = "E")]
    PendingReplace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "1")]
    Market,
    #[serde(rename = "2")]
    Limit,
    #[serde(rename = "3")]
    Stop,
    #[serde(rename = "4")]
    StopLimit,
    #[serde(rename = "5")]
    MarketOnClose,
    #[serde(rename = "6")]
    WithOrWithout,
    #[serde(rename = "7")]
    LimitOrBetter,
    #[serde(rename = "8")]
    LimitWithOrWithout,
    #[serde(rename = "9")]
    OnBasis,
    #[serde(rename = "A")]
    OnClose,
    #[serde(rename = "B")]
    LimitOnClose,
    #[serde(rename = "C")]
    ForexMarket,
    #[serde(rename = "D")]
    PreviouslyQuoted,
    #[serde(rename = "E")]
    PreviouslyIndicated,
    #[serde(rename = "F")]
    ForexLimit,
    #[serde(rename = "G")]
    ForexSwap,
    #[serde(rename = "H")]
    ForexPreviouslyQuoted,
    #[serde(rename = "I")]
    Funari,
    #[serde(rename = "P")]
    Pegged,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    #[serde(rename = "1")]
    Buy,
    #[serde(rename = "2")]
    Sell,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Trailer {
    pub signature_length: Option<usize>,
    pub signature: Option<String>,
}
