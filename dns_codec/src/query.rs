#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub header: crate::Header,
    pub question: crate::Question,
}
