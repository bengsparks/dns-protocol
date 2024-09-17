use std::io;

use byteorder::{NetworkEndian, ReadBytesExt as _, WriteBytesExt as _};
use bytes::BufMut as _;
use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum QClass {
    /// The Internet
    IN = 1,

    /// The CSNET class
    CS = 2,

    /// The CHAOS class
    CH = 3,

    /// Hesiod
    HS = 4,

    /// Any class
    STAR = 255,
}

impl QClass {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let decoded = src.read_u16::<NetworkEndian>()?;
        let class = decoded
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Some(class))
    }

    pub(crate) fn encode(self, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), io::Error> {
        let mut writer = dst.writer();
        writer.write_u16::<NetworkEndian>(self as u16)?;
        Ok(())
    }
}
