use futures::{SinkExt as _, StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::Flags;

#[tokio::test]
async fn query() {
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
            name: crate::Name("example.com".into()),
            kind: crate::QType::A,
            class: crate::QClass::IN,
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
