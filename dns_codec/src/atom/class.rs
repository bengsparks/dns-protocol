use std::io;

use byteorder::{NetworkEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

use super::rtri;


#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum Class {
    /// The Internet
    IN = 1,

    /// The CSNET class
    CS = 2,

    /// The CHAOS class
    CH = 3,

    /// Hesiod
    HS = 4,
}

impl Class {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let decoded = rtri!(src.read_u16::<NetworkEndian>());
        let class = decoded
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Some(class))
    }
}
