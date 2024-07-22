mod codec;
mod header;
mod name;
mod query;
mod question;
mod record;
mod response;

pub use codec::{QueryCodec, ResponseCodec};

pub use query::Query;
pub use response::Response;

pub use header::{Flags, Header};
pub use name::Name;
pub use question::{QClass, QType, Question};
pub use record::{Class, Type, Record};

pub(crate) use codec::decode::components::label;

#[cfg(test)]
mod tests;
