use std::io;

use byteorder::{NetworkEndian, ReadBytesExt as _, WriteBytesExt};
use bytes::BufMut as _;
use num_enum::TryFromPrimitive;

use super::rtri;

/// QTYPE fields appear in the question part of a query.  
/// QTYPES are a superset of TYPEs, hence all TYPEs are valid QTYPEs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum QType {
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

    /// A request for a transfer of an entire zone
    AXFR = 252,

    /// A request for mailbox-related records (MB, MG or MR)
    MAILB = 253,

    /// A request for mail agent RRs (Obsolete - see MX)
    MAILA = 254,

    /// A request for all records
    STAR = 255,
}

impl QType {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let decoded = rtri!(src.read_u16::<NetworkEndian>());
        let class = decoded
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Some(class))
    }

    pub(crate) fn encode(self, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), io::Error> { 
        let mut writer = dst.writer();
        writer.write_u16::<NetworkEndian>(self as u16)?;
        Ok(())
    }
}
