use std::sync::Arc;
use std::{convert::TryFrom, path::Path};

use keri::derivation::self_addressing::SelfAddressing;
use keri::derivation::self_signing::SelfSigning;
use keri::event::SerializationFormats;
use keri::keri::witness::Witness as KeriWitness;
use keri::oobi::{Oobi, OobiManager, Scheme};
use keri::prefix::BasicPrefix;
use keri::processor::validator::EventValidator;
use keri::query::reply::{ReplyEvent, SignedReply};
use keri::signer::Signer;
use keri::{
    event_message::signed_event_message::Message, event_parsing::message::signed_event_stream,
    prefix::IdentifierPrefix,
};

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Witness {
    pub address: String,
    witness: KeriWitness,
    pub oobi_manager: OobiManager,
    signer: Arc<Signer>,
}

impl Witness {
    pub fn new(db_path: &Path, address: String, signer: Arc<Signer>) -> Self {
        let wit = KeriWitness::new(db_path, signer.public_key()).unwrap();
        let oobi_manager = OobiManager::new(EventValidator::new(wit.get_db_ref()));

        Self {
            address,
            signer: signer,
            oobi_manager,
            witness: wit,
        }
    }

    pub fn get_prefix(&self) -> BasicPrefix {
        self.witness.prefix.clone()
    }

    pub fn process(&self, stream: &str) -> Result<(Vec<Message>, Vec<Error>, Vec<u8>)> {
        println!("\nGot events stream: {}\n", stream);
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
            let responses = self.witness.respond(self.signer.clone())?;

            let errors = processing_errors
                .unwrap_or_default()
                .into_iter()
                .map(Error::KerioxError);

            let all_errors = parsing_errors
                .into_iter()
                .map(|e| e.unwrap_err())
                .chain(errors)
                .collect();

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

    pub fn oobi_proof(&self, url: String) -> Result<Message> {
        let oobi = Oobi::new(
            IdentifierPrefix::Basic(self.get_prefix()),
            Scheme::Http,
            url,
        );
        let rpy = ReplyEvent::new_reply(
            oobi,
            keri::query::Route::ReplyOobi,
            SelfAddressing::Blake3_256,
            SerializationFormats::JSON,
        )?;
        let signature = SelfSigning::Ed25519Sha512
            .derive(self.signer.clone().sign(rpy.serialize().unwrap()).unwrap());
        let rr = SignedReply::new_nontrans(rpy, self.get_prefix(), signature);
        Ok(Message::SignedOobi(rr))
    }
}
