pub(crate) mod decode;
mod encode;

// Entrypoint Codecs; intended for public API usage
pub struct QueryCodec;

pub struct ResponseCodec;

// Internal implementation codecs; can be modified at will to fit the implementation
pub(crate) struct HeaderCodec;

pub(crate) struct QuestionCodec;
