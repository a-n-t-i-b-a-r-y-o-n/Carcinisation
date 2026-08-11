pub mod bootstrap_message;

use tonic::{async_trait, include_proto, Request, Response, Status};
use crate::phenotype_server::Phenotype;

include_proto!("ipc");

#[derive(Default)]
pub struct PhenotypeService;

#[async_trait]
impl Phenotype for PhenotypeService {
    async fn ping_pong(&self, request: Request<Ping>) -> Result<Response<Pong>, Status> {
        println!("[Server] Received ping!");
        println!("[Server] Sending pong...");
        Ok(Response::new(Pong { }))
    }
}