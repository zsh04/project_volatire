pub mod client;
pub mod feynman;
pub mod market;
pub mod ingest;
pub mod ledger;
pub mod taleb;
pub mod audit;
pub mod simons;
pub mod execution;
pub mod governor;
pub mod brain;
pub mod telemetry;
pub mod sim;
pub mod db;
pub mod server;

// Import the generated code
pub mod reflex_proto {
    tonic::include_proto!("reflex");
}

pub mod brain_proto {
     tonic::include_proto!("brain");
}
