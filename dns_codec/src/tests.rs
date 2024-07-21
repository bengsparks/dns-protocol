use futures::{SinkExt as _, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::Flags;

#[tokio::test]
async fn header_codec() {
    let header = crate::Header {
        id: 0x1314,
        flags: Flags(0),
        qdcount: 1,
        arcount: 0,
        ncount: 0,
        ancount: 0,
    };
    let expected = b"\x13\x14\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00";

    {
        let mut stream = FramedRead::new(&expected[..], crate::HeaderCodec);
        let value = stream.next().await.unwrap().unwrap();

        assert_eq!(value, header);
    }

    {
        let mut bytes = vec![];
        let mut sink = FramedWrite::new(&mut bytes, crate::HeaderCodec);

        sink.send(header).await.unwrap();
        assert_eq!(bytes, expected, "{bytes:x?} != {expected:?}");
    }
}

#[tokio::test]
async fn question_codec() {
    let message = crate::Question {
        name: "example.com".into(),
        kind: crate::Type::A,
        class: crate::Class::IN,
    };
    let expected = b"\x07example\x03com\x00\x00\x01\x00\x01";

    // Decoding
    {
        let mut stream = FramedRead::new(&expected[..], crate::QuestionCodec);
        let value = stream.next().await.unwrap().unwrap();

        assert_eq!(value, message);
    }

    // Encoding
    {
        let mut v = vec![];
        let mut sink = FramedWrite::new(&mut v, crate::QuestionCodec);

        sink.send(message).await.unwrap();
        assert_eq!(v, expected, "{v:x?} != {expected:x?}");
    }
}

#[tokio::test]
async fn query_codec() {
    let query = crate::Query {
        header: crate::Header {
            id: 0x8298,
            flags: Flags(1 << 8),
            qdcount: 1,
            ancount: 0,
            arcount: 0,
            ncount: 0,
        },
        question: crate::Question {
            name: "example.com".into(),
            kind: crate::Type::A,
            class: crate::Class::IN,
        },
    };
    let bytes =
        b"\x82\x98\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01";

    // Decoding
    {
        let mut stream = FramedRead::new(&bytes[..], crate::QueryCodec);
        let value = stream.next().await.unwrap().unwrap();

        assert_eq!(value, query);
    }

    // Encoding
    {
        let mut b = Vec::new();
        let mut sink = FramedWrite::new(&mut b, crate::QueryCodec);

        sink.send(query).await.unwrap();
        assert_eq!(b, bytes, "{b:x?} != {bytes:x?}");
    }
}
