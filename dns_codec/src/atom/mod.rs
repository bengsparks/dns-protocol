//! All components required for representing queries and response.
//! All

mod class;
mod header;
mod name;
mod qclass;
mod qtype;
mod rdata;
mod ttl;
mod r#type;

// Owned representations
pub use class::Class;
pub use header::Header;
pub use name::Name;
pub use qclass::QClass;
pub use qtype::QType;
pub use r#type::Type;
pub use rdata::RData;
pub use ttl::Ttl;

/// Module-local macro for converting Err(std::io::ErrorKind::UnexpectedEof) in Result<T, E> into Ok(None) for [`tokio_util::codec::Decoder`]
macro_rules! rtri {
    ( $x:expr ) => {
        match $x {
            Ok(v) => v,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e)?,
        }
    };
}
pub(crate) use rtri;

/// Module-local macro for converting Err(std::io::ErrorKind::UnexpectedEof) in Result<Option<T>, E> into Ok(None) for [`tokio_util::codec::Decoder`]
macro_rules! rotri {
    ( $x:expr ) => {
        match $x {
            Ok(Some(v)) => v,
            Ok(None) => return Ok(None),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e)?,
        }
    };
}
pub(crate) use rotri;
