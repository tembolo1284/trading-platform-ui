use bytes::{Buf, BufMut, BytesMut};
use std::io;

/// Protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Message types matching the C++ protocol
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    // Client → Engine
    NewOrder = 0x01,
    CancelOrder = 0x02,
    ReplaceOrder = 0x03,
    
    // Engine → Client
    OrderAck = 0x10,
    OrderReject = 0x11,
    OrderCancelled = 0x12,
    OrderReplaced = 0x13,
    
    // Executions
    Execution = 0x20,
    
    // Market Data
    Trade = 0x30,
    Quote = 0x31,
    
    // System
    Heartbeat = 0xF0,
    Logon = 0xF1,
    Logout = 0xF2,
}

impl TryFrom<u8> for MessageType {
    type Error = io::Error;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageType::NewOrder),
            0x02 => Ok(MessageType::CancelOrder),
            0x03 => Ok(MessageType::ReplaceOrder),
            0x10 => Ok(MessageType::OrderAck),
            0x11 => Ok(MessageType::OrderReject),
            0x12 => Ok(MessageType::OrderCancelled),
            0x13 => Ok(MessageType::OrderReplaced),
            0x20 => Ok(MessageType::Execution),
            0x30 => Ok(MessageType::Trade),
            0x31 => Ok(MessageType::Quote),
            0xF0 => Ok(MessageType::Heartbeat),
            0xF1 => Ok(MessageType::Logon),
            0xF2 => Ok(MessageType::Logout),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown message type: 0x{:02x}", value),
            )),
        }
    }
}

/// Order side
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy = 0x01,
    Sell = 0x02,
}

/// Order type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit = 0x01,
    Market = 0x02,
}

/// Message header (16 bytes)
#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub version: u8,
    pub msg_type: MessageType,
    pub reserved: u16,
    pub length: u32,
    pub sequence: u64,
}

impl MessageHeader {
    pub fn new(msg_type: MessageType, length: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            reserved: 0,
            length,
            sequence: 0,
        }
    }
    
    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.version);
        buf.put_u8(self.msg_type as u8);
        buf.put_u16(self.reserved);
        buf.put_u32(self.length);
        buf.put_u64(self.sequence);
    }
    
    pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        if buf.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data for header",
            ));
        }
        
        let version = buf.get_u8();
        let msg_type = MessageType::try_from(buf.get_u8())?;
        let reserved = buf.get_u16();
        let length = buf.get_u32();
        let sequence = buf.get_u64();
        
        Ok(Self {
            version,
            msg_type,
            reserved,
            length,
            sequence,
        })
    }
}

/// New Order Message
#[derive(Debug, Clone)]
pub struct NewOrderMessage {
    pub header: MessageHeader,
    pub symbol: String,
    pub client_order_id: u64,
    pub user_id: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub price: u64,      // Price in cents (fixed-point)
    pub quantity: u64,
    pub timestamp: u64,
}

impl NewOrderMessage {
    pub fn new(
        symbol: String,
        client_order_id: u64,
        user_id: u64,
        side: Side,
        order_type: OrderType,
        price: u64,
        quantity: u64,
    ) -> Self {
        Self {
            header: MessageHeader::new(MessageType::NewOrder, 88), // Fixed size
            symbol,
            client_order_id,
            user_id,
            side,
            order_type,
            price,
            quantity,
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
        }
    }
    
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(88);
        
        // Header
        self.header.encode(&mut buf);
        
        // Symbol (16 bytes, null-padded)
        let mut symbol_bytes = [0u8; 16];
        let symbol_len = self.symbol.len().min(15);
        symbol_bytes[..symbol_len].copy_from_slice(&self.symbol.as_bytes()[..symbol_len]);
        buf.put_slice(&symbol_bytes);
        
        // Fields
        buf.put_u64(self.client_order_id);
        buf.put_u64(self.user_id);
        buf.put_u8(self.side as u8);
        buf.put_u8(self.order_type as u8);
        buf.put_u16(0); // reserved
        buf.put_u64(self.price);
        buf.put_u64(self.quantity);
        buf.put_u64(self.timestamp);
        
        buf
    }
}

/// Cancel Order Message
#[derive(Debug, Clone)]
pub struct CancelOrderMessage {
    pub header: MessageHeader,
    pub symbol: String,
    pub client_order_id: u64,
    pub user_id: u64,
    pub timestamp: u64,
}

impl CancelOrderMessage {
    pub fn new(symbol: String, client_order_id: u64, user_id: u64) -> Self {
        Self {
            header: MessageHeader::new(MessageType::CancelOrder, 56), // Fixed size
            symbol,
            client_order_id,
            user_id,
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
        }
    }
    
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(56);
        
        // Header
        self.header.encode(&mut buf);
        
        // Symbol (16 bytes, null-padded)
        let mut symbol_bytes = [0u8; 16];
        let symbol_len = self.symbol.len().min(15);
        symbol_bytes[..symbol_len].copy_from_slice(&self.symbol.as_bytes()[..symbol_len]);
        buf.put_slice(&symbol_bytes);
        
        // Fields
        buf.put_u64(self.client_order_id);
        buf.put_u64(self.user_id);
        buf.put_u64(self.timestamp);
        
        buf
    }
}

/// Order Acknowledgement
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OrderAckMessage {
    pub client_order_id: u64,
    pub exchange_order_id: u64,
    pub user_id: u64,
    pub timestamp: u64,
}

impl OrderAckMessage {
    pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        if buf.len() < 32 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data for OrderAck",
            ));
        }
        
        Ok(Self {
            client_order_id: buf.get_u64(),
            exchange_order_id: buf.get_u64(),
            user_id: buf.get_u64(),
            timestamp: buf.get_u64(),
        })
    }
}

/// Order Reject
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OrderRejectMessage {
    pub client_order_id: u64,
    pub user_id: u64,
    pub reason: u8,
    pub text: String,
    pub timestamp: u64,
}

impl OrderRejectMessage {
    pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        if buf.len() < 88 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data for OrderReject",
            ));
        }
        
        let client_order_id = buf.get_u64();
        let user_id = buf.get_u64();
        let reason = buf.get_u8();
        
        // Skip reserved bytes
        buf.advance(7);
        
        // Read text (64 bytes, null-terminated)
        let mut text_bytes = [0u8; 64];
        buf.copy_to_slice(&mut text_bytes);
        let text = String::from_utf8_lossy(&text_bytes)
            .trim_end_matches('\0')
            .to_string();
        
        let timestamp = buf.get_u64();
        
        Ok(Self {
            client_order_id,
            user_id,
            reason,
            text,
            timestamp,
        })
    }
}

/// Execution Report
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutionMessage {
    pub symbol: String,
    pub client_order_id: u64,
    pub exchange_order_id: u64,
    pub execution_id: u64,
    pub user_id: u64,
    pub side: Side,
    pub fill_price: u64,
    pub fill_quantity: u64,
    pub leaves_quantity: u64,
    pub timestamp: u64,
}

impl ExecutionMessage {
    pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        if buf.len() < 88 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data for Execution",
            ));
        }
        
        // Symbol (16 bytes)
        let mut symbol_bytes = [0u8; 16];
        buf.copy_to_slice(&mut symbol_bytes);
        let symbol = String::from_utf8_lossy(&symbol_bytes)
            .trim_end_matches('\0')
            .to_string();
        
        let client_order_id = buf.get_u64();
        let exchange_order_id = buf.get_u64();
        let execution_id = buf.get_u64();
        let user_id = buf.get_u64();
        let side = if buf.get_u8() == 0x01 {
            Side::Buy
        } else {
            Side::Sell
        };
        
        // Skip reserved bytes
        buf.advance(7);
        
        let fill_price = buf.get_u64();
        let fill_quantity = buf.get_u64();
        let leaves_quantity = buf.get_u64();
        let timestamp = buf.get_u64();
        
        Ok(Self {
            symbol,
            client_order_id,
            exchange_order_id,
            execution_id,
            user_id,
            side,
            fill_price,
            fill_quantity,
            leaves_quantity,
            timestamp,
        })
    }
}
