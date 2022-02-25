use std::{convert::TryFrom, path::Path};

use keri::keri::witness::Witness as KeriWitness;
use keri::prefix::{BasicPrefix, Prefix};
use keri::{
    event_message::signed_event_message::Message, event_parsing::message::signed_event_stream,
    prefix::IdentifierPrefix,
};

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Witness {
    resolvers: Vec<String>,
    witness: KeriWitness,
}

impl Witness {
    pub fn new(db_path: &Path, resolvers: Vec<String>) -> Self {
        let wit = KeriWitness::new(db_path).unwrap();

        Self {
            resolvers,
            witness: wit,
        }
    }

    /// Creates a new Witness using the specified private ED25519_dalek key.
    pub fn new_with_key(db_path: &Path, resolvers: Vec<String>, priv_key: &[u8]) -> Self {
        let wit = KeriWitness::new_with_key(db_path, priv_key).unwrap();

        Self {
            resolvers,
            witness: wit,
        }
    }

    pub fn get_prefix(&self) -> BasicPrefix {
        self.witness.prefix.clone()
    }

    pub fn process(&self, stream: &str) -> Result<(Vec<Message>, Vec<Error>, Vec<u8>)> {
        // println!("\nGot events stream: {}\n", stream);
        // Parse incoming events
        let (rest, events) = signed_event_stream(stream.as_bytes()).map_err(|err| {
            let reason = err.map(|(_rest, kind)| kind.description().to_string());
            Error::ParseError(reason.to_string())
        })?;

        if events.is_empty() {
            Err(Error::ParseError("Stream can't be parsed".into()))
        } else {
            let (oks, parsing_errors): (Vec<_>, Vec<_>) = events
                .into_iter()
                .map(|e| Message::try_from(e).map_err(|e| Error::ParseError(e.to_string())))
                .partition(Result::is_ok);
            let oks: Vec<_> = oks.into_iter().map(Result::unwrap).collect();
            // process parsed events
            let processing_errors = self.witness.process(&oks)?;
            let responses = self.witness.respond()?;

            let errors = processing_errors
                .unwrap_or_default()
                .into_iter()
                .map(Error::KerioxError);

            let all_errors = parsing_errors
                .into_iter()
                .map(|e| e.unwrap_err())
                .chain(errors)
                .collect();


            let publish_state = |id: &IdentifierPrefix| -> Result<()> {
                if let Some(events) = self.witness.get_kel_for_prefix(&id)? {
                    let resolver = self.resolvers.first().expect("There's no resolver set");
                    let url = format!("{}/messages/{}", resolver, id.to_str());
                    if let Err(e) = ureq::post(&url).send_bytes(&events) {
                        println!("Problem with publishing state in resolver: {:?}", e);
                    };
                };
                Ok(())
            };
            let _updated_ids =
                responses
                    .iter()
                    .map(|msg| msg.get_prefix())
                    // remove duplicates
                    .fold(vec![], |mut acc, id| {
                        if !acc.contains(&id) {
                            acc.push(id);
                            publish_state(&id);
                        }
                        acc
                    });

            Ok((responses, all_errors, rest.to_vec()))
        }
    }

    pub fn resolve(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        self.witness.get_kel_for_prefix(id).map_err(|e| e.into())
    }

    pub fn get_receipts(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        self.witness
            .get_receipts_for_prefix(&id)
            .map_err(|e| Error::KerioxError(e))
    }
}
