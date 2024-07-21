#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// A domain name represented as a sequence of labels, where each label consists of a
    /// length octet followed by that number of octets.
    /// The domain name terminates with the zero length octet for the null label of the root. Note that this field may be an odd number of octets; no padding is used.
    pub name: Vec<u8>,

    /// Specifies the type of the query. The values for this field include all codes valid for a TYPE field,
    /// together with some more general codes which can match more than one type of RR.
    pub kind: Type,

    /// Specifies the class of the query.
    /// For example, the QCLASS field is IN for the Internet.
    pub class: Class,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl std::convert::TryFrom<u16> for Type {
    type Error = std::io::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let variant = match value {
            1 => Self::A,
            2 => Self::NS,
            3 => Self::MD,
            4 => Self::MF,
            5 => Self::CNAME,
            6 => Self::SOA,
            7 => Self::MB,
            8 => Self::MG,
            9 => Self::MR,
            10 => Self::NULL,
            11 => Self::WKS,
            12 => Self::PTR,
            13 => Self::HINFO,
            14 => Self::MINFO,
            15 => Self::MX,
            16 => Self::TXT,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Type is expected to be between 1 and 16",
                ))
            }
        };

        Ok(variant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl std::convert::TryFrom<u16> for Class {
    type Error = std::io::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let variant = match value {
            1 => Self::IN,
            2 => Self::CS,
            3 => Self::CH,
            4 => Self::HS,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Class is expected to be between 1 and 16",
                ))
            }
        };

        Ok(variant)
    }
}
