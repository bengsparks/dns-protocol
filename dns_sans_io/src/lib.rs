use core::net;
use std::collections::{HashMap, VecDeque};

use log;

#[derive(Debug)]
struct Enqueued {
    target: net::SocketAddr,
    query: dns_codec::Query,
}

#[derive(Debug)]
struct Received {
    source: net::SocketAddr,
    target: net::SocketAddr,
    interest: dns_codec::QType,
    response: dns_codec::ResponseBytes,
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
    pub action: Outcome,
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

        let query = dns_codec::Query {
            header: dns_codec::Header {
                id,
                flags: dns_codec::Flags(0),
                qdcount: 1,
                ancount: 0,
                ncount: 0,
                arcount: 0,
            },
            question: dns_codec::Question {
                name: dns_codec::Name(resource),
                kind: type_,
                class: dns_codec::QClass::IN,
            },
        };

        log::info!(target: &event, "enqueue: received outgoing query for {}", nameserver);
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
        log::debug!(target: &event, "poll: query for {target}");

        Some(Transmit { target, query })
    }

    pub fn handle_response(
        &mut self,
        nameserver: net::SocketAddr,
        response: dns_codec::ResponseBytes,
    ) -> Option<Response> {
        let header = response.header();
        let event = format!("0x{:04x}", header.id);

        let Some((target, interest)) = self.transmitted.remove(&header.id) else {
            log::warn!(target: &event, "response: unknown id");
            return None;
        };

        let mut action = Outcome::Unresolved;

        if matches!(action, Outcome::Unresolved) {
            let records =
                Vec::from_iter(response.answers().filter(|answer| answer.kind == interest));
            if !records.is_empty() {
                log::info!(target: &event, "response: resolved!");
                action = Outcome::Resolved(records)
            }
        }

        if matches!(action, Outcome::Unresolved) {
            let additionals = Vec::from_iter(
                response
                    .additionals()
                    .filter(|answer| matches!(answer.kind, dns_codec::Type::NS)),
            );
            if !additionals.is_empty() {
                log::info!(target: &event, "response: received namespace IPs!");
                action = Outcome::NamespaceIp(additionals)
            }
        }

        if matches!(action, Outcome::Unresolved) {
            let authorities = Vec::from_iter(
                response
                    .authorities()
                    .filter(|answer| matches!(answer.kind, dns_codec::Type::NS)),
            );
            if !authorities.is_empty() {
                log::info!(target: &event, "response: received namespace names!");
                action = Outcome::NamespaceNames(authorities)
            }
        }

        if matches!(action, Outcome::Unresolved) {
            log::warn!(target: &event, "response: failed to resolve");
        }

        Some(Response {
            source: nameserver,
            target,
            action,
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
            0x01,
            dns_codec::QType::A,
            b"google.com".to_vec(),
        );

        let crate::Transmit { target, query: _ } = resolver.poll_query().unwrap();

        // UDP Send....

        // UDP receive...
        let origin = target;

        let raw_bytes = b"\x00\x01\x80\x80\0\x01\0\x01\0\0\0\0\x06google\x03com\0\0\x01\0\x01\xc0\x0c\0\x01\0\x01\0\0\0\xc2\0\x04\xac\xd9\x10\xae";
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&raw_bytes[..]);

        let mut codec = dns_codec::ResponseCodec;
        let response = codec.decode(&mut bytes).unwrap().unwrap();

        let super::Response {
            source,
            target,
            action,
        } = resolver.handle_response(origin, response).unwrap();
        dbg!(action);
    }
}
