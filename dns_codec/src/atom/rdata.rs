use core::net;
use std::{
    io::{self, Read},
    net::{Ipv4Addr, Ipv6Addr},
};

use atom::{Class, Type};
use byteorder::{NetworkEndian, ReadBytesExt};

use crate::atom;

use super::rtri;

impl RData {
    pub(crate) fn decode<'a>(
        src: &mut io::Cursor<&'a [u8]>,
        length: u16,
        kind: Type,
        class: Class,
    ) -> Result<Option<Self>, io::Error> {
        let mut available = src.take(length.into());

        let rdata = match (kind, class) {
            (Type::A, Class::IN) => {
                let bits = rtri!(available.read_u32::<NetworkEndian>());
                let address = Ipv4Addr::from_bits(bits);
                RData::Ipv4(address)
            }
            (Type::AAAA, Class::IN) => {
                let bits = rtri!(available.read_u128::<NetworkEndian>());
                let address = Ipv6Addr::from_bits(bits);
                RData::Ipv6(address)
            }
            _ => {
                unimplemented!("Unsupported kind, class pair: {kind:?}, {class:?})")
            } /*
              _ => {
                  let mut data = Vec::with_capacity(length.into());
                  let _ = rtri!(available.read_to_end(&mut data));
                  RData::Otherwise(data)
              }
              */
        };

        Ok(Some(rdata))
    }
}

#[derive(Debug)]
#[repr(u16)]
pub enum RData {
    Ipv4(net::Ipv4Addr) = 1,
    Ipv6(net::Ipv6Addr) = 26,
    Name(atom::Name),
}
