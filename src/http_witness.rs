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

    pub fn listen(&self) -> impl Future {
        let api = filters::all_filters(Arc::clone(&self.witness));

        warp::serve(api).run(([0, 0, 0, 0], 3030))
    }
}

mod filters {
    use std::sync::Arc;

    use keri::prefix::IdentifierPrefix;
    use warp::{hyper::body::Bytes, Filter};

    use crate::witness::Witness;

    pub fn all_filters(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        publish(db.clone()).or(get_kel(db.clone()).or(get_receipts(db)))
    }

    // POST /publish with JSON body
    pub fn publish(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("publish")
            .and(warp::body::bytes())
            .and(warp::any().map(move || db.clone()))
            .map(move |param: Bytes, wit: Arc<Witness>| {
                let b = String::from_utf8(param.to_vec()).unwrap();
                match wit.process(&b) {
                    Ok(receipts) => Ok(String::from_utf8(receipts).unwrap()),
                    Err(e) => Ok(e.to_string()),
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
                let id: IdentifierPrefix = identifier.parse().unwrap();

                // TODO avoid unwraps
                let kel = String::from_utf8(wit.resolve(&id).unwrap().unwrap()).unwrap();
                Ok(kel)
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
                let id: IdentifierPrefix = identifier.parse().unwrap();

                // TODO avoid unwraps
                let rcps = String::from_utf8(wit.get_receipts(&id).unwrap().unwrap()).unwrap();
                Ok(rcps)
            })
    }
}
