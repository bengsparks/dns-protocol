pub struct Resource {
    /// A domain name to which this resource record pertains.
    pub name: Vec<u8>,

    /// Contains one of the RR type codes. This field specifies the meaning of the data in the RDATA field.
    pub kind: crate::Type,

    /// Specifies the class of the data in the RDATA field.
    pub class: [u8; 2],

    /// Specifies the time interval (in seconds) that the resource record may be cached before it should be discarded.
    /// Zero values are interpreted to mean that the RR can only be used for the transaction in progress, and should not be cached.
    pub ttl: [u8; 4],

    /// Specifies the length in octets of the RDATA field.
    pub rdlength: [u8; 2],

    /// A variable length string of octets that describes the resource.
    /// The format of this information varies according to the TYPE and CLASS of the resource record.
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    pub rdata: Vec<u8>,
}
