use crate::witness::Witness;
use keri::{prefix::Prefix, signer::Signer};
use std::{path::Path, sync::Arc};
use warp::Future;

pub struct HttpWitness {
    host: String,
    port: u16,
    witness: Arc<Witness>,
}

impl HttpWitness {
    /// Uses provided ED25519 priv key or a random one if none.
    pub fn new(db_path: &Path, oobi_path: &Path, host: String, port: u16, priv_key: Option<&[u8]>) -> Self {
        let signer = match priv_key {
            Some(priv_key) => Signer::new_with_key(priv_key).unwrap(),
            None => Signer::new(),
        };

        let address = format!("{}:{}", host, port);
        let witness = Arc::new(Witness::new(db_path, oobi_path, address, Arc::new(signer)));
        Self {
            host,
            port,
            witness,
        }
    }

    pub fn listen(&self) -> impl Future {
        let api = filters::all_filters(Arc::clone(&self.witness));
        println!(
            "Witness with DID {} is listening on port {}",
            self.witness.get_prefix().to_str(),
            self.port
        );

        warp::serve(api).run(([0, 0, 0, 0], self.port))
    }
}

mod filters {
    use std::sync::Arc;

    use http::StatusCode;
    use keri::{
        event_message::signed_event_message::Message, event_parsing::SignedEventData,
        prefix::IdentifierPrefix,
    };

    use warp::{hyper::body::Bytes, reply::with_status, Filter, Reply};

    use crate::{error::Error, witness::Witness};

    pub fn all_filters(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        publish(db.clone()).or(get_kel(db.clone())
            .or(get_receipts(db.clone()))
            .or(get_oobi_proof(db)))
    }

    // POST /publish with JSON body
    pub fn publish(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        use serde::{Deserialize, Serialize};
        warp::path("publish")
            .and(warp::body::bytes())
            .and(warp::any().map(move || db.clone()))
            .map(move |param: Bytes, wit: Arc<Witness>| {
                let b = String::from_utf8(param.to_vec()).unwrap();
                match wit.process(&b) {
                    Ok((receipts, errors, rest)) => {
                        #[derive(Serialize, Deserialize)]
                        struct RespondData {
                            parsed: u64,
                            not_parsed: String,
                            receipts: Vec<String>,
                            errors: Vec<String>,
                        }

                        let res = RespondData {
                            parsed: (receipts.len() + errors.len()) as u64,
                            not_parsed: String::from_utf8(rest).unwrap(),
                            receipts: receipts
                                .into_iter()
                                .map(|r| {
                                    if let Message::NontransferableRct(rct) = r {
                                        String::from_utf8(SignedEventData::from(rct).to_cesr().unwrap())
                                            .unwrap()
                                    } else {
                                        todo!()
                                    }
                                })
                                .collect(),
                            errors: errors
                                .iter()
                                .map(|e| e.to_string())
                                .collect::<Vec<String>>(),
                        };
                        // let generated_receipts = res.receipts.join("\n\t");
                        // println!("\nParse {} events, \nnot parsed stream part: {}, \ngenerated receipts: \n\t{}, \n\nprocessing_errors: {:?}", res.parsed, res.not_parsed, generated_receipts, res.errors);
                        // let response = serde_json::to_string(&res).unwrap();
                        Ok(with_status(warp::reply::json(&res), StatusCode::OK))
                    }
                    Err(e) => {
                        #[derive(Serialize)]
                        struct ErrorWrapper {
                            error: Error,
                        }
                        Ok(with_status(
                            warp::reply::json(&ErrorWrapper { error: e }),
                            StatusCode::UNPROCESSABLE_ENTITY,
                        ))
                    }
                }
            })
    }

    // GET /identifier/{identifier}/kel
    pub fn get_kel(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("identifier")
            .and(warp::path::param())
            .and(warp::path("kel"))
            .and(warp::any().map(move || db.clone()))
            .map(move |identifier: String, wit: Arc<Witness>| {
                match identifier.parse::<IdentifierPrefix>() {
                    Ok(id) => {
                        // TODO avoid unwraps
                        match wit.resolve(&id).unwrap() {
                            Some(kel) => {
                                with_status(String::from_utf8(kel).unwrap(), StatusCode::OK)
                                    .into_response()
                            }
                            None => StatusCode::NOT_FOUND.into_response(),
                        }
                    }
                    Err(_e) => StatusCode::NOT_FOUND.into_response(),
                }
            })
    }

    // GET /identifier/{identifier}/receipts
    pub fn get_receipts(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("identifier")
            .and(warp::path::param())
            .and(warp::path("receipts"))
            .and(warp::any().map(move || db.clone()))
            .map(move |identifier: String, wit: Arc<Witness>| {
                match identifier.parse::<IdentifierPrefix>() {
                    Ok(id) => {
                        // TODO avoid unwraps
                        match wit.get_receipts(&id).unwrap() {
                            Some(rcps) => {
                                with_status(String::from_utf8(rcps).unwrap(), StatusCode::OK)
                                    .into_response()
                            }
                            None => StatusCode::NOT_FOUND.into_response(),
                        }
                    }
                    Err(_e) => StatusCode::NOT_FOUND.into_response(),
                }
            })
    }

    // GET /oobi/{identifier}
    pub fn get_oobi_proof(
        witness: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("oobi")
            .and(warp::path::param())
            .and(warp::any().map(move || witness.clone()))
            .map(move |identifier: String, wit: Arc<Witness>| {
                match identifier.parse::<IdentifierPrefix>() {
                    Ok(id) => {
                        if IdentifierPrefix::Basic(wit.get_prefix()) == id {
                            let resp = wit
                                .oobi_proof(format!("http://{}", wit.clone().address.clone()))
                                .unwrap()
                                .to_cesr()
                                .unwrap();
                            with_status(String::from_utf8(resp).unwrap(), StatusCode::OK)
                                .into_response()
                        } else {
                            StatusCode::NOT_FOUND.into_response()
                        }
                    }
                    Err(_e) => StatusCode::NOT_FOUND.into_response(),
                }
            })
    }
}
