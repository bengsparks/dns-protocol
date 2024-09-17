use dns_codec;
use dns_sans_io;

use log;

use futures::{SinkExt, StreamExt};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

// mod io;

#[tokio::main]
async fn main() {
    env_logger::init();

    let udpsocket = UdpSocket::bind("0.0.0.0:53").await.unwrap();
    let mut sink = UdpFramed::new(&udpsocket, dns_codec::QueryCodec);
    let mut stream = UdpFramed::new(&udpsocket, dns_codec::ResponseCodec);

    let id = 0x8296;

    let mut sans_io = dns_sans_io::DnsSansIo::new();
    let resource = "google.com".to_owned();

    sans_io.enqueue_query(
        "1.1.1.1:53".parse().unwrap(),
        id,
        dns_codec::QType::AAAA,
        resource.clone().into_bytes(),
    );
    let response = loop {
        if let Some(transmit) = sans_io.poll_query() {
            let dns_sans_io::Transmit { query, target } = transmit;
            sink.send((query, target)).await.unwrap()
        }

        tokio::select! {
            message = stream.next() => {
                match message {
                    Some(Ok((response, source))) => {
                        let response = sans_io.handle_response(source, response).unwrap();
                        match response.outcome {
                            dns_sans_io::Outcome::Resolved(records) => { break records; },
                            dns_sans_io::Outcome::NamespaceIp(records) => panic!("namespace ips"),
                            dns_sans_io::Outcome::NamespaceNames(_) => panic!("namespace names"),
                            dns_sans_io::Outcome::Unresolved => panic!("{resource} is unknown!"),
                        }
                    },
                    Some(Err(e)) => { log::error!("{e}"); return; },
                    None => { log::error!("closed?"); return; }
                };

            }
        }
    };

    println!("{response:#?}");
}
