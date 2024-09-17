use std::{cmp::Ordering, io::{self, Read}};

use byteorder::{NetworkEndian, ReadBytesExt};

use super::rtri;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ttl(i32);

impl Ttl {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let ttl = rtri!(src.read_i32::<NetworkEndian>());
        if ttl.is_negative() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TTL was decoded into a negative value",
            ));
        }

        Ok(Some(Ttl(ttl)))
    }
}

impl PartialEq<i32> for Ttl {
    fn eq(&self, other: &i32) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<i32> for Ttl {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
