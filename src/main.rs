use std::{path::Path, sync::Arc};

use keri::database::sled::SledEventDatabase;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    body: String,
}

#[tokio::main]
async fn main() {
    let db = Arc::new(SledEventDatabase::new(Path::new("./db")).unwrap()); //Arc::new(Mutex::new(Witness::new(Path::new("./db"))));

    let api = filters::all_filters(db);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

mod filters {
    use std::sync::Arc;

    use hosting_service::witness;
    use keri::{database::sled::SledEventDatabase, prefix::IdentifierPrefix};
    use warp::Filter;

    use crate::{json_body, Message};

    pub fn all_filters(
        db: Arc<SledEventDatabase>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        publish(db.clone()).or(get_kel(db.clone()))
    }

    // POST /publish with JSON body
    pub fn publish(
        db: Arc<SledEventDatabase>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("publish")
            .and(json_body())
            .and(warp::any().map(move || db.clone()))
            .map(move |param: Message, wit: Arc<SledEventDatabase>| {
                match witness::process(wit, &param.body) {
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
        db: Arc<SledEventDatabase>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("kel")
            .and(warp::path::param())
            .and(warp::any().map(move || db.clone()))
            .map(move |identifier: String, wit: Arc<SledEventDatabase>| {
                let id: IdentifierPrefix = identifier.parse().unwrap();

                let kel = String::from_utf8(witness::resolve(wit, &id).unwrap().unwrap()).unwrap();
                Ok(warp::reply::json(&kel))
            })
    }
}

fn json_body() -> impl Filter<Extract = (Message,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
