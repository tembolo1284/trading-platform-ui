use crate::matching::{MatchingClient, OrderType as MatchOrderType, Side as MatchSide};
use crate::proto::{
    common::{OrderType, RejectReason, Side},
    trading::{
        trading_service_server::TradingService, CancelRequest, CancelResponse,
        ExecutionReport, OrderBookRequest, OrderBookSnapshot, OrderRequest, OrderResponse,
        OrderStatusRequest, OrderStatusResponse, StreamRequest, TradeReport,
    },
    Timestamp,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

/// Trading service implementation
#[derive(Clone)]
pub struct TradingServiceImpl {
    matching_client: Arc<MatchingClient>,
}

impl TradingServiceImpl {
    pub fn new(matching_client: Arc<MatchingClient>) -> Self {
        Self { matching_client }
    }
    
    /// Convert gRPC Side to matching engine Side
    fn convert_side(side: Side) -> Result<MatchSide, Status> {
        match side {
            Side::Buy => Ok(MatchSide::Buy),
            Side::Sell => Ok(MatchSide::Sell),
        }
    }
    
    /// Convert gRPC OrderType to matching engine OrderType
    fn convert_order_type(order_type: OrderType) -> Result<MatchOrderType, Status> {
        match order_type {
            OrderType::Limit => Ok(MatchOrderType::Limit),
            OrderType::Market => Ok(MatchOrderType::Market),
        }
    }
    
    /// Convert price from dollars to cents (fixed-point)
    fn price_to_cents(price: f64) -> u64 {
        (price * 100.0).round() as u64
    }
}

#[tonic::async_trait]
impl TradingService for TradingServiceImpl {
    async fn submit_order(
        &self,
        request: Request<OrderRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
        let req = request.into_inner();
        
        debug!(
            "Submitting order: symbol={}, side={:?}, price=${:.2}, qty={}",
            req.symbol, req.side, req.price, req.quantity
        );
        
        // Validate request
        if req.symbol.is_empty() {
            return Err(Status::invalid_argument("Symbol cannot be empty"));
        }
        
        if req.quantity == 0 {
            return Err(Status::invalid_argument("Quantity must be greater than 0"));
        }
        
        if req.order_type() == OrderType::Limit && req.price <= 0.0 {
            return Err(Status::invalid_argument(
                "Limit orders must have positive price",
            ));
        }
        
        // Convert types
        let side = Self::convert_side(req.side())?;
        let order_type = Self::convert_order_type(req.order_type())?;
        let price = Self::price_to_cents(req.price);
        
        // Submit to matching engine
        match self
            .matching_client
            .submit_order(
                req.symbol.clone(),
                req.user_id,
                side,
                order_type,
                price,
                req.quantity,
            )
            .await
        {
            Ok(client_order_id) => {
                info!(
                    "Order submitted successfully: id={}, symbol={}",
                    client_order_id, req.symbol
                );
                
                Ok(Response::new(OrderResponse {
                    client_order_id,
                    exchange_order_id: client_order_id, // Will be updated by ack
                    accepted: true,
                    reject_reason: RejectReason::None as i32,
                    error_message: String::new(),
                    timestamp: Some(Timestamp {
                        nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    }),
                }))
            }
            Err(e) => {
                error!("Failed to submit order: {}", e);
                
                Ok(Response::new(OrderResponse {
                    client_order_id: 0,
                    exchange_order_id: 0,
                    accepted: false,
                    reject_reason: RejectReason::SystemError as i32,
                    error_message: e.to_string(),
                    timestamp: Some(Timestamp {
                        nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    }),
                }))
            }
        }
    }
    
    async fn cancel_order(
        &self,
        request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        let req = request.into_inner();
        
        debug!(
            "Cancelling order: id={}, symbol={}",
            req.client_order_id, req.symbol
        );
        
        // Validate request
        if req.symbol.is_empty() {
            return Err(Status::invalid_argument("Symbol cannot be empty"));
        }
        
        if req.client_order_id == 0 {
            return Err(Status::invalid_argument("Invalid order ID"));
        }
        
        // Submit cancel to matching engine
        match self
            .matching_client
            .cancel_order(req.symbol.clone(), req.client_order_id, req.user_id)
            .await
        {
            Ok(()) => {
                info!("Order cancelled: id={}", req.client_order_id);
                
                Ok(Response::new(CancelResponse {
                    client_order_id: req.client_order_id,
                    cancelled: true,
                    error_message: String::new(),
                    timestamp: Some(Timestamp {
                        nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    }),
                }))
            }
            Err(e) => {
                error!("Failed to cancel order: {}", e);
                
                Ok(Response::new(CancelResponse {
                    client_order_id: req.client_order_id,
                    cancelled: false,
                    error_message: e.to_string(),
                    timestamp: Some(Timestamp {
                        nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    }),
                }))
            }
        }
    }

    // Streaming methods - stub implementations for now
    type StreamExecutionsStream =
        tokio_stream::wrappers::ReceiverStream<Result<ExecutionReport, Status>>;
    
    async fn stream_executions(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamExecutionsStream>, Status> {
        let req = request.into_inner();
        debug!("Starting execution stream for symbol: {}", req.symbol);
        
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        
        // TODO: Implement actual streaming from matching engine
        warn!("Execution streaming not yet fully implemented");
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
    
    type StreamOrderBookStream =
        tokio_stream::wrappers::ReceiverStream<Result<OrderBookSnapshot, Status>>;
    
    async fn stream_order_book(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamOrderBookStream>, Status> {
        let req = request.into_inner();
        debug!("Starting order book stream for symbol: {}", req.symbol);
        
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        
        // TODO: Implement actual streaming from matching engine
        warn!("Order book streaming not yet fully implemented");
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
    
    type StreamTradesStream = tokio_stream::wrappers::ReceiverStream<Result<TradeReport, Status>>;
    
    async fn stream_trades(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamTradesStream>, Status> {
        let req = request.into_inner();
        debug!("Starting trade stream for symbol: {}", req.symbol);
        
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        
        // TODO: Implement actual streaming from matching engine
        warn!("Trade streaming not yet fully implemented");
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
    
    async fn get_order_book(
        &self,
        request: Request<OrderBookRequest>,
    ) -> Result<Response<OrderBookSnapshot>, Status> {
        let req = request.into_inner();
        debug!(
            "Getting order book for symbol: {}, depth: {}",
            req.symbol, req.depth
        );
        
        // TODO: Query matching engine for order book snapshot
        warn!("Order book query not yet implemented");
        
        Ok(Response::new(OrderBookSnapshot {
            symbol: req.symbol,
            bids: vec![],
            asks: vec![],
            timestamp: Some(Timestamp {
                nanos: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            }),
            sequence: 0,
        }))
    }
    
    async fn get_order_status(
        &self,
        request: Request<OrderStatusRequest>,
    ) -> Result<Response<OrderStatusResponse>, Status> {
        let req = request.into_inner();
        debug!("Getting order status for id: {}", req.client_order_id);
        
        // TODO: Query matching engine for order status
        warn!("Order status query not yet implemented");
        
        Err(Status::unimplemented("Order status query not yet implemented"))
    }
}
