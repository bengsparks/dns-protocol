use std::io;

use crate::codec::decode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub header: crate::Header,
    pub questions: Vec<crate::Question>,
    pub answers: Vec<crate::Record>,
    pub authorities: Vec<crate::Record>,
    pub additionals: Vec<crate::Record>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Tuples of (offset, count)
pub(crate) struct Metadata {
    pub(crate) questions: (u16, u64),
    pub(crate) answers: (u16, u64),
    pub(crate) authorities: (u16, u64),
    pub(crate) additionals: (u16, u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseBytes {
    pub(crate) underlying: bytes::Bytes,
    pub(crate) offsets: Metadata,
}

impl ResponseBytes {
    pub fn header(&self) -> crate::Header {
        let mut cursor = io::Cursor::new(&*self.underlying);

        let header = decode::deep::header(&mut cursor).unwrap().unwrap();
        header
    }

    pub fn questions(&self) -> QuestionDecoder<'_> {
        let mut cursor = io::Cursor::new(&*self.underlying);

        let (count, begin) = self.offsets.questions;
        let (_, end) = self.offsets.answers;

        cursor.set_position(begin);
        QuestionDecoder {
            cursor,
            end,
            hint: count,
        }
    }

    pub fn answers(&self) -> RecordDecoder<'_> {
        let mut cursor = io::Cursor::new(&*self.underlying);

        let (count, begin) =  self.offsets.answers;
        let (_, end) =  self.offsets.authorities;

        cursor.set_position(begin);
        RecordDecoder {
            cursor,
            end,
            hint: count,
        }
    }

    pub fn authorities(&self) -> RecordDecoder<'_> {
        let mut cursor = io::Cursor::new(&*self.underlying);

        let (count, begin) =  self.offsets.authorities;
        let (_, end) =  self.offsets.additionals;

        cursor.set_position(begin);
        RecordDecoder {
            cursor,
            end,
            hint: count,
        }
    }

    pub fn additionals(&self) -> RecordDecoder<'_> {
        let mut cursor = io::Cursor::new(&*self.underlying);

        let (count, begin) = self.offsets.additionals;
        let end = self.underlying.len() as u64;

        cursor.set_position(begin);
        RecordDecoder {
            cursor,
            end,
            hint: count,
        }
    }
}

pub struct QuestionDecoder<'a> {
    cursor: std::io::Cursor<&'a [u8]>,
    end: u64,
    hint: u16,
}

impl<'a> std::iter::Iterator for QuestionDecoder<'a> {
    type Item = crate::Question;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.position() >= self.end {
            return None;
        };

        let question = decode::deep::question(&mut self.cursor).unwrap().unwrap();
        Some(question)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.hint.into(), Some(self.hint.into()))
    }
}

pub struct RecordDecoder<'a> {
    cursor: std::io::Cursor<&'a [u8]>,
    end: u64,
    hint: u16,
}

impl<'a> std::iter::Iterator for RecordDecoder<'a> {
    type Item = crate::Record;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.position() >= self.end {
            return None;
        };

        let record = decode::deep::record(&mut self.cursor).unwrap().unwrap();
        Some(record)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.hint.into(), Some(self.hint.into()))
    }
}

impl std::convert::From<ResponseBytes> for Response {
    fn from(value: ResponseBytes) -> Self {
        // Invariant: the `underlying` field always contains the bytes required for this conversion
        let header = value.header();

        let questions = Vec::from_iter(value.questions());
        let answers = Vec::from_iter(value.answers());
        let authorities = Vec::from_iter(value.authorities());
        let additionals = Vec::from_iter(value.additionals());

        Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        }
    }
}
