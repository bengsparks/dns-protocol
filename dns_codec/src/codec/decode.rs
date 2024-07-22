use std::io::{self};
use tokio_util::bytes::{Buf as _, BytesMut};

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

impl tokio_util::codec::Decoder for super::ResponseCodec {
    type Item = crate::ResponseBytes;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let underlying: &[u8] = &src;
        let mut cursor = io::Cursor::new(underlying);

        // Headers have a fixed length of 96 bits; only decode counts
        let (qdcount, ancount, ncount, arcount) = thin::header(&mut cursor)?;

        let questions = cursor.position();
        for _ in 0..qdcount {
            let _ = thin::question(&mut cursor)?;
        }

        let answers = cursor.position();
        for _ in 0..ancount {
            let _ = thin::record(&mut cursor)?;
        }

        let authorities = cursor.position();
        for _ in 0..ncount {
            let _ = thin::record(&mut cursor)?;
        }

        let additionals = cursor.position();
        for _ in 0..arcount {
            let _ = thin::record(&mut cursor)?;
        }

        let underlying = src.split_to(cursor.position() as usize);
        Ok(Some(crate::ResponseBytes {
            underlying: underlying.freeze(),
            offsets: crate::Metadata {
                questions: (qdcount, questions),
                answers: (ancount, answers),
                authorities: (ncount, authorities),
                additionals: (arcount, additionals),
            },
        }))
    }
}

impl tokio_util::codec::Decoder for super::QueryCodec {
    type Item = crate::Query;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let bytes: &[u8] = &*src;
        let mut cursor = io::Cursor::new(bytes);

        let header = match deep::header(&mut cursor)? {
            Some(header) => header,
            None => return Ok(None),
        };
        let question = match deep::question(&mut cursor)? {
            Some(question) => question,
            None => return Ok(None),
        };

        src.advance(cursor.position().try_into().unwrap());
        Ok(Some(crate::Query { header, question }))
    }
}

/// Skip over bytes, ensuring everything is in the correct position
pub mod thin {
    use std::io::{self, Seek as _};

    use byteorder::{NetworkEndian, ReadBytesExt as _};

    pub fn header(cursor: &mut io::Cursor<&[u8]>) -> Result<(u16, u16, u16, u16), io::Error> {
        let (qdcount, ancount, ncount, arcount) = {
            // Skip id and flags, each are 2 bytes wide
            cursor.seek(io::SeekFrom::Current(4))?;
            (
                cursor.read_u16::<NetworkEndian>()?,
                cursor.read_u16::<NetworkEndian>()?,
                cursor.read_u16::<NetworkEndian>()?,
                cursor.read_u16::<NetworkEndian>()?,
            )
        };

        Ok((qdcount, ancount, ncount, arcount))
    }

    pub fn question(cursor: &mut io::Cursor<&[u8]>) -> Result<(), io::Error> {
        label(cursor)?;
        // kind and type are each u16, skip 4 bytes
        cursor.seek(io::SeekFrom::Current(4))?;
        Ok(())
    }

    pub fn record(cursor: &mut io::Cursor<&[u8]>) -> Result<(), io::Error> {
        label(cursor)?;

        // kind (u16), class (u16), ttl (i32)
        cursor.seek(io::SeekFrom::Current((16 + 16 + 32) / 8))?;

        // rdlength (u16) + #rdlength bytes
        let rdlength = cursor.read_u16::<NetworkEndian>()?;
        cursor.seek(io::SeekFrom::Current(rdlength.into()))?;

        Ok(())
    }

    fn label(cursor: &mut io::Cursor<&[u8]>) -> Result<(), io::Error> {
        loop {
            let length = cursor.read_u8()?;
            if length == 0 {
                break;
            }

            // If uncompressed, skip over the given length
            // Compressed, only one more byte is present.
            if (length & 0b1100_0000) == 0 {
                cursor.seek(io::SeekFrom::Current(length.into()))?;
            } else {
                cursor.seek(io::SeekFrom::Current(1))?;
                break;
            };
        }

        Ok(())
    }
}

/// Actually parse the bytestream
pub mod deep {
    use core::net;
    use std::io::{self, Read, Seek};

    use byteorder::{NetworkEndian, ReadBytesExt};
    use bytes::Buf as _;

    pub fn header(cursor: &mut io::Cursor<&[u8]>) -> Result<Option<crate::Header>, io::Error> {
        let id = cursor.read_u16::<NetworkEndian>()?;
        let flags = crate::Flags(cursor.read_u16::<NetworkEndian>()?);
        let qdcount = cursor.read_u16::<NetworkEndian>()?;
        let ancount = cursor.read_u16::<NetworkEndian>()?;
        let ncount = cursor.read_u16::<NetworkEndian>()?;
        let arcount = cursor.read_u16::<NetworkEndian>()?;

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
        let name = match label(cursor)? {
            Some(name) => name,
            None => return Ok(None),
        };

        let kind = cursor.read_u16::<NetworkEndian>()?;
        let class = cursor.read_u16::<NetworkEndian>()?;

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

        let mut length = cursor.read_u8()?;
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

            length = cursor.read_u8()?;
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

        let kind = cursor.read_u16::<NetworkEndian>()?;
        let class = cursor.read_u16::<NetworkEndian>()?;
        let ttl = cursor.read_i32::<NetworkEndian>()?;
        let rdlength = cursor.read_u16::<NetworkEndian>()?;

        let mut rdata = Vec::with_capacity(rdlength.into());
        Read::take(cursor, rdlength.into()).read_to_end(&mut rdata)?;

        if rdata.len() != rdlength.into() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Read wrong amount of bytes while parsing data portion of record!",
            ));
        }

        let kind = kind.try_into().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported resource TYPE: {kind}"),
            )
        })?;
        let class = class.try_into().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported resource CLASS: {class}"),
            )
        })?;

        Ok(Some(crate::Record {
            name: label,
            kind,
            class,
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
        Read::take(cursor, length.into()).read_to_end(&mut name)?;
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
            let following = cursor.read_u8()?;

            let combined = [unmasked, following];
            let mut reader = combined.reader();
            reader.read_u16::<NetworkEndian>()?
        };

        let backup = cursor.position();
        cursor.seek(std::io::SeekFrom::Start(pointer.into()))?;

        let crate::Name(extracted) = match label(cursor)? {
            Some(name) => name,
            None => return Ok(None),
        };
        name.extend_from_slice(&extracted);

        cursor.set_position(backup);
        Ok(Some(()))
    }
}
