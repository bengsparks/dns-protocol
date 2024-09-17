use std::io::{self, Read, Write};

use byteorder::{NetworkEndian, ReadBytesExt as _, WriteBytesExt};
use bytes::BufMut;

use super::{rotri, rtri};

use tokio_util::bytes::BytesMut;

// Names are represented as a sequence of labels, where each label consists of a
/// length octet followed by that number of octets.
/// The domain name terminates with the zero length octet for the null label of the root.
/// Note that this field may be an odd number of octets; no padding is used.
///
/// Domain names are subsets of ASCII, consisting of characters between a-z, A-Z, 0-9 and hypens.
#[derive(Clone, PartialEq, Eq)]
pub struct Name(pub(crate) Vec<u8>);

impl Name {
    pub(crate) fn decode<'a>(
        src: &mut io::Cursor<&'a [u8]>,
    ) -> Result<Option<Self>, io::Error> {
        let mut label_length = rtri!(src.read_u8());
        let mut expanded = Vec::with_capacity(label_length.into());
        loop {
            // End of stream
            if label_length == 0 {
                break;
            }
            // Uncompressed label
            else if label_length & 0b1100_0000 == 0 {
                let consumed = rtri!(src
                    .by_ref()
                    .take(label_length.into())
                    .read_to_end(&mut expanded));
                if consumed != label_length.into() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Unexpectedly encountered end of stream while parsing name",
                    ));
                }
            }
            // Compressed label
            else {
                let next = rtri!(src.read_u8());
                let combined = [label_length & 0b0011_1111, next];
                let pointer = rtri!(combined.as_ref().read_u16::<NetworkEndian>());

                let mut jmp = src.clone();
                jmp.set_position(pointer.into());

                // TODO: Safeguard against infinite recursion here
                let Name(decoded) = rotri!(Name::decode(&mut jmp));

                expanded.extend(decoded);
                break;
            }

            label_length = rtri!(src.read_u8());
            if label_length != 0 {
                expanded.push(b'.');
            }
        }

        Ok(Some(Name(expanded)))
    }

    pub(crate) fn encode(self, dst: &mut BytesMut) -> Result<(), io::Error> {
        let mut writer = dst.writer();

        for label in self.0.split(|v| *v == b'.') {
            writer.write_u8(label.len() as u8)?;
            writer.write_all(label)?;
        }
        writer.write_u8(0)?;


        Ok(())
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Name").field(&std::str::from_utf8(&self.0).unwrap()).finish()
    }
}

impl std::convert::TryFrom<Vec<u8>> for Name {
    type Error = io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO: Further restrict characters
        if !value.is_ascii() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Name is not ASCII",
            ));
        }

        Ok(Name(value))
    }
}

// https://www.freesoft.org/CIE/RFC/1035/43.htm
/*
#[test]
fn decode() {
    // Empty bytes for the purpose of testing offsets
    let mut bytes = vec![0u8; 20];

    bytes.extend(vec![
        1, b'F', 3, b'I', b'S', b'I', 4, b'A', b'R', b'P', b'A', 0,
    ]);
    let fisiarpa = bytes.len();

    bytes.extend(vec![3, b'F', b'O', b'O', 0b1100_0000, 20]);
    let foofisiarpa = bytes.len();

    bytes.extend(vec![0b1100_0000, 26]);
    let arpa = bytes.len();

    {
        let view = NameView {
            underlying: &bytes[20..fisiarpa],
            message_cursor: io::Cursor::new(&bytes),
        };
        let Name(decoded): Name = view.try_into().unwrap();
        assert_eq!(&b"F.ISI.ARPA"[..], decoded);
    }

    {
        let view = NameView {
            underlying: &bytes[fisiarpa..foofisiarpa],
            message_cursor: io::Cursor::new(&bytes),
        };
        let Name(decoded): Name = view.try_into().unwrap();
        assert_eq!(&b"FOO.F.ISI.ARPA"[..], decoded);
    }

    {
        let view = NameView {
            underlying: &bytes[foofisiarpa..arpa],
            message_cursor: io::Cursor::new(&bytes),
        };
        let Name(decoded): Name = view.try_into().unwrap();
        assert_eq!(&b"ARPA"[..], decoded);
    }
}
*/
