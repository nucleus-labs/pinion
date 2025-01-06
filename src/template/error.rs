use derive_more::From;

use super::StoreIndex;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Native(minijinja::Error),

    SourceReadFailure(std::ffi::OsString),
    AlreadyInStore(StoreIndex),

    RenderFailure(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
