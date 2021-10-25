use std::{path::Path, sync::Arc};

use keri::{
    database::sled::SledEventDatabase, event_message::parse::signed_message,
    prefix::IdentifierPrefix, processor::EventProcessor,
};

type Result<T> = std::result::Result<T, keri::error::Error>;

pub struct Witness {
    db: Arc<SledEventDatabase>,
}

impl Witness {
    pub fn new(path: &Path) -> Self {
        Self {
            db: Arc::new(SledEventDatabase::new(path).unwrap()),
        }
    }

    pub fn process(&self, stream: &[u8]) -> Result<()> {
        let (_rest, message) = signed_message(&stream).unwrap();
        let processor = EventProcessor::new(Arc::clone(&self.db));
        processor.process(message)?;

        // TODO Create witness receipt and add it to db

        Ok(())
    }

    pub fn resolve(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        let processor = EventProcessor::new(Arc::clone(&self.db));
        processor.get_kerl(id)
    }

    pub fn get_receipts(&self, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
        todo!()
    }
}
