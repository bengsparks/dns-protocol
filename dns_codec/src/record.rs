use num_enum::TryFromPrimitive;

struct DebugHaystack<'a>(pub(crate) &'a [u8]);

#[inline(always)]
fn len(byte: u8) -> Option<usize> {
    if byte <= 0x7F {
        return Some(1);
    } else if byte & 0b1100_0000 == 0b1000_0000 {
        return None;
    } else if byte <= 0b1101_1111 {
        Some(2)
    } else if byte <= 0b1110_1111 {
        Some(3)
    } else if byte <= 0b1111_0111 {
        Some(4)
    } else {
        None
    }
}

#[inline(always)]
pub(crate) fn decode(bytes: &[u8]) -> Option<Result<char, u8>> {
    if bytes.is_empty() {
        return None;
    }
    let len = match len(bytes[0]) {
        None => return Some(Err(bytes[0])),
        Some(len) if len > bytes.len() => return Some(Err(bytes[0])),
        Some(1) => return Some(Ok(bytes[0] as char)),
        Some(len) => len,
    };
    match core::str::from_utf8(&bytes[..len]) {
        Ok(s) => Some(Ok(s.chars().next().unwrap())),
        Err(_) => Some(Err(bytes[0])),
    }
}

impl<'a> core::fmt::Debug for DebugHaystack<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "\"")?;
        // This is a sad re-implementation of a similar impl found in bstr.
        let mut bytes = self.0;
        while let Some(result) = decode(bytes) {
            let ch = match result {
                Ok(ch) => ch,
                Err(byte) => {
                    write!(f, r"\x{:02x}", byte)?;
                    bytes = &bytes[1..];
                    continue;
                }
            };
            bytes = &bytes[ch.len_utf8()..];
            match ch {
                '\0' => write!(f, "\\0")?,
                // ASCII control characters except \0, \n, \r, \t
                '\x01'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x19' | '\x7f' => {
                    write!(f, "\\x{:02x}", ch as u32)?;
                }
                '\n' | '\r' | '\t' | _ => {
                    write!(f, "{}", ch.escape_debug())?;
                }
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Record {
    /// A domain name to which this resource record pertains.
    pub name: crate::Name,

    /// Contains one of the RR type codes. This field specifies the meaning of the data in the RDATA field.
    pub kind: crate::Type,

    /// Specifies the class of the data in the RDATA field.
    pub class: crate::Class,

    /// Specifies the time interval (in seconds) that the resource record may be cached before it should be discarded.
    /// Zero values are interpreted to mean that the RR can only be used for the transaction in progress, and should not be cached.
    pub ttl: i32,

    /// Specifies the length in octets of the RDATA field.
    pub length: u16,

    /// A variable length string of octets that describes the resource.
    /// The format of this information varies according to the TYPE and CLASS of the resource record.
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    pub data: Vec<u8>,

    /// Extra field, containing the fully expanded `RDATA` as per `TYPE`
    pub full_data: Option<String>,
}

impl std::fmt::Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Record")
            .field("name", &DebugHaystack(&self.name.0))
            .field("kind", &self.kind)
            .field("class", &self.class)
            .field("ttl", &self.ttl)
            .field("length", &self.length)
            .field("data", &DebugHaystack(&self.data))
            .field("full_data", &self.full_data)
            .finish()
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

impl PartialEq<crate::QType> for crate::Type {
    fn eq(&self, other: &crate::QType) -> bool {
        (*self as u16) == (*other as u16)
    }
}

impl PartialEq<crate::Type> for crate::QType {
    fn eq(&self, other: &crate::Type) -> bool {
        (*self as u16) == (*other as u16)
    }
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
