use std::io::{self, Read};

use byteorder::{NetworkEndian, ReadBytesExt};
use tokio_util::bytes::{Buf as _, BytesMut};

/// Converts e.kind() == UnexpectedEof from Err(e) to Ok(None) for buffering purposes.
macro_rules! tri {
    ($read: expr) => {
        match $read {
            Ok(value) => value,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }
    };
}

impl tokio_util::codec::Decoder for super::HeaderCodec {
    type Item = crate::Header;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        const SIZE: usize = std::mem::size_of::<crate::Header>();
        if src.len() < SIZE {
            return Ok(None);
        }

        let mut reader = src.reader();

        let id = tri!(reader.read_u16::<NetworkEndian>());
        let flags = crate::Flags(tri!(reader.read_u16::<NetworkEndian>()));
        let qdcount = tri!(reader.read_u16::<NetworkEndian>());
        let ancount = tri!(reader.read_u16::<NetworkEndian>());
        let ncount = tri!(reader.read_u16::<NetworkEndian>());
        let arcount = tri!(reader.read_u16::<NetworkEndian>());

        let header = crate::Header {
            id,
            flags,
            qdcount,
            ancount,
            ncount,
            arcount,
        };
        Ok(Some(header))
    }
}

impl tokio_util::codec::Decoder for super::QuestionCodec {
    type Item = crate::Question;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut reader = src.reader();
        let mut name = vec![];

        let mut length = tri!(reader.read_u8());
        loop {
            let mut segment_reader = (&mut reader).take(length.into());
            let read = tri!(segment_reader.read_to_end(&mut name));
            if read != length as usize {
                return Ok(None);
            }

            length = tri!(reader.read_u8());
            if length == 0 {
                break;
            } else {
                name.push(b'.');
            }
        }

        let kind = tri!(reader.read_u16::<NetworkEndian>());
        let class = tri!(reader.read_u16::<NetworkEndian>());
        Ok(Some(crate::Question {
            name,
            kind: kind.try_into()?,
            class: class.try_into()?,
        }))
    }
}

impl tokio_util::codec::Decoder for super::QueryCodec {
    type Item = crate::Query;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut header_codec = super::HeaderCodec;
        let header = match header_codec.decode(src)? {
            Some(header) => header,
            None => return Ok(None),
        };

        let mut question_codec = super::QuestionCodec;
        let question = match question_codec.decode(src)? {
            Some(question) => question,
            None => return Ok(None),
        };

        Ok(Some(Self::Item { header, question }))
    }
}
