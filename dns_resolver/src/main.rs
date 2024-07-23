use dns_codec;
use dns_sans_io;

use futures::{SinkExt, StreamExt};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

#[tokio::main]
async fn main() {
    env_logger::init();

    let udpsocket = UdpSocket::bind("0.0.0.0:53").await.unwrap();
    let mut stream = UdpFramed::new(&udpsocket, dns_codec::ResponseCodec);
    let mut sink = UdpFramed::new(&udpsocket, dns_codec::QueryCodec);

    let id = 0x8296;

    let mut sans_io = dns_sans_io::DnsSansIo::new();
    sans_io.enqueue_query(
        "8.8.8.8:53".parse().unwrap(),
        id,
        dns_codec::QType::A,
        b"google.com".to_vec(),
    );
    let address = loop {
        if let Some(transmit) = sans_io.poll_query() {
            let dns_sans_io::Transmit { query, target } = transmit;
            sink.send((query, target)).await.unwrap()
        }

        tokio::select! {
            Some(Ok((response, source))) = stream.next() => {
                let dns_sans_io::Response { action, .. } = sans_io.handle_response(source, response).unwrap();
                match action {
                    dns_sans_io::Outcome::Resolved(records) => { break records; },
                    dns_sans_io::Outcome::NamespaceIp(_) => todo!(),
                    dns_sans_io::Outcome::NamespaceNames(_) => todo!(),
                    dns_sans_io::Outcome::Unresolved => panic!("google.com is unknown!"),
                }
            }
        }
    };

    println!("{address:#?}");
}
