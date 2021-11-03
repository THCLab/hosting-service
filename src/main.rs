use hosting_service::http_witness::HttpWitness;

#[tokio::main]
async fn main() {
    let service = HttpWitness::new();
    service.listen().await;
}
