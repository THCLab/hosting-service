use std::{convert::TryFrom, path::Path};

use keri::keri::witness::Witness as KeriWitness;
use keri::prefix::{BasicPrefix, Prefix};
use keri::{
    event_message::signed_event_message::{Message, SignedNontransferableReceipt},
    event_parsing::{message::signed_event_stream, SignedEventData},
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

    pub fn get_prefix(&self) -> BasicPrefix {
        self.witness.prefix.clone()
    }

    pub fn process(
        &self,
        stream: &str,
    ) -> Result<(Vec<SignedNontransferableReceipt>, Vec<Error>, Vec<u8>)> {
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
            let (receipts, processing_errors) = self.witness.process(&oks)?;
            let errors = processing_errors.into_iter().map(Error::KerioxError);

            let all_errors = parsing_errors
                .into_iter()
                .map(|e| e.unwrap_err())
                .chain(errors)
                .collect();

            // TODO not every receipt should update resolver.
            for rct in receipts.iter() {
                match self.witness.get_state_for_prefix(&rct.body.event.prefix)? {
                    // update key state in resolver
                    Some(state) => {
                        let resolver = self.resolvers.first().expect("There's no resolver set");
                        // TODO: Should send signed event data as octet-stream
                        if let Err(e) =
                            ureq::post(&[resolver, "/messages/", &state.prefix.to_str()].join(""))
                                .send_json(&state)
                        {
                            println!("Problem with publishing state in resolver: {:?}", e);
                        };
                    }
                    None => (),
                };
            }
            Ok((receipts, all_errors, rest.to_vec()))
        }
    }

    pub fn resolve(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        self.witness.processor.get_kerl(id).map_err(|e| e.into())
    }

    pub fn get_receipts(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        Ok(self.witness.get_nt_receipts(id)?.map(|rcts| {
            rcts.into_iter()
                .map(|rcp| SignedEventData::from(rcp).to_cesr().unwrap())
                .flatten()
                .collect()
        }))
    }
}
