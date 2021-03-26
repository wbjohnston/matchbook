use bytes::{BufMut, BytesMut};
use fixer_upper::Message as FixMessage;
use matchbook_types::*;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone, Default)]
pub struct MatchbookMessageCodec {
    buf: Vec<u8>,
}

impl MatchbookMessageCodec {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }
}

impl Encoder<Message> for MatchbookMessageCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: Message,
        dst: &mut BytesMut,
    ) -> Result<(), <Self as Encoder<Message>>::Error> {
        serde_json::to_writer(&mut self.buf, &item)?;
        dst.put(&self.buf[..self.buf.len()]);
        self.buf.clear();
        Ok(())
    }
}

impl Decoder for MatchbookMessageCodec {
    type Item = Message;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            Ok(None)
        } else {
            let maybe_message = serde_json::from_slice(src).map(Some).map_err(|e| e.into());
            src.clear();
            maybe_message
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FixJsonCodec {
    buf: Vec<u8>,
}

impl FixJsonCodec {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }
}

impl Encoder<FixMessage> for FixJsonCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: FixMessage,
        dst: &mut BytesMut,
    ) -> Result<(), <Self as Encoder<FixMessage>>::Error> {
        serde_json::to_writer(&mut self.buf, &item)?;
        dst.put(&self.buf[..self.buf.len()]);
        self.buf.clear();
        Ok(())
    }
}

impl Decoder for FixJsonCodec {
    type Item = FixMessage;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            Ok(None)
        } else {
            let maybe_message = serde_json::from_slice(src).map(Some).map_err(|e| e.into());
            src.clear();
            maybe_message
        }
    }
}
