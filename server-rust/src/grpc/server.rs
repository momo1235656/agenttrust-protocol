use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

use crate::state::AppState;

pub async fn serve(state: Arc<AppState>, port: u16) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    tracing::info!("gRPC server listening on {}", addr);

    let svc = super::payment_grpc::PaymentService::new(state);

    Server::builder()
        .add_service(crate::grpc::payment_grpc::agent_payment_service_server::AgentPaymentServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
