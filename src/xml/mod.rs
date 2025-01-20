mod error;

use xmltree;

use std::cell::OnceCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak};
use std::vec::Vec;

use crate::template;
use crate::AsyncHandle;

pub use error::{Error, SourceReadFailureContents};

pub type StoreIndex = String;
pub type Namespace = String;

#[derive(Debug)]
pub struct XmlNode {
    pub prefix: Option<String>,
    pub namespace: Option<Namespace>,
    pub namespaces: Option<BTreeMap<String, String>>,

    pub name: String,
    pub attributes: HashMap<(Namespace, String), String>,

    pub children: Vec<NodeAsync>,
    pub parent: Option<Weak<RwLock<XmlNode>>>,
}

#[derive(Debug, Clone)]
pub struct NodeAsync(AsyncHandle<XmlNode>);

#[derive(Debug)]
pub struct StoreEntry {
    pub store: Weak<RwLock<XmlStore>>,
    pub index: StoreIndex,
    pub nodes: Arc<[NodeAsync]>,
    pub source: String,
}
pub type StoreEntryAsync = AsyncHandle<StoreEntry>;

#[derive(Debug, Clone)]
pub struct XmlStore {
    pub indices: AsyncHandle<HashMap<StoreIndex, StoreEntryAsync>>,
    handle: OnceCell<Arc<RwLock<Self>>>,
}

impl XmlNode {
    pub fn has_attribute(&self, namespace: &str, attribute: &str) -> bool {
        let key = &(namespace.into(), attribute.into());
        self.attributes.contains_key(key)
    }

    pub fn get_attribute(&self, namespace: &str, attribute: &str) -> Option<String> {
        let key = &(namespace.into(), attribute.into());
        self.attributes.get(key).cloned()
    }
}

impl NodeAsync {
    pub fn to_ptr(&self) -> *const XmlNode {
        let arc = &self.0;
        let node_ptr = Arc::as_ptr(arc);

        // UNSAFE: dereference arc to get raw pointer to Node
        // this assumes the Arc is not dropped while being used.
        let node_ref = unsafe { (*node_ptr).read().unwrap() };
        &*node_ref as *const XmlNode
    }

    pub fn get_leaves(&self) -> Arc<[Self]> {
        let mut stack: Vec<Self> = vec![self.clone()];
        let mut leaves: Vec<Self> = vec![];

        while let Some(node) = stack.pop() {
            let node_guard = node.read().unwrap();
            if node_guard.children.is_empty() {
                leaves.push(node.clone());
            } else {
                for child in node_guard.children.iter() {
                    stack.push(child.clone());
                }
            }
        }

        leaves[..].into()
    }
}

impl Deref for NodeAsync {
    type Target = Arc<RwLock<XmlNode>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for NodeAsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut result = String::new();

        {
            result += &self.read().unwrap().name;
        }

        // TODO: more

        let parent: &Option<Weak<RwLock<XmlNode>>> = &self.read().unwrap().parent;
        if parent.is_some() {
            let parent_handle = parent.as_ref().unwrap();
            let parent_turned: NodeAsync = parent_handle.upgrade().unwrap().into();
            let parent_str = parent_turned.to_string();
            result = format!("{} > {}", parent_str, result);
        }

        write!(f, "{}", result)
    }
}

impl From<XmlNode> for NodeAsync {
    fn from(alt: XmlNode) -> Self {
        Self(Arc::new(RwLock::new(alt)))
    }
}

impl From<Arc<RwLock<XmlNode>>> for NodeAsync {
    fn from(value: Arc<RwLock<XmlNode>>) -> Self {
        Self(value)
    }
}

impl From<xmltree::Element> for NodeAsync {
    fn from(native_node: xmltree::Element) -> Self {
        // change empty namespace key "" into namespace key "Default"
        let namespaces: Option<BTreeMap<String, String>> = match native_node.namespaces {
            None => None,
            Some(namespaces) => {
                let mut result = namespaces.0.clone();
                let mut bad_key: Option<String> = None;
                for key in result.keys() {
                    if key.is_empty() {
                        bad_key = Some(key.clone());
                    }
                }
                if let Some(bad_key_contents) = bad_key {
                    result.insert("Default".into(), result[&bad_key_contents].clone());
                    result.remove(&bad_key_contents);
                }

                Some(result)
            }
        };

        let namespace: Option<String> = match native_node.namespace {
            Some(namespace) => {
                let mut result: Option<String> = None;

                if namespaces.is_some() {
                    for (key, value) in namespaces.as_ref().unwrap().iter() {
                        if *value == namespace {
                            result = Some(key.into());
                            break;
                        }
                    }
                }

                result
            }
            None => None,
        };

        let mut attributes: HashMap<(Namespace, String), String> = HashMap::new();
        // k: String, v: String
        for (k, v) in native_node.attributes.into_iter() {
            let (mut namespace, name) = k.split_at(k.find(":").unwrap_or(0usize));
            if namespace.is_empty() {
                namespace = "Default";
            }
            attributes.insert((namespace.into(), name.into()), v);
        }
        attributes.entry(("Default".into(), "id".into())).or_insert_with(|| format!("pk-{}", uuid::Uuid::new_v4()));

        let node: NodeAsync = Self(Arc::new(RwLock::new(XmlNode {
            prefix: native_node.prefix,
            namespace,
            namespaces,

            name: native_node.name,
            attributes,

            children: Vec::default(),
            parent: None,
        })));

        let mut children: Vec<NodeAsync> = Vec::new();

        for child in native_node.children.iter() {
            match child {
                xmltree::XMLNode::Text(content) => {
                    let mut text_element = xmltree::Element::new("text-content");
                    text_element.namespace = Some("Default".into());
                    text_element.attributes.insert("id".into(), format!("pk-{}", uuid::Uuid::new_v4()));
                    text_element.attributes.insert("content".into(), content.trim().to_string());

                    let child_turned = NodeAsync::from(text_element);
                    child_turned.write().unwrap().parent = Some(Arc::downgrade(&node.0));
                    children.push(child_turned);
                }
                xmltree::XMLNode::Element(element) => {
                    let child_turned = NodeAsync::from(element.clone());
                    child_turned.write().unwrap().parent = Some(Arc::downgrade(&node.0));
                    children.push(child_turned);
                },
                _ => continue,
            }
        }

        {
            node.write().unwrap().children.append(&mut children);
        }

        node
    }
}

impl XmlStore {
    pub fn new() -> Arc<RwLock<XmlStore>> {
        let store = XmlStore {
            #[allow(clippy::arc_with_non_send_sync)]
            indices: Arc::new(RwLock::new(HashMap::new())),
            handle: OnceCell::new(),
        };

        #[allow(clippy::arc_with_non_send_sync)]
        let arc: Arc<RwLock<XmlStore>> = Arc::new(RwLock::new(store));

        arc.write().unwrap().handle.set(arc.clone()).unwrap();
        arc.clone()
    }

    pub fn get_handle(&self) -> Arc<RwLock<XmlStore>> {
        self.handle.get().unwrap().clone()
    }

    pub fn append_from_template(
        &mut self,
        index: StoreIndex,
        template: template::StoreEntryAsync,
    ) -> Result<StoreEntryAsync, Error> {
        if self.has(index.clone()) {
            Err(Error::AlreadyInStore(index))
        } else {
            let mut store_guard = self.indices.write().unwrap();
            let readable_guard = template.read().unwrap();

            match xmltree::Element::parse_all(readable_guard.source.as_bytes()) {
                Ok(nodes_vec) => {
                    let nodes_async_vec: Vec<NodeAsync> = nodes_vec
                        .into_iter()
                        .filter(|x| matches!(x, xmltree::XMLNode::Element(_)))
                        .map(|x| NodeAsync::from((*x.as_element().unwrap()).to_owned()))
                        .collect();

                    #[allow(clippy::arc_with_non_send_sync)]
                    let store_entry: StoreEntryAsync = Arc::new(RwLock::new(StoreEntry {
                        store: Arc::downgrade(&self.get_handle()),
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
                }
                Err(err) => Err(Error::SourceReadFailure(SourceReadFailureContents {
                    entry_index: index,
                    failure_message: err.to_string(),
                })),
            }
        }
    }

    pub fn append_from_source(
        &mut self,
        index: StoreIndex,
        source: String,
    ) -> Result<StoreEntryAsync, Error> {
        if self.has(index.clone()) {
            Err(Error::AlreadyInStore(index))
        } else {
            let mut store_guard = self.indices.write().unwrap();

            match xmltree::Element::parse_all(source.as_bytes()) {
                Ok(nodes_vec) => {
                    let nodes_async_vec: Vec<NodeAsync> = nodes_vec
                        .into_iter()
                        .filter(|x| matches!(x, xmltree::XMLNode::Element(_)))
                        .map(|x| NodeAsync::from((*x.as_element().unwrap()).to_owned()))
                        .collect();

                    #[allow(clippy::arc_with_non_send_sync)]
                    let store_entry: StoreEntryAsync = Arc::new(RwLock::new(StoreEntry {
                        store: Arc::downgrade(&self.get_handle()),
                        nodes: nodes_async_vec[..].into(),
                        index,
                        source: source.clone(),
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
                }
                Err(err) => Err(Error::SourceReadFailure(SourceReadFailureContents {
                    entry_index: index,
                    failure_message: err.to_string(),
                })),
            }
        }
    }

    pub fn has(&self, index: StoreIndex) -> bool {
        let indices_guard = self.indices.read().unwrap();
        indices_guard.contains_key(&index)
    }

    pub fn get(&self, index: StoreIndex) -> Option<StoreEntryAsync> {
        let indices_guard = self.indices.read().unwrap();
        indices_guard.get(&index).cloned()
    }
}
