use core::net;
use std::{
    collections::{HashMap, VecDeque},
    io,
};

use log;

#[derive(Debug)]
struct Enqueued {
    target: net::SocketAddr,
    query: dns_codec::Query,
}

#[derive(Debug)]
pub enum Outcome {
    Resolved(Vec<dns_codec::Record>),
    NamespaceIp(Vec<dns_codec::Record>),
    NamespaceNames(Vec<dns_codec::Record>),
    Unresolved,
}

#[derive(Debug)]
pub struct Transmit {
    pub target: net::SocketAddr,
    pub query: dns_codec::Query,
}

#[derive(Debug)]
pub struct Response {
    pub source: net::SocketAddr,
    pub target: net::SocketAddr,
    pub outcome: Outcome,
}

#[derive(Debug, Default)]
pub struct DnsSansIo {
    enqueued: VecDeque<Enqueued>,
    transmitted: HashMap<u16, (net::SocketAddr, dns_codec::QType)>,
}

impl DnsSansIo {
    pub fn new() -> Self {
        DnsSansIo {
            ..Default::default()
        }
    }
}

impl DnsSansIo {
    /// Build and enqueue DNS query
    /// TODO: Build portion
    pub fn enqueue_query(
        &mut self,
        nameserver: net::SocketAddr,
        id: u16,
        type_: dns_codec::QType,
        resource: Vec<u8>,
    ) {
        let event = format!("0x{:04x}", id);
        log::info!(target: &event, "enqueue: outgoing query for {} to {}", std::str::from_utf8(&resource).unwrap(), nameserver);

        let query = dns_codec::Query {
            header: dns_codec::Header {
                id,
                flags: 0,
                qdcount: 1,
                ancount: 0,
                ncount: 0,
                arcount: 0,
            },
            question: dns_codec::Question {
                name: resource.try_into().unwrap(),
                kind: type_,
                class: dns_codec::QClass::IN,
            },
        };

        self.enqueued.push_back(Enqueued {
            target: nameserver,
            query,
        });

        /*
        match self.enqueued.entry(query.header.id) {
            Entry::Occupied(dst) => {
                log::warn!(target: &event, "enqueue: failed because ID is already in use for {dst:?}");
            }
            Entry::Vacant(empty) => {
                empty.insert(Enqueued {
                    target: nameserver,
                    query,
                });
                log::debug!(target: &event, "enqueue: success");
            }
        };
        */
    }

    pub fn poll_query(&mut self) -> Option<Transmit> {
        let Some(Enqueued { target, query }) = self.enqueued.pop_front() else {
            return None;
        };
        /*
        let  = match self.enqueued.pop_front() {
            Some(enqueued) => enqueued,
            None => { log::warn!(target: &query.header.id, "poll: failed because ID is unknown"); return None; },
        };
        */

        let event = format!("0x{:04x}", query.header.id);

        self.transmitted
            .insert(query.header.id, (target, query.question.kind));
        log::debug!(target: &event, "poll: query {target} for {:?}", query.question.name);

        Some(Transmit { target, query })
    }

    pub fn handle_response(
        &mut self,
        nameserver: net::SocketAddr,
        response: dns_codec::Response,
    ) -> io::Result<Response> {
        // We must decode the header
        let header = response.header;
        let event = format!("0x{:04x}", header.id);

        let Some((target, interest)) = self.transmitted.remove(&header.id) else {
            log::warn!(target: &event, "response: unknown id {}", header.id);
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Unknown id {} was received", header.id),
            ));
        };

        let mut outcome = Outcome::Unresolved;

        if matches!(outcome, Outcome::Unresolved) {
            let records: Vec<_> = response
                .answers
                .into_iter()
                .filter(|r| r.kind == interest)
                .collect();
            if !records.is_empty() {
                log::info!(target: &event, "response: resolved!");
                outcome = Outcome::Resolved(records);
            }
        }

        if matches!(outcome, Outcome::Unresolved) {
            let records: Vec<_> = response
                .additionals
                .into_iter()
                .filter(|r| r.kind == interest)
                .collect();
            if !records.is_empty() {
                log::info!(target: &event, "response: received namespace ips!");
                outcome = Outcome::NamespaceIp(records);
            }
        }

        if matches!(outcome, Outcome::Unresolved) {
            let records: Vec<_> = response
                .authorities
                .into_iter()
                .filter(|r| r.kind == dns_codec::Type::NS)
                .collect();
            if !records.is_empty() {
                log::info!(target: &event, "response: received namespace names!");
                outcome = Outcome::NamespaceNames(records);
            }
        }

        Ok(Response {
            source: nameserver,
            target,
            outcome,
        })
    }

    /*

    /// Enqueue the correct message depending on the contents of
    pub fn handle_response(
        &mut self,
        nameserver: net::SocketAddr,
        response: dns_codec::ResponseBytes,
    ) {
        let header = response.header();
        let id = header.id;

        let event = format!("0x{:x}", id);
        log::debug!(target: &event, "received incoming response from {}", nameserver);

        let entry = match self.mapping.entry(header.id) {
            Entry::Occupied(entry) => entry,
            Entry::Vacant(_empty) => {
                log::error!(target: &event, "response handling failed; ID is unknown");
                return;
            }
        };

        match entry.get() {
            State::Enqueued(Enqueued { target, query }) => {
                let new_state = State::Received(Received {
                    source: nameserver,
                    target: *target,
                    interest: query.question.kind,
                    response,
                });
                *entry.into_mut() = new_state
            }
            State::Received(Received { source, target, .. }) => {
                log::error!(target: &event, "response handle failed; previous response was first received from {source} targetting {target}, current is duplicate from {nameserver}");
                return;
            }
        };
    }
    */
}

#[cfg(test)]
mod test {
    use std::net;

    use tokio_util::{bytes::BytesMut, codec::Decoder};

    #[test_log::test]
    fn resolve_ip() {
        let nameserver: net::SocketAddr = "8.8.8.8:53".parse().unwrap();
        let mut resolver = crate::DnsSansIo::default();

        // I want to know about google.com
        resolver.enqueue_query(
            nameserver,
            0x8298,
            dns_codec::QType::A,
            b"google.com".to_vec(),
        );

        let crate::Transmit { target, query: _ } = resolver.poll_query().unwrap();

        // UDP Send....

        // UDP receive...
        let origin = target;

        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(b"\x82\x98\x80\x80\0\x01\0\x01\0\0\0\0\x06google\x03com\0\0\x01\0\x01\xc0\x0c\0\x01\0\x01\0\0\0\xc2\0\x04\xac\xd9\x10\xae");

        let mut codec = dns_codec::ResponseCodec;
        let response = codec.decode(&mut bytes).unwrap().unwrap();

        let super::Response {
            source,
            target,
            outcome,
        } = resolver.handle_response(origin, response).unwrap();
        dbg!(outcome);
    }
}
