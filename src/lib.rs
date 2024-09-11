
mod error;
pub mod template;
pub mod selector;
pub mod xml;

pub use error::{ Error, Result };

use std::sync::{ RwLock, Arc };

type AsyncHandle<T> = Arc<RwLock<T>>;
