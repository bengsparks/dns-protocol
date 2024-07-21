mod codec;
mod header;
mod query;
mod question;

pub use codec::{HeaderCodec, QueryCodec, QuestionCodec};

pub use header::{Flags, Header};
pub use query::Query;
pub use question::{Class, Question, Type};

#[cfg(test)]
mod tests;
