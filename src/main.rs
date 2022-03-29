use std::path::{Path, PathBuf};

use anyhow::Result;
use figment::{
    providers::{Format, Json},
    Figment,
};
use futures::future::join_all;
use keri_witness_http::http_witness::HttpWitness;
use serde::Deserialize;
use structopt::StructOpt;

#[derive(Deserialize)]
struct Config {
    witnesses: Option<Vec<WitnessData>>,
}

#[derive(Deserialize)]
pub struct WitnessData {
    witness_db_path: PathBuf,
    oobis_db_path: PathBuf,
    /// Witness listen host.
    api_host: String,
    /// Witness listen port.
    api_port: u16,
    /// Witness private key
    priv_key: Option<Vec<u8>>,
}

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short = "c", long, default_value = "config.json")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Opts { config_file } = Opts::from_args();

    let Config { witnesses } = Figment::new().join(Json::file(config_file)).extract()?;

    if let Some(witnesses) = witnesses {
        join_all(witnesses.iter().map(|data| {
            let sk = data.priv_key.as_ref().map(|k| k.as_slice());
            let service = HttpWitness::new(
                data.witness_db_path.as_path(),
                data.oobis_db_path.as_path(),
                data.api_host.clone(),
                data.api_port,
                sk,
            );
            service.listen()
        }))
        .await;
    } else {
        let service = HttpWitness::new(&Path::new("db"), &Path::new("oobi_db"), "127.0.0.1".into(), 3214, None);
        service.listen().await;
    }

    Ok(())
}
