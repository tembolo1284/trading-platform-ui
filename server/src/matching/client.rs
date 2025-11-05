use super::protocol::*;
use anyhow::{Context, Result};
use bytes::{Buf, BytesMut};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Connection to the matching engine gateway
pub struct MatchingConnection {
    stream: Arc<Mutex<TcpStream>>,
    message_tx: mpsc::UnboundedSender<IncomingMessage>,
    sequence: Arc<RwLock<u64>>,
}

/// Incoming message types
#[derive(Debug)]
#[allow(dead_code)]
pub enum IncomingMessage {
    OrderAck(OrderAckMessage),
    OrderReject(OrderRejectMessage),
    Execution(ExecutionMessage),
}

impl MatchingConnection {
    /// Connect to the matching engine gateway
    pub async fn connect(
        address: &str,
        connect_timeout: Duration,
    ) -> Result<(Self, mpsc::UnboundedReceiver<IncomingMessage>)> {
        info!("Connecting to matching engine gateway at {}", address);
        
        let stream = timeout(connect_timeout, TcpStream::connect(address))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to gateway")?;
        
        // Disable Nagle's algorithm for low latency
        stream.set_nodelay(true)?;
        
        info!("Connected to matching engine gateway");
        
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        let conn = Self {
            stream: Arc::new(Mutex::new(stream)),
            message_tx,
            sequence: Arc::new(RwLock::new(0)),
        };
        
        // Start message receiver task
        conn.start_receiver();
        
        Ok((conn, message_rx))
    }
    
    /// Submit a new order
    pub async fn submit_order(
        &self,
        symbol: String,
        user_id: u64,
        side: Side,
        order_type: OrderType,
        price: u64,
        quantity: u64,
    ) -> Result<u64> {
        let client_order_id = self.next_sequence().await;
        
        let msg = NewOrderMessage::new(
            symbol,
            client_order_id,
            user_id,
            side,
            order_type,
            price,
            quantity,
        );
        
        debug!(
            "Submitting order: id={}, symbol={}, side={:?}, price={}, qty={}",
            client_order_id, msg.symbol, side, price, quantity
        );
        
        self.send_message(msg.encode()).await?;
        
        Ok(client_order_id)
    }
    
    /// Cancel an existing order
    pub async fn cancel_order(
        &self,
        symbol: String,
        client_order_id: u64,
        user_id: u64,
    ) -> Result<()> {
        let msg = CancelOrderMessage::new(symbol, client_order_id, user_id);
        
        debug!("Cancelling order: id={}", client_order_id);
        
        self.send_message(msg.encode()).await?;
        
        Ok(())
    }
    
    /// Send a raw message
    async fn send_message(&self, data: BytesMut) -> Result<()> {
        let mut stream = self.stream.lock().await;
        
        stream
            .write_all(&data)
            .await
            .context("Failed to send message")?;
        
        stream.flush().await.context("Failed to flush")?;
        
        Ok(())
    }
    
    /// Get next sequence number
    async fn next_sequence(&self) -> u64 {
        let mut seq = self.sequence.write().await;
        *seq += 1;
        *seq
    }
    
    /// Start the message receiver task
    fn start_receiver(&self) {
        let stream = Arc::clone(&self.stream);
        let message_tx = self.message_tx.clone();
        
        tokio::spawn(async move {
            let mut buf = BytesMut::with_capacity(4096);
            
            loop {
                let mut stream = stream.lock().await;
                
                // Read data into buffer
                match stream.read_buf(&mut buf).await {
                    Ok(0) => {
                        warn!("Gateway connection closed");
                        break;
                    }
                    Ok(n) => {
                        debug!("Received {} bytes from gateway", n);
                    }
                    Err(e) => {
                        error!("Error reading from gateway: {}", e);
                        break;
                    }
                }
                
                // Release the lock while processing messages
                drop(stream);
                
                // Process messages in buffer
                while buf.len() >= 16 {
                    // Peek at header
                    let mut peek_buf = buf.clone();
                    let header = match MessageHeader::decode(&mut peek_buf) {
                        Ok(h) => h,
                        Err(e) => {
                            error!("Failed to decode header: {}", e);
                            buf.clear();
                            break;
                        }
                    };
                    
                    // Check if we have full message
                    if buf.len() < header.length as usize {
                        debug!(
                            "Waiting for more data: have {}, need {}",
                            buf.len(),
                            header.length
                        );
                        break;
                    }
                    
                    // Remove header from buffer
                    let mut msg_buf = buf.split_to(header.length as usize);
                    msg_buf.advance(16); // Skip header
                    
                    // Process message based on type
                    match header.msg_type {
                        MessageType::OrderAck => {
                            match OrderAckMessage::decode(&mut msg_buf) {
                                Ok(msg) => {
                                    debug!("Received OrderAck: {:?}", msg);
                                    let _ = message_tx.send(IncomingMessage::OrderAck(msg));
                                }
                                Err(e) => error!("Failed to decode OrderAck: {}", e),
                            }
                        }
                        MessageType::OrderReject => {
                            match OrderRejectMessage::decode(&mut msg_buf) {
                                Ok(msg) => {
                                    debug!("Received OrderReject: {:?}", msg);
                                    let _ = message_tx.send(IncomingMessage::OrderReject(msg));
                                }
                                Err(e) => error!("Failed to decode OrderReject: {}", e),
                            }
                        }
                        MessageType::Execution => {
                            match ExecutionMessage::decode(&mut msg_buf) {
                                Ok(msg) => {
                                    debug!("Received Execution: {:?}", msg);
                                    let _ = message_tx.send(IncomingMessage::Execution(msg));
                                }
                                Err(e) => error!("Failed to decode Execution: {}", e),
                            }
                        }
                        _ => {
                            debug!("Ignoring message type: {:?}", header.msg_type);
                        }
                    }
                }
            }
            
            warn!("Message receiver task terminated");
        });
    }
}

/// Connection pool for managing multiple connections
#[allow(dead_code)]
pub struct MatchingClient {
    address: String,
    pool_size: usize,
    connect_timeout: Duration,
    connections: Arc<RwLock<Vec<Arc<MatchingConnection>>>>,
}

impl MatchingClient {
    pub async fn new(address: String, pool_size: usize, connect_timeout_ms: u64) -> Result<Self> {
        let connect_timeout = Duration::from_millis(connect_timeout_ms);
        
        info!(
            "Creating matching client pool: address={}, size={}",
            address, pool_size
        );
        
        let mut connections = Vec::with_capacity(pool_size);
        
        // Create initial connections
        for i in 0..pool_size {
            match MatchingConnection::connect(&address, connect_timeout).await {
                Ok((conn, mut rx)) => {
                    // Spawn task to handle incoming messages
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            // Here we could dispatch to subscribers
                            debug!("Pool connection {} received: {:?}", i, msg);
                        }
                    });
                    
                    connections.push(Arc::new(conn));
                }
                Err(e) => {
                    error!("Failed to create connection {}: {}", i, e);
                }
            }
        }
        
        if connections.is_empty() {
            anyhow::bail!("Failed to create any connections to gateway");
        }
        
        info!("Created {} connections to gateway", connections.len());
        
        Ok(Self {
            address,
            pool_size,
            connect_timeout,
            connections: Arc::new(RwLock::new(connections)),
        })
    }
    
    /// Get a connection from the pool (round-robin)
    async fn get_connection(&self) -> Result<Arc<MatchingConnection>> {
        let connections = self.connections.read().await;
        
        if connections.is_empty() {
            anyhow::bail!("No connections available");
        }
        
        // Simple round-robin
        let idx = (chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as usize)
            % connections.len();
        
        Ok(Arc::clone(&connections[idx]))
    }
    
    /// Submit an order through the pool
    pub async fn submit_order(
        &self,
        symbol: String,
        user_id: u64,
        side: Side,
        order_type: OrderType,
        price: u64,
        quantity: u64,
    ) -> Result<u64> {
        let conn = self.get_connection().await?;
        conn.submit_order(symbol, user_id, side, order_type, price, quantity)
            .await
    }
    
    /// Cancel an order through the pool
    pub async fn cancel_order(
        &self,
        symbol: String,
        client_order_id: u64,
        user_id: u64,
    ) -> Result<()> {
        let conn = self.get_connection().await?;
        conn.cancel_order(symbol, client_order_id, user_id).await
    }
}
