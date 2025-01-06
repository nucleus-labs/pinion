use derive_more::From;

use super::StoreIndex;

#[derive(Debug)]
pub struct SourceReadFailureContents {
    pub entry_index: StoreIndex,
    pub failure_message: String,
}

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Native(xmltree::Error),

    #[from]
    SourceReadFailure(SourceReadFailureContents),

    AlreadyInStore(StoreIndex),
}

impl std::fmt::Display for SourceReadFailureContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.failure_message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SourceReadFailureContents {}
impl std::error::Error for Error {}
