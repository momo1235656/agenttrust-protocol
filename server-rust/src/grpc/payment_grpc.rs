use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use crate::state::AppState;

// Include generated proto code
pub mod agenttrust {
    pub mod payment {
        tonic::include_proto!("agenttrust.payment");
    }
}

pub use agenttrust::payment::agent_payment_service_server;
use agenttrust::payment::*;

pub struct PaymentService {
    state: Arc<AppState>,
}

impl PaymentService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl agent_payment_service_server::AgentPaymentService for PaymentService {
    async fn initiate_transfer(
        &self,
        request: Request<TransferRequest>,
    ) -> Result<Response<TransferResponse>, Status> {
        let req = request.into_inner();
        let a2a_req = crate::services::a2a_service::InitiateRequest {
            sender_did: req.sender_did,
            receiver_did: req.receiver_did,
            amount: req.amount,
            currency: Some(req.currency),
            description: Some(req.description),
            service_type: Some(req.service_type),
            timeout_minutes: Some(req.timeout_minutes as i64),
            sender_signature: None,
            message: None,
        };

        let mut redis = self.state.redis.clone();
        match crate::services::a2a_service::initiate(&self.state.db, &mut redis, &self.state.kafka, a2a_req).await {
            Ok(result) => Ok(Response::new(TransferResponse {
                transfer_id: result.transfer_id.to_string(),
                saga_id: result.saga_id.to_string(),
                status: result.status,
                timeout_at: result.timeout_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn complete_service(
        &self,
        request: Request<CompleteRequest>,
    ) -> Result<Response<CompleteResponse>, Status> {
        let req = request.into_inner();
        // Find transfer by saga_id
        let saga_id = req.saga_id.parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Invalid saga_id"))?;

        let row = sqlx::query("SELECT id FROM a2a_transfers WHERE saga_id = $1")
            .bind(saga_id)
            .fetch_optional(&self.state.db)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Transfer not found for saga"))?;

        use sqlx::Row;
        let transfer_id: uuid::Uuid = row.try_get("id").map_err(|e| Status::internal(e.to_string()))?;

        match crate::services::a2a_service::complete_transfer(&self.state.db, &self.state.kafka, transfer_id, &req.reporter_did, &req.result_summary).await {
            Ok(_) => Ok(Response::new(CompleteResponse {
                accepted: true,
                status: "completed".to_string(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    type WatchTransferStream = tokio_stream::wrappers::ReceiverStream<Result<TransferStatusUpdate, Status>>;

    async fn watch_transfer(
        &self,
        request: Request<WatchRequest>,
    ) -> Result<Response<Self::WatchTransferStream>, Status> {
        let req = request.into_inner();
        let transfer_id = req.transfer_id.parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Invalid transfer_id"))?;

        let (tx, rx) = mpsc::channel(10);
        let db = self.state.db.clone();

        tokio::spawn(async move {
            // Poll transfer status every 2 seconds for up to 5 minutes
            for _ in 0..150u32 {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let transfer = match sqlx::query_as::<_, crate::models::a2a_transfer::A2ATransfer>(
                    "SELECT * FROM a2a_transfers WHERE id = $1"
                )
                .bind(transfer_id)
                .fetch_optional(&db)
                .await {
                    Ok(Some(t)) => t,
                    _ => break,
                };

                let (current_step, step_name) = if let Some(saga_id) = transfer.saga_id {
                    let saga = sqlx::query_as::<_, crate::models::saga::Saga>(
                        "SELECT * FROM sagas WHERE id = $1"
                    )
                    .bind(saga_id)
                    .fetch_optional(&db)
                    .await
                    .ok()
                    .flatten();
                    match saga {
                        Some(s) => (s.current_step, format!("step_{}", s.current_step)),
                        None => (0, "unknown".to_string()),
                    }
                } else {
                    (0, "unknown".to_string())
                };

                let update = TransferStatusUpdate {
                    transfer_id: transfer.id.to_string(),
                    status: transfer.status.clone(),
                    current_step,
                    total_steps: 10,
                    step_name: step_name.clone(),
                    step_status: "executing".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                if tx.send(Ok(update)).await.is_err() {
                    break;
                }

                if matches!(transfer.status.as_str(), "settled" | "refunded" | "timeout") {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn dispute_escrow(
        &self,
        request: Request<DisputeRequest>,
    ) -> Result<Response<DisputeResponse>, Status> {
        let req = request.into_inner();
        let escrow_id = req.escrow_id.parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Invalid escrow_id"))?;

        match crate::services::escrow_service::dispute(
            &self.state.db, &self.state.kafka, escrow_id,
            &req.disputed_by, &req.reason, Some(&req.evidence_hash)
        ).await {
            Ok(escrow) => Ok(Response::new(DisputeResponse {
                dispute_id: escrow.id.to_string(),
                status: escrow.status,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
