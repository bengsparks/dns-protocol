use std::io;

use bytes::Buf as _;
use tokio_util::bytes::BytesMut;

use crate::{atom::rotri, Header, Question, Record, ResponseCodec};

#[derive(Debug)]
pub struct Response {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
}

impl tokio_util::codec::Decoder for ResponseCodec {
    type Item = crate::Response;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let underlying: &[u8] = src;
        let mut cursor = io::Cursor::new(underlying);

        let header = rotri!(Header::decode(&mut cursor));

        let mut questions = Vec::with_capacity(header.qdcount.into());
        for _ in 0..header.qdcount {
            let question = rotri!(Question::decode(&mut cursor)); 
            questions.push(question);
        }

        let mut answers = Vec::with_capacity(header.ancount.into());
        for _ in 0..header.ancount {
            let answer = rotri!(Record::decode(&mut cursor));
            answers.push(answer);
        }

        let mut authorities = Vec::with_capacity(header.ncount.into());
        for _ in 0..header.ncount {
            let authority = rotri!(Record::decode(&mut cursor));
            authorities.push(authority);
        }

        let mut additionals = Vec::with_capacity(header.ncount.into());
        for _ in 0..header.ncount {
            let additional = rotri!(Record::decode(&mut cursor));
            additionals.push(additional);
        }
        src.advance(cursor.position().try_into().unwrap());

        let response = crate::Response {
            header,
            questions,
            answers,
            authorities,
            additionals,
        };
        
        Ok(Some(response))
    }
}
