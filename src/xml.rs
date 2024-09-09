
use xmltree as Xml;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::vec::Vec;

use crate::AsyncHandle;
use crate::template as Templating;

pub type StoreIndex = String;

#[derive(Debug)]
pub struct SourceReadFailureContents {
    entry_index: String,
    failure_message: String,
}

#[derive(Debug)]
pub enum Error {
    Native(String),
    SourceReadFailure(SourceReadFailureContents),
    AlreadyInStore(String),
}

pub type Node = Xml::XMLNode;
pub type NodeAsync = AsyncHandle<Node>;

pub struct StoreEntry {
    pub store: Weak<Store>,
    pub index: StoreIndex,
    pub nodes: Arc<[NodeAsync]>,
    pub source: String,
}
pub type StoreEntryAsync = AsyncHandle<StoreEntry>;

pub struct Store {
    indices: AsyncHandle<HashMap<StoreIndex, StoreEntryAsync>>
}

impl Store {
    pub fn new() -> Arc<Store> {
        Store{
            indices: Arc::new(RwLock::new(HashMap::new()))
        }.into()
    }

    pub fn append(self: &Arc<Self>, index: StoreIndex, template: Templating::StoreEntryAsync) -> Result<StoreEntryAsync, Error> {
        if self.has(index.clone()) {
            return Err(Error::AlreadyInStore(index.into()));
        }
        else {
            let mut store_guard = self.indices.write().unwrap();
            let readable_guard = template.read().unwrap();

            match Xml::Element::parse_all(readable_guard.source.as_bytes()) {
                Ok(nodes_vec) => {
                    let nodes_async_vec: Vec<NodeAsync> = nodes_vec.into_iter()
                        .map(|x| Arc::new(RwLock::new(x)))
                        .collect();

                    let store_entry: StoreEntryAsync = Arc::new(RwLock::new(StoreEntry{
                        store: Arc::downgrade(self),
                        nodes: nodes_async_vec[..].into(),
                        index,
                        source: readable_guard.source.clone(),
                    }));
                    let entry_index: String;
                    {
                        let entry_guard = store_entry.read().unwrap();
                        entry_index = entry_guard.index.clone();
                    }
                    match store_guard.insert(entry_index.clone(), store_entry.clone()) {
                        Some(_) => Err(Error::AlreadyInStore(entry_index)),
                        None => Ok(store_entry),
                    }
                },
                Err(err) => Err(Error::SourceReadFailure(SourceReadFailureContents{
                    entry_index: index.into(),
                    failure_message: err.to_string(),
                }))
            }
        }
    }

    pub fn has(self: &Arc<Self>, index: StoreIndex) -> bool {
        let indices_guard = self.indices.read().unwrap();
        return indices_guard.contains_key(&index);
    }

    pub fn get(self: &Arc<Self>, index: StoreIndex) -> Option<StoreEntryAsync> {
        let indices_guard = self.indices.read().unwrap();
        match indices_guard.get(&index) {
            None => None,
            Some(entry) => Some(entry.clone())
        }
    }
}
