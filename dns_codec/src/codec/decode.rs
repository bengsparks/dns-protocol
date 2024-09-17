use std::io::{self};
use tokio_util::bytes::{Buf as _, BytesMut};

use crate::{atom::rotri, Header, Question};

/// Converts e.kind() == UnexpectedEof from Err(e) to Ok(None) for buffering purposes.
/*
macro_rules! tri {
    ($read: expr) => {
        match $read {
            Ok(value) => value,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }
    };
}
*/

impl tokio_util::codec::Decoder for super::QueryCodec {
    type Item = crate::Query;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(&*src);

        let header = rotri!(Header::decode(&mut cursor));
        let question = rotri!(Question::decode(&mut cursor));

        src.advance(cursor.position().try_into().unwrap());

        let query = crate::Query { header, question };
        Ok(Some(query))
    }
}


impl tokio_util::codec::Encoder<crate::Query> for super::QueryCodec {
    type Error = io::Error;

    fn encode(&mut self, item: crate::Query, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.header.encode(dst)?;
        item.question.encode(dst)?;

        Ok(())
    }
}