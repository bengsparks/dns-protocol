use std::net::Ipv4Addr;

use futures::{SinkExt as _, StreamExt as _};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 53)).await.unwrap();

    let mut sink = UdpFramed::new(&socket, dns_codec::QueryCodec);
    let mut stream = UdpFramed::new(&socket, dns_codec::ResponseCodec);

    let query = dns_codec::Query {
        header: dns_codec::Header {
            id: 0x8298,
            flags: dns_codec::Flags(0),
            qdcount: 1,
            ancount: 0,
            ncount: 0,
            arcount: 0,
        },
        question: dns_codec::Question {
            name: dns_codec::Name(b"google.com".into()),
            kind: dns_codec::QType::A,
            class: dns_codec::QClass::IN,
        },
    };

    let _ = sink
        .send((query, "198.41.0.4:53".parse().unwrap()))
        .await
        .unwrap();

    let read = stream.next().await.unwrap();
    let (response, origin) = read.unwrap();

    println!("{origin}: {response:#?}");

    println!("[HEADER]: {:?}", response.header());

    for question in response.questions() {
        println!("[QUESTION]: {:?}", question);
    }
    for record in response.answers() {
        println!("[ANSWER]: {:?}", record);
    }
    for record in response.authorities() {
        println!("[AUTHORITY]: {:?}", record);
    }
    for record in response.additionals() {
        println!("[ADDITIONAL]: {:?}", record);
    }
}
