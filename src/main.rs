use std::path::PathBuf;

use keri_witness_http::http_witness::HttpWitness;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// Witness db path.
    #[structopt(
        short = "d",
        long,
        default_value = "witness.db",
        env = "WITNESS_DB_FILE"
    )]
    witness_db_path: PathBuf,

    /// Witness listen host.
    #[structopt(short = "h", long, default_value = "0.0.0.0", env = "WITNESS_API_HOST")]
    api_host: String,

    /// Witness listen port.
    #[structopt(short = "p", long, default_value = "3030", env = "WITNESS_API_PORT")]
    api_port: u16,

    /// Resolver address.
    #[structopt(
        short = "r",
        default_value = "0.0.0.0:9599",
        env = "DKMS_RESOLVER_ENDPOINT"
    )]
    resolver_address: String,
}

#[tokio::main]
async fn main() {
    let Opts {
        witness_db_path: kel_db_path,
        api_host,
        api_port,
        resolver_address,
    } = Opts::from_args();

    let service = HttpWitness::new(
        kel_db_path.as_path(),
        api_host,
        api_port,
        ["http://".to_string(), resolver_address].join(""),
    );
    service.listen(api_port).await;
}
