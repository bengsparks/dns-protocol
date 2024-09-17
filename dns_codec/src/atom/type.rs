use std::io;

use byteorder::{NetworkEndian, ReadBytesExt as _};
use num_enum::TryFromPrimitive;

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

impl Type {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let decoded = src.read_u16::<NetworkEndian>()?;
        let type_ = decoded
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Some(type_))
    }
}
