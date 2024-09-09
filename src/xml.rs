
use xmltree as Xml;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::ops::Deref;
use std::vec::Vec;

use crate::AsyncHandle;
use crate::template as Templating;

pub type StoreIndex = String;

#[derive(Debug)]
pub struct SourceReadFailureContents {
    entry_index: StoreIndex,
    failure_message: String,
}

#[derive(Debug)]
pub enum Error {
    Native(String),
    SourceReadFailure(SourceReadFailureContents),
    AlreadyInStore(StoreIndex),
}

pub struct Node {
    pub text_content: Option<String>,

    pub prefix: Option<String>,
    pub namespace: Option<String>,
    pub namespaces: Option<Xml::Namespace>,
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<NodeAsync>,

    pub parent: Option<Weak<RwLock<Node>>>,
}
#[derive(Clone)]
pub struct NodeAsync(AsyncHandle<Node>);

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

impl Node {
    fn get_leaves(&self) -> Arc<[NodeAsync]> { todo!() }
}

impl From<Node> for NodeAsync {
    fn from(alt: Node) -> NodeAsync {
        NodeAsync(Arc::new(RwLock::new(alt)))
    }
}

impl Deref for NodeAsync {
    type Target = Arc<RwLock<Node>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Xml::Element> for NodeAsync {
    fn from(native_node: Xml::Element) -> Self {
        let node = NodeAsync(Arc::new(RwLock::new(Node{
            text_content: None,

            prefix: native_node.prefix,
            namespace: native_node.namespace,
            namespaces: native_node.namespaces,
            name: native_node.name,
            attributes: native_node.attributes,

            children: Vec::default(),
            parent: None,
        })));

        let mut children: Vec<NodeAsync> = Vec::new();

        let mut text_content = String::new();

        for child in native_node.children.iter() {
            match child {
                Xml::XMLNode::Element(_) => (),
                Xml::XMLNode::Text(content) => {
                    text_content += content;
                    text_content += "\n";
                    continue;
                },
                _ => continue,
            }

            // assumption safe because of above matching
            let element_unwrapped = child.as_element().unwrap();

            let child_turned = NodeAsync::from(element_unwrapped.to_owned());
            child_turned.0.write().unwrap().parent = Some(Arc::downgrade(&node.0));

            children.push(child_turned.into());
        }

        if ! text_content.is_empty() {
            node.0.write().unwrap().text_content = Some(text_content);
        }

        node
    }
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
                        .filter(|x| matches!(x, Xml::XMLNode::Element(_)))
                        .map(|x| NodeAsync::from((*x.as_element().unwrap()).to_owned()))
                        .collect();

                    let store_entry: StoreEntryAsync = Arc::new(RwLock::new(StoreEntry{
                        store: Arc::downgrade(self),
                        nodes: nodes_async_vec[..].into(),
                        index,
                        source: readable_guard.source.clone(),
                    }));
                    let entry_index: StoreIndex;
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
        indices_guard.contains_key(&index)
    }

    pub fn get(self: &Arc<Self>, index: StoreIndex) -> Option<StoreEntryAsync> {
        let indices_guard = self.indices.read().unwrap();
        match indices_guard.get(&index) {
            None => None,
            Some(entry) => Some(entry.clone())
        }
    }
}
