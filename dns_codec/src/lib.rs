mod codec;
mod header;
mod name;
mod query;
mod question;
mod record;
mod response;

pub use codec::{QueryCodec, ResponseCodec};

pub use query::Query;
pub(crate) use response::Metadata;
pub use response::{Response, ResponseBytes};

pub use header::{Flags, Header};
pub use name::Name;
pub use question::{QClass, QType, Question};
pub use record::{Class, Record, Type};

#[cfg(test)]
mod tests;
