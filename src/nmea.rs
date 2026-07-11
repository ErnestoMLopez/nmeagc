#[derive(Clone, Debug)]
pub struct RawNmeaLog {
    pub sentence: String,
    pub status: RawNmeaStatus,
}

#[derive(Clone, Debug)]
pub enum RawNmeaStatus {
    Gnss,
    Other,
    Incomplete,
    Error,
}
