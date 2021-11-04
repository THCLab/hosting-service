use keri_witness_http::http_witness::HttpWitness;

#[tokio::main]
async fn main() {
    let service = HttpWitness::new("./db");
    service.listen().await;
}
