use std::io::Write as _;

use byteorder::{NetworkEndian, WriteBytesExt};
use tokio_util::bytes::{BufMut as _, BytesMut};

impl tokio_util::codec::Encoder<crate::Header> for super::HeaderCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: crate::Header, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut writer = dst.writer();

        writer.write_u16::<NetworkEndian>(item.id)?;
        writer.write_u16::<NetworkEndian>(item.flags.0)?;
        writer.write_u16::<NetworkEndian>(item.qdcount)?;
        writer.write_u16::<NetworkEndian>(item.ancount)?;
        writer.write_u16::<NetworkEndian>(item.ncount)?;
        writer.write_u16::<NetworkEndian>(item.arcount)?;

        Ok(())
    }
}

impl tokio_util::codec::Encoder<crate::Question> for super::QuestionCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: crate::Question, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut writer = dst.writer();

        let crate::Name(labels) = item.name;

        for label in labels.split(|c| *c == b'.') {
            writer.write_u8(label.len() as u8)?;
            writer.write_all(label)?;
        }
        writer.write_u8(0)?;

        writer.write_u16::<NetworkEndian>(item.kind as u16)?;
        writer.write_u16::<NetworkEndian>(item.class as u16)?;

        Ok(())
    }
}

impl tokio_util::codec::Encoder<crate::Query> for super::QueryCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: crate::Query, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let crate::Query { header, question } = item;

        let mut header_codec = super::HeaderCodec;
        header_codec.encode(header, dst)?;

        let mut question_codec = super::QuestionCodec;
        question_codec.encode(question, dst)?;

        Ok(())
    }
}
