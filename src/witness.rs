use std::{path::Path, sync::Arc};

use keri::{
    database::sled::SledEventDatabase,
    derivation::{basic::Basic, self_signing::SelfSigning},
    event_message::{
        event_msg_builder::ReceiptBuilder,
        parse::{signed_event_stream, signed_message, Deserialized},
        signed_event_message::SignedNontransferableReceipt,
    },
    keys::{PrivateKey, PublicKey},
    prefix::{BasicPrefix, IdentifierPrefix},
    processor::EventProcessor,
};
use rand::rngs::OsRng;

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Witness {
    keypair: (PrivateKey, PublicKey),
    prefix: BasicPrefix,
    db: Arc<SledEventDatabase>,
}

impl Witness {
    pub fn new(db_path: &Path) -> Self {
        let kp = ed25519_dalek::Keypair::generate(&mut OsRng {});
        let (vk, sk) = (kp.public, kp.secret);
        let vk = PublicKey::new(vk.to_bytes().to_vec());
        let sk = PrivateKey::new(sk.to_bytes().to_vec());
        let keypair = (sk, vk.clone());
        let prefix = Basic::Ed25519.derive(vk);
        let db = Arc::new(SledEventDatabase::new(db_path).unwrap());
        Self {
            keypair,
            prefix,
            db,
        }
    }

    pub fn process(
        &self,
        stream: &str,
    ) -> Result<(Vec<SignedNontransferableReceipt>, Vec<Error>, Vec<u8>)> {
        let (rest, events) = signed_event_stream(stream.as_bytes()).map_err(|err| {
            let reason = err.map(|(_rest, kind)| kind.description().to_string());
            Error::ParseError(reason.to_string())
        })?;

        if events.is_empty() {
            Err(Error::ParseError("Stream can't be parsed".into()))
        } else {
            let (oks, errs): (Vec<_>, Vec<_>) = events
                .iter()
                .map(|e| self.process_one(e))
                .partition(Result::is_ok);
            let oks: Vec<_> = oks.into_iter().map(Result::unwrap).collect();
            let errs: Vec<_> = errs.into_iter().map(Result::unwrap_err).collect();

            Ok((oks, errs, rest.to_vec()))
        }
    }

    pub fn process_one(&self, message: &Deserialized) -> Result<SignedNontransferableReceipt> {
        let processor = EventProcessor::new(Arc::clone(&self.db));

        // Create witness receipt and add it to db
        if let Deserialized::Event(ev) = message.clone() {
            let sn = ev.deserialized_event.event_message.event.sn;
            let prefix = ev.deserialized_event.event_message.event.prefix.clone();
            processor
                .process(message.to_owned())
                .map_err(|e| Error::ProcessingError(e, sn, prefix.clone()))?;
            let ser = ev.deserialized_event.raw;
            let signature = self
                .keypair
                .0
                .sign_ed(ser)
                .map_err(|e| Error::ProcessingError(e, sn, prefix.clone()))?;
            let rcp = ReceiptBuilder::new()
                .with_receipted_event(ev.deserialized_event.event_message)
                .build()
                .map_err(|e| Error::ProcessingError(e, sn, prefix))?;

            let signature = SelfSigning::Ed25519Sha512.derive(signature);

            let signed_rcp =
                SignedNontransferableReceipt::new(&rcp, vec![(self.prefix.clone(), signature)]);

            processor.process(signed_message(&signed_rcp.serialize()?).unwrap().1)?;

            Ok(signed_rcp)
        } else {
            // It's a receipt
            todo!()
        }
    }

    pub fn resolve(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        let processor = EventProcessor::new(Arc::clone(&self.db));
        Ok(processor.get_kerl(id)?)
    }

    pub fn get_receipts(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        Ok(self
            .db
            .get_receipts_nt(id)
            .map(|rcps| rcps.map(|r| r.serialize().unwrap()).flatten().collect()))
    }
}
