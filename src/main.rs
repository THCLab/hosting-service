use std::path::Path;

use keri_witness_http::http_witness::HttpWitness;

#[tokio::main]
async fn main() {
    let service = HttpWitness::new(Path::new("./db"));
    service.listen(3030).await;
}
