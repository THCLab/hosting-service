use std::sync::Arc;
use hosting_service::witness::Witness;

#[tokio::main]
async fn main() {
    let db = Arc::new(Witness::new());

    let api = filters::all_filters(db);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

mod filters {
    use std::sync::Arc;

    use hosting_service::witness::Witness;
    use keri::prefix::IdentifierPrefix;
    use warp::{hyper::body::Bytes, Filter};

    pub fn all_filters(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        publish(db.clone()).or(get_kel(db.clone()).or(get_receipts(db.clone())))
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
                    Ok(_) => {
                        return Ok("Success".to_string());
                    }
                    Err(e) => {
                        return Ok(e.to_string());
                    }
                };
            })
    }

    // GET/kel/{identifier}
    pub fn get_kel(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("kel")
            .and(warp::path::param())
            .and(warp::any().map(move || db.clone()))
            .map(move |identifier: String, wit: Arc<Witness>| {
                let id: IdentifierPrefix = identifier.parse().unwrap();

                // TODO avoid unwraps
                let kel = String::from_utf8(wit.resolve(&id).unwrap().unwrap()).unwrap();
                Ok(kel)
            })
    }

    // GET/receipts/{identifier}
    pub fn get_receipts(
        db: Arc<Witness>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("receipts")
            .and(warp::path::param())
            .and(warp::any().map(move || db.clone()))
            .map(move |identifier: String, wit: Arc<Witness>| {
                let id: IdentifierPrefix = identifier.parse().unwrap();

                // TODO avoid unwraps
                let rcps = String::from_utf8(wit.get_receipts(&id).unwrap().unwrap()).unwrap();
                Ok(rcps)
            })
    }
}
