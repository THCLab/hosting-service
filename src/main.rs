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

    /// ED25519 priv key
    #[structopt(short = "k", long, parse(try_from_str = parse_hex), env = "WITNESS_PRIV_KEY")]
    // `Vec<T>` means multiple args so let's use `Box<[T]>`
    priv_key: Option<Box<[u8]>>,
}

fn parse_hex(s: &str) -> Result<Box<[u8]>, hex::FromHexError> {
    Ok(hex::decode(s)?.into())
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    
    let Opts {
        witness_db_path: kel_db_path,
        api_host,
        api_port,
        resolver_address,
        priv_key,
    } = Opts::from_args();

    let service = HttpWitness::new(
        kel_db_path.as_path(),
        priv_key.as_deref(),
        api_host,
        api_port,
        ["http://".to_string(), resolver_address].join(""),
    );
    service.listen(api_port).await;
}
