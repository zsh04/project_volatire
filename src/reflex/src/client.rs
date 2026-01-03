use tonic::transport::Channel;

// Import generated types
pub mod brain {
    tonic::include_proto!("brain");
}

use brain::brain_service_client::BrainServiceClient;
use brain::StateVector;

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
        price: f64,
        velocity: f64,
    ) -> Result<brain::ContextResponse, tonic::Status> {
        let request = tonic::Request::new(brain::ContextRequest {
            price,
            velocity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        });

        let response = self.client.get_context(request).await?;
        Ok(response.into_inner())
    }
}
