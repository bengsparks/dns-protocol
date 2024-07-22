use std::{io, net::{Ipv4Addr, Ipv6Addr}};

use byteorder::{NetworkEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use tokio_util::bytes::Buf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// A domain name to which this resource record pertains.
    pub name: crate::Name,

    /// Contains one of the RR type codes. This field specifies the meaning of the data in the RDATA field.
    pub kind: crate::Type,

    /// Specifies the class of the data in the RDATA field.
    pub class: crate::Class,

    /// Specifies the time interval (in seconds) that the resource record may be cached before it should be discarded.
    /// Zero values are interpreted to mean that the RR can only be used for the transaction in progress, and should not be cached.
    pub ttl: u32,

    /// Specifies the length in octets of the RDATA field.
    pub length: u16,

    /// A variable length string of octets that describes the resource.
    /// The format of this information varies according to the TYPE and CLASS of the resource record.
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    pub data: Vec<u8>,
}

impl Record {
    pub fn pretty_data(&self) -> String {
        match &self.kind {
            crate::Type::A => {
                let mut reader = self.data.reader();
                // TODO: Upgrade when #![feature(ip_bits)] becomes stable
                // TODO: let address = reader.read_u32::<NetworkEndian>().unwrap();
                // TODO: Ipv4Addr::from_bits(address)
                Ipv4Addr::new(
                    reader.read_u8().unwrap(),
                    reader.read_u8().unwrap(),
                    reader.read_u8().unwrap(),
                    reader.read_u8().unwrap(),
                )
                .to_string()
            }
            crate::Type::AAAA => {
                let mut reader = self.data.reader();

                // TODO: Upgrade when #![feature(ip_bits)] becomes stable
                // TODO: let address = reader.read_u128::<NetworkEndian>().unwrap();
                // TODO: Ipv6Addr::from_bits(address)
                Ipv6Addr::new(
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                    reader.read_u16::<NetworkEndian>().unwrap(),
                )
                .to_string()
            }
            crate::Type::NS => {
                // TODO: This breaks when handling compressed messages, as the offset 
                // TODO: should be from the start of the packet, but here is from the start of bytestream
                // TODO: from the point of 
                let mut cursor = io::Cursor::new(&*self.data);
                let crate::Name(label) = crate::label(&mut cursor)
                    .unwrap()
                    .unwrap();
                let ipv6 = std::str::from_utf8(&label).unwrap();
                ipv6.to_string()
            }
            _ => todo!("{:?} x {:?}", &self.kind, &self.class),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum Type {
    /// A host address
    A = 1,

    /// An authoritative name server
    NS = 2,

    /// A mail destination (Obsolete - use MX)
    MD = 3,

    /// A mail forwarder (Obsolete - use MX)
    MF = 4,

    /// The canonical name for an alias
    CNAME = 5,

    /// Marks the start of a zone of authority
    SOA = 6,

    /// A mailbox domain name (EXPERIMENTAL)
    MB = 7,

    /// A mail group member (EXPERIMENTAL)
    MG = 8,

    /// A mail rename domain name (EXPERIMENTAL)
    MR = 9,

    /// A null RR (EXPERIMENTAL)
    NULL = 10,

    /// A well known service description
    WKS = 11,

    /// A domain name pointer
    PTR = 12,

    /// Host information
    HINFO = 13,

    /// Mailbox or mail list information
    MINFO = 14,

    /// Mail exchange
    MX = 15,

    /// Text strings
    TXT = 16,

    /// Responsible Person
    RP = 17,

    /// Location of database servers of an AFS cell.
    AFSDB = 18,

    SIG = 24,

    KEY = 25,

    AAAA = 28,

    LOC = 29,

    SRV = 33,

    NAPTR = 35,

    KX = 36,

    CERT = 37,

    DNAME = 39,

    APL = 42,

    DS = 43,

    SSHFP = 44,

    IPSECKEY = 45,

    RRSIG = 46,

    NSEC = 47,

    DNSKEY = 48,

    DHCID = 49,

    NSEC3 = 50,

    NSEC3PARAM = 51,

    TSLA = 52,

    SMIMEA = 53,

    HIP = 55,

    CDS = 59,

    CDNSKEY = 60,

    OPENPGPKEY = 61,

    CSYNC = 62,

    ZONEMD = 63,

    SVCB = 64,

    HTTOS = 65,

    EUI48 = 108,

    EUI64 = 109,

    TKEY = 249,

    TSIG = 250,

    URI = 256,

    CAA = 257,

    WALLET = 262,

    TA = 32768,

    DLV = 32769,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum Class {
    /// The Internet
    IN = 1,

    /// The CSNET class
    CS = 2,

    /// The CHAOS class
    CH = 3,

    /// Hesiod
    HS = 4,
}
