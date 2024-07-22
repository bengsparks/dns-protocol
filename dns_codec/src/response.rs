#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub header: crate::Header,
    pub questions: Vec<crate::Question>,
    pub answers: Vec<crate::Record>,
    pub authorities: Vec<crate::Record>,
    pub additionals: Vec<crate::Record>,
}
