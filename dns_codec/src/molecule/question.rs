use std::io::{self};

use crate::atom::{self, rotri, Name, QClass, QType};

use tokio_util::bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub name: Name,

    /// Specifies the type of the query. The values for this field include all codes valid for a TYPE field,
    /// together with some more general codes which can match more than one type of RR.
    pub kind: QType,

    /// Specifies the class of the query.
    /// For example, the QCLASS field is IN for the Internet.
    pub class: QClass,
}

impl Question {
    pub(crate) fn decode<'a>(src: &mut io::Cursor<&'a [u8]>) -> Result<Option<Self>, io::Error> {
        let name = rotri!(Name::decode(src));
        let kind = rotri!(QType::decode(src));
        let class = rotri!(QClass::decode(src));

        let question = Question { name, kind, class };
        Ok(Some(question))
    }

    pub(crate) fn encode(self, dst: &mut BytesMut) -> Result<(), io::Error> {
        self.name.encode(dst)?;
        self.kind.encode(dst)?;
        self.class.encode(dst)?;

        Ok(())
    }
}

impl std::convert::From<atom::Type> for QType {
    fn from(value: crate::Type) -> Self {
        (value as u16).try_into().unwrap()
    }
}
