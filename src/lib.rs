mod error;
pub mod template;
pub mod xml;

pub use error::{Error, Result};

use std::sync::{Arc, RwLock};

type AsyncHandle<T> = Arc<RwLock<T>>;

pub use template::TemplateStore;
pub use xml::XmlStore;
