use std::io;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt as _};
use bytes::BufMut;
use tokio_util::bytes::{Buf, BytesMut};

use super::rtri;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Identifier assigned by the program that generates any kind of query.
    /// This identifier is copied the corresponding reply and can be used by the requester
    /// to match up replies to outstanding queries.
    pub id: u16,

    ///
    pub flags: u16,

    /// Number of entries in the question section.
    pub qdcount: u16,

    /// Number of resource records in the answer section.
    pub ancount: u16,

    /// Number of name server resource records in the authority records section.
    pub ncount: u16,

    /// Number of resource records in the additional records section.
    pub arcount: u16,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format ID as hex, leave remainder untouched
        f.debug_struct("Header")
            .field("id", &format_args!("0x{:x}", self.id))
            .field("flags", &self.flags)
            .field("qdcount", &self.qdcount)
            .field("ancount", &self.ancount)
            .field("ncount", &self.ncount)
            .field("arcount", &self.arcount)
            .finish()
    }
}

impl Header {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let mut reader = src.reader();

        let id = rtri!(reader.read_u16::<NetworkEndian>());
        let flags = rtri!(reader.read_u16::<NetworkEndian>());
        let qdcount = rtri!(reader.read_u16::<NetworkEndian>());
        let ancount = rtri!(reader.read_u16::<NetworkEndian>());
        let ncount = rtri!(reader.read_u16::<NetworkEndian>());
        let arcount = rtri!(reader.read_u16::<NetworkEndian>());

        let header = Header {
            id,
            flags,
            qdcount,
            ancount,
            ncount,
            arcount,
        };
        Ok(Some(header))
    }

    pub(crate) fn encode(self, dst: &mut BytesMut) -> Result<(), io::Error> {
        let mut writer = dst.writer();

        writer.write_u16::<NetworkEndian>(self.id)?;
        writer.write_u16::<NetworkEndian>(self.flags)?;
        writer.write_u16::<NetworkEndian>(self.qdcount)?;
        writer.write_u16::<NetworkEndian>(self.ancount)?;
        writer.write_u16::<NetworkEndian>(self.ncount)?;
        writer.write_u16::<NetworkEndian>(self.arcount)?;

        Ok(())
    }
}
