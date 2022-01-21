use std::path::PathBuf;

use keri_witness_http::http_witness::HttpWitness;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// Witness db path.
    #[structopt(short = "d", long, default_value = "witness.db")]
    witness_db_path: PathBuf,

    /// Witness listen port.
    #[structopt(long, default_value = "3030")]
    api_port: u16,
}

#[tokio::main]
async fn main() {
    let Opts {
        witness_db_path: kel_db_path,
        api_port,
    } = Opts::from_args();

    let service = HttpWitness::new(kel_db_path.as_path());
    service.listen(api_port).await;
}
