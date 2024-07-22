use std::io;
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

impl tokio_util::codec::Decoder for super::ResponseCodec {
    type Item = crate::Response;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let bytes: &[u8] = &*src;
        let mut cursor = io::Cursor::new(bytes);

        let header = match components::header(&mut cursor)? {
            Some(header) => header,
            None => return Ok(None),
        };

        let mut questions = Vec::with_capacity(header.qdcount.into());
        for _ in 0..header.qdcount {
            questions.push(match components::question(&mut cursor)? {
                Some(question) => question,
                None => return Ok(None),
            });
        }

        let mut answers = Vec::with_capacity(header.ancount.into());
        for _ in 0..header.ancount {
            answers.push(match components::record(&mut cursor)? {
                Some(record) => record,
                None => return Ok(None),
            });
        }

        let mut authorities = Vec::with_capacity(header.ncount.into());
        for _ in 0..header.ncount {
            authorities.push(match components::record(&mut cursor)? {
                Some(record) => record,
                None => return Ok(None),
            });
        }

        let mut additionals = Vec::with_capacity(header.arcount.into());
        for _ in 0..header.arcount {
            additionals.push(match components::record(&mut cursor)? {
                Some(record) => record,
                None => return Ok(None),
            });
        }

        src.advance(cursor.position().try_into().unwrap());
        Ok(Some(crate::Response {
            header,
            questions,
            answers,
            authorities,
            additionals,
        }))
    }
}

impl tokio_util::codec::Decoder for super::QueryCodec {
    type Item = crate::Query;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let bytes: &[u8] = &*src;
        let mut cursor = io::Cursor::new(bytes);

        let header = match components::header(&mut cursor)? {
            Some(header) => header,
            None => return Ok(None),
        };
        let question = match components::question(&mut cursor)? {
            Some(question) => question,
            None => return Ok(None),
        };

        src.advance(cursor.position().try_into().unwrap());
        Ok(Some(crate::Query { header, question }))
    }
}

pub mod components {
    use std::io::{self, Read, Seek};

    use byteorder::{NetworkEndian, ReadBytesExt as _};
    use tokio_util::bytes::Buf as _;

    pub fn header(cursor: &mut io::Cursor<&[u8]>) -> Result<Option<crate::Header>, io::Error> {
        let id = tri!(cursor.read_u16::<NetworkEndian>());
        let flags = crate::Flags(tri!(cursor.read_u16::<NetworkEndian>()));
        let qdcount = tri!(cursor.read_u16::<NetworkEndian>());
        let ancount = tri!(cursor.read_u16::<NetworkEndian>());
        let ncount = tri!(cursor.read_u16::<NetworkEndian>());
        let arcount = tri!(cursor.read_u16::<NetworkEndian>());

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

    pub fn question(cursor: &mut io::Cursor<&[u8]>) -> Result<Option<crate::Question>, io::Error> {
        let name = match tri!(label(cursor)) {
            Some(name) => name,
            None => return Ok(None),
        };

        let kind = tri!(cursor.read_u16::<NetworkEndian>());
        let class = tri!(cursor.read_u16::<NetworkEndian>());

        Ok(Some(crate::Question {
            name,
            kind: kind.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported resource QTYPE: {kind}"),
                )
            })?,
            class: class.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported resource QCLASS: {class}"),
                )
            })?,
        }))
    }

    pub fn label(cursor: &mut io::Cursor<&[u8]>) -> Result<Option<crate::Name>, io::Error> {
        let mut name = vec![];

        let mut length = tri!(cursor.read_u8());
        while length != 0 {
            if length & 0b1100_0000 != 0 {
                if decode_compressed(&mut name, cursor, length)?.is_none() {
                    return Ok(None);
                }
                break;
            } else {
                if decode_uncompressed(&mut name, cursor, length)?.is_none() {
                    return Ok(None);
                }
            };

            length = tri!(cursor.read_u8());
            if length != 0 {
                name.push(b'.');
            }
        }
        Ok(Some(crate::Name(name)))
    }

    pub fn record(cursor: &mut io::Cursor<&[u8]>) -> Result<Option<crate::Record>, io::Error> {
        let label = match label(cursor)? {
            Some(header) => header,
            None => return Ok(None),
        };

        let kind = tri!(cursor.read_u16::<NetworkEndian>());
        let class = tri!(cursor.read_u16::<NetworkEndian>());
        let ttl = tri!(cursor.read_u32::<NetworkEndian>());
        let rdlength = tri!(cursor.read_u16::<NetworkEndian>());

        let mut rdata = Vec::with_capacity(rdlength.into());
        tri!(std::io::Read::take(cursor, rdlength.into()).read_to_end(&mut rdata));

        if rdata.len() != rdlength.into() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Read wrong amount of bytes while parsing data portion of record!",
            ));
        }

        Ok(Some(crate::Record {
            name: label,
            kind: kind.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported resource TYPE: {kind}"),
                )
            })?,
            class: class.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported resource CLASS: {class}"),
                )
            })?,
            ttl,
            length: rdlength,
            data: rdata,
        }))
    }

    fn decode_uncompressed(
        mut name: &mut Vec<u8>,
        cursor: &mut io::Cursor<&[u8]>,
        length: u8,
    ) -> Result<Option<()>, io::Error> {
        let current = name.len();
        tri!(std::io::Read::take(cursor, length.into()).read_to_end(&mut name));
        let now = name.len();

        if now - current != length.into() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Read wrong amount of bytes while parsing uncompressed label",
            ));
        }

        Ok(Some(()))
    }

    fn decode_compressed(
        name: &mut Vec<u8>,
        cursor: &mut io::Cursor<&[u8]>,
        length: u8,
    ) -> Result<Option<()>, io::Error> {
        let pointer = {
            let unmasked = length & 0b0011_1111;
            let following = tri!(cursor.read_u8());

            let combined = [unmasked, following];
            let mut reader = combined.reader();
            tri!(reader.read_u16::<NetworkEndian>())
        };

        let backup = cursor.position();
        cursor.seek(std::io::SeekFrom::Start(pointer.into()))?;

        let crate::Name(extracted) = match tri!(label(cursor)) {
            Some(name) => name,
            None => return Ok(None),
        };
        name.extend_from_slice(&extracted);

        cursor.set_position(backup);
        Ok(Some(()))
    }
}
