use crate::witness::Witness;
use std::{path::Path, sync::Arc};
use warp::Future;

pub struct HttpWitness {
    witness: Arc<Witness>,
}

impl HttpWitness {
    pub fn new(db_path: &Path) -> Self {
        Self {
            witness: Arc::new(Witness::new(db_path)),
        }
    }

    pub fn listen(&self, port: u16) -> impl Future {
        let api = filters::all_filters(Arc::clone(&self.witness));

        warp::serve(api).run(([0, 0, 0, 0], port))
    }
}

mod filters {
    use std::sync::Arc;

    use http::StatusCode;
    use keri::{event_parsing::SignedEventData, prefix::IdentifierPrefix};

    use warp::{hyper::body::Bytes, reply::with_status, Filter, Reply};

    use crate::{error::Error, witness::Witness};

    pub fn all_filters(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        publish(db.clone()).or(get_kel(db.clone()).or(get_receipts(db)))
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
                                    String::from_utf8(SignedEventData::from(r).to_cesr().unwrap())
                                        .unwrap()
                                })
                                .collect(),
                            errors: errors
                                .iter()
                                .map(|e| e.to_string())
                                .collect::<Vec<String>>(),
                        };
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
}
