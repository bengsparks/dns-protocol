#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Header {
    /// Identifier assigned by the program that generates any kind of query.
    /// This identifier is copied the corresponding reply and can be used by the requester
    /// to match up replies to outstanding queries.
    pub id: u16,

    ///
    pub flags: Flags,

    /// Number of entries in the question section.
    pub qdcount: u16,

    /// Number of resource records in the answer section.
    pub ancount: u16,

    /// Number of name server resource records in the authority records section.
    pub ncount: u16,

    /// Number of resource records in the additional records section.
    pub arcount: u16,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("id", &format_args!("0x{:x}", self.id))
            .field("flags", &self.flags)
            .field("qdcount", &self.qdcount)
            .field("ancount", &self.ancount)
            .field("ncount", &self.ncount)
            .field("arcount", &self.arcount)
            .finish()
    }
}

///  In basic DNS, query messages should have empty answer, authority and additional sections.

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Flags(pub u16);

impl std::fmt::Debug for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Flags")
            .field(&&format_args!("0x{:x}", self.0))
            .finish()
    }
}

/// A one bit field that specifies whether this message is a query (0), or a response (1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QR {
    Query = 0,
    Response = 1,
}

/// Kind of query in this message.
/// This value is set by the originator of a query and copied into the response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opcode {
    Query = 0,
    IQuery = 1,
    Status = 2,
}

/// This bit is valid in responses, and specifies that the responding name server is an authority
/// for the domain name in question section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthoritativeAnswer {
    Query = 0,
    IQuery = 1,
    Status = 2,
}

/// Specifies that this message was truncated due to length greater than that permitted on the transmission channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrunCation {
    Entire = 0,
    Truncated = 1,
}

/// If RD is set, it directs the name server to pursue the query recursively.
/// Recursive query support is optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecursionDesired {
    NotDesired = 0,
    Desired = 1,
}

/// This be is set or cleared in a response, and denotes whether recursive query
/// support is available in the name server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecursionAvailable {
    NotAvailable = 0,
    Available = 1,
}

/// This 4 bit field is set as part of responses
#[derive(Debug, Clone, Copy, thiserror::Error)]
enum ResponseCode {
    #[error("No Error")]
    NoError = 0,

    #[error("Format Error")]
    FormErr = 1,

    #[error("Server Failure")]
    ServFail = 2,

    #[error("Non-Existent Domain")]
    NXDomain = 3,

    #[error("Not Implemented")]
    NotImp = 4,

    #[error("Query Refused")]
    Refused = 5,

    #[error("Name Exists when it should not")]
    YXDomain = 6,

    #[error("RR Set Exists when it should not")]
    YXRRSet = 7,

    #[error("RR Set that should exist does not")]
    NXRRSet = 8,

    #[error("Server Not Authoritative for zone")]
    NotAuth = 9,

    #[error("Name not contained in zone")]
    NotZone = 10,

    #[error("DSO-TYPE Not Implemented")]
    DSOTYPENI = 11,
}
