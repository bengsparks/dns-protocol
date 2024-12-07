use std::{
    fs::File,
    io,
    net::{self, IpAddr},
};

use dns_codec::{self, RData};
use dns_sans_io;

use log;
use std::io::Write as _;

use futures::{SinkExt, TryStreamExt};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

// mod io;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(
            File::create("dns.log").unwrap(),
        )))
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] - {}",
                record.file().unwrap(),
                record.line().unwrap(),
                record.level(),
                record.args()
            )
        })
        .parse_default_env()
        .init();

    let udpsocket = UdpSocket::bind("0.0.0.0:53").await.unwrap();
    let mut sans_io = dns_sans_io::DnsSansIo::new();

    let ip = resolve(
        &mut sans_io,
        &udpsocket,
        "made-by-fin.de".to_owned(),
        dns_codec::QClass::IN,
    )
    .await
    .unwrap();

    println!("{ip:#?}");
}

async fn resolve(
    protocol: &mut dns_sans_io::DnsSansIo,
    socket: &UdpSocket,
    domain_name: String,
    record_type: dns_codec::QClass,
) -> Result<IpAddr, io::Error> {
    let mut id = 0x8296;

    let mut sink = UdpFramed::new(socket, dns_codec::QueryCodec);
    let mut stream = UdpFramed::new(socket, dns_codec::ResponseCodec);

    // let mut nameserver: net::IpAddr = "94.135.228.220".parse().unwrap();
    let mut nameserver: net::IpAddr = "194.146.107.6".parse().unwrap();

    loop {
        id += 1;
        protocol.enqueue_query(
            net::SocketAddr::from((nameserver, 53)),
            id,
            dns_codec::QType::A,
            domain_name.clone().into_bytes(),
        );

        if let Some(transmit) = protocol.poll_query() {
            let dns_sans_io::Transmit { query, target } = transmit;
            sink.send((query, target)).await.unwrap()
        }

        let (response, source) = stream.try_next().await?.unwrap();
        log::debug!("{:#?}", response);
        let simple = protocol.handle_response(source, response).unwrap();

        match simple.outcome {
            dns_sans_io::Outcome::Resolved(mut records) => {
                log::info!("Successfully resolved! {records:#?}");
                return Ok(records.pop().unwrap().rdata.try_into().unwrap());
            }
            dns_sans_io::Outcome::NamespaceIp(mut records) => {
                log::info!("Namespace IP(s) found");
                log::debug!("{:#?}", records[0]);

                match records.pop().unwrap().rdata {
                    RData::Ipv4(ns_ip) => {
                        // log::info!("Enqueuing IPv4 query to nameserver for {resource}");
                        nameserver = ns_ip.into();
                    }
                    RData::Ipv6(ns_ip) => {
                        // log::info!("Enqueuing IPv6 query to nameserver for {resource}");
                        nameserver = ns_ip.into();
                    }
                    RData::Name(ns_name) => {
                        nameserver = Box::pin(resolve(
                            protocol,
                            socket,
                            String::from_utf8(ns_name.0).unwrap(),
                            record_type,
                        ))
                        .await?;
                    } // _ => panic!("Expected an IPv{{4,6}} or name, got {:#?}", records[0].rdata)
                };
            }
            dns_sans_io::Outcome::NamespaceNames(_) => panic!("namespace names"),
            dns_sans_io::Outcome::Unresolved => panic!("{domain_name} is unknown!"),
        }
    }
}
