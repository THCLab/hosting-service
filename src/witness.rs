use std::{path::Path, sync::Arc};

use keri::{
    database::sled::SledEventDatabase,
    derivation::{basic::Basic, self_signing::SelfSigning},
    event_message::{
        event_msg_builder::ReceiptBuilder,
        parse::{signed_message, Deserialized},
        signed_event_message::SignedNontransferableReceipt,
    },
    keys::{PrivateKey, PublicKey},
    prefix::{BasicPrefix, IdentifierPrefix},
    processor::EventProcessor,
};
use rand::rngs::OsRng;

type Result<T> = std::result::Result<T, keri::error::Error>;

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

    pub fn process(&self, stream: &str) -> Result<Vec<u8>> {
        let mut s = stream.as_bytes();
        while !s.is_empty() {
            let (rest, message) = signed_message(s).unwrap();
            s = rest;
            self.process_one(message)?;
        }
        // TODO return receipts?
        Ok(vec![])
    }

    pub fn process_one(&self, message: Deserialized) -> Result<Vec<u8>> {
        let processor = EventProcessor::new(Arc::clone(&self.db));

        processor.process(message.clone())?;

        // Create witness receipt and add it to db
        if let Deserialized::Event(ev) = message {
            let ser = ev.deserialized_event.raw;
            let signature = self.keypair.0.sign_ed(ser)?;
            let rcp = ReceiptBuilder::new()
                .with_receipted_event(ev.deserialized_event.event_message)
                .build()?;

            let signature = SelfSigning::Ed25519Sha512.derive(signature);

            let signed_rcp =
                SignedNontransferableReceipt::new(&rcp, vec![(self.prefix.clone(), signature)]);

            processor.process(signed_message(&signed_rcp.serialize()?).unwrap().1)?;

            signed_rcp.serialize()
        } else {
            // It's a receipt
            Ok(vec![])
        }
    }

    pub fn resolve(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        let processor = EventProcessor::new(Arc::clone(&self.db));
        processor.get_kerl(id)
    }

    pub fn get_receipts(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        Ok(Some(
            self.db
                .get_receipts_nt(id)
                .unwrap()
                .map(|r| r.serialize().unwrap())
                .flatten()
                .collect(),
        ))
    }
}
