
use std::sync::Arc;

use keri::{
    database::sled::SledEventDatabase, event_message::parse::signed_message,
    prefix::IdentifierPrefix, processor::EventProcessor,
};

type Result<T> = std::result::Result<T, keri::error::Error>;

pub fn process(db: Arc<SledEventDatabase>, stream: &str) -> Result<()> {
    let (_rest, message) = signed_message(&stream.as_bytes()).unwrap();
    let processor = EventProcessor::new(Arc::clone(&db));

    processor.process(message)?;

    // TODO Create witness receipt and add it to db

    Ok(())
}

pub fn resolve(db: Arc<SledEventDatabase>, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
    let processor = EventProcessor::new(Arc::clone(&db));
    processor.get_kerl(id)
}

pub fn get_receipts(db: Arc<SledEventDatabase>, id: &IdentifierPrefix) -> Result<Option<Vec<u8>>> {
    todo!()
}
