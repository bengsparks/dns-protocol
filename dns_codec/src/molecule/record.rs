use std::io;

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::{
    atom::{rotri, rtri},
    Class, Name, QType, RData, Ttl, Type,
};

#[derive(Debug)]
pub struct Record {
    /// A domain name to which this resource record pertains.
    pub name: Name,

    /// Contains one of the RR type codes. This field specifies the meaning of the data in the RDATA field.
    pub kind: Type,

    /// Specifies the class of the data in the RDATA field.
    pub class: Class,

    /// Specifies the time interval (in seconds) that the resource record may be cached before it should be discarded.
    /// Zero values are interpreted to mean that the RR can only be used for the transaction in progress, and should not be cached.
    pub ttl: Ttl,

    /// Specifies the length in octets of the RDATA field.
    pub length: u16,

    /// A variable length string of octets that describes the resource.
    /// The format of this information varies according to the TYPE and CLASS of the resource record.
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    pub rdata: RData,
}

impl Record {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let name = rotri!(Name::decode(src));
        log::trace!("{name:?}");
        
        let kind = rotri!(Type::decode(src));
        log::trace!("{kind:?}");
        
        let class = rotri!(Class::decode(src));
        log::trace!("{class:?}");
        
        let ttl = rotri!(Ttl::decode(src));
        log::trace!("{ttl:?}");

        let length = rtri!(src.read_u16::<NetworkEndian>());
        let rdata = rotri!(RData::decode(src, length, kind, class));

        let record = Record {
            name,
            kind,
            class,
            ttl,
            length,
            rdata,
        };
        Ok(Some(record))
    }
}

impl PartialEq<QType> for crate::Type {
    fn eq(&self, other: &QType) -> bool {
        (*self as u16) == (*other as u16)
    }
}

impl PartialEq<Type> for QType {
    fn eq(&self, other: &crate::Type) -> bool {
        (*self as u16) == (*other as u16)
    }
}
