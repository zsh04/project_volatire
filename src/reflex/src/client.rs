use tonic::transport::Channel;

// Import generated types
pub mod brain {
    tonic::include_proto!("brain");
}

use brain::brain_service_client::BrainServiceClient;
use brain::StateVector;
use crate::auditor::truth_envelope::TruthEnvelope; // D-87

pub struct BrainClient {
    client: BrainServiceClient<Channel>,
}

impl BrainClient {
    pub async fn connect(dst: String) -> Result<Self, tonic::transport::Error> {
        let client = BrainServiceClient::connect(dst).await?;
        Ok(Self { client })
    }

    pub async fn reason(
        &mut self,
        price: f64,
        velocity: f64,
        vol_cluster: f64,
        entropy: f64,
        simons_prediction: f64,
    ) -> Result<brain::StrategyIntent, tonic::Status> {
        let request = tonic::Request::new(StateVector {
            price,
            velocity,
            vol_cluster,
            entropy,
            simons_prediction,
        });

        let response = self.client.reason(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_context(
        &mut self,
        truth: &TruthEnvelope
    ) -> Result<brain::ContextResponse, tonic::Status> {
        let envelope_json = serde_json::to_string(truth).unwrap_or_default();
        
        let request = tonic::Request::new(brain::ContextRequest {
            price: truth.mid_price,
            velocity: truth.velocity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            truth_envelope: envelope_json, // D-87
        });

        let response = self.client.get_context(request).await?;
        Ok(response.into_inner())
    }
}
