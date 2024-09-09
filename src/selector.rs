
use std::sync::{ RwLock, Arc };
use std::collections::HashMap;

type AttributeSelector = HashMap<&'static str, &'static str>;
type PseudoclassSelector = HashMap<&'static str, Arc<NodeSelector>>;

pub struct NodeSelector {
    namespace: Option<String>,                  // namespace()

    type_name: RwLock<Option<&'static str>>,    // named()
    id: RwLock<Option<&'static str>>,           // is()
    classes: RwLock<Arc<[&'static str]>>,       // classes()
    attributes: RwLock<AttributeSelector>,      // with()
    parent: RwLock<Option<Arc<NodeSelector>>>,  // child()
    
    is_universal: RwLock<bool>,                 // NodeSelector::any()
    pseudoclasses: RwLock<PseudoclassSelector>, // variant()
}

pub trait MatchSelector {
    fn matches(&self, selection: &NodeSelector) -> bool;
}

impl NodeSelector {
    pub fn any() -> Arc<NodeSelector> {
        Arc::new(Self{
            namespace: None,

            type_name: None.into(),
            id: None.into(),
            classes: RwLock::new(Arc::new([])),

            is_universal: RwLock::new(true),

            attributes: RwLock::new(AttributeSelector::new()),

            parent: RwLock::new(None),
            pseudoclasses: RwLock::new(PseudoclassSelector::new()),
        })
    }

    pub fn new() -> Arc<NodeSelector> {
        Arc::new(Self{
            namespace: None,

            type_name: None.into(),
            id: None.into(),
            classes: RwLock::new(Arc::new([])),

            is_universal: RwLock::new(false),

            attributes: RwLock::new(AttributeSelector::new()),

            parent: RwLock::new(None),
            pseudoclasses: RwLock::new(PseudoclassSelector::new()),
        })
    }

    // supply functions

    /// tag name
    pub fn named(self: &Arc<Self>, type_name: &'static str) -> Arc<NodeSelector> {
        *self.type_name.write().unwrap() = Some(type_name);
        self.clone()
    }

    /// id
    pub fn is(self: &Arc<Self>, id: &'static str) -> Arc<NodeSelector> {
        *self.id.write().unwrap() = Some(id);
        self.clone()
    }

    /// classes
    pub fn classes(self: &Arc<Self>, classes: Vec<&'static str>) -> Arc<NodeSelector> {
        *self.classes.write().unwrap() = classes[..].into();
        self.clone()
    }

    /// *an* attribute
    pub fn with(self: &Arc<Self>, attribute: &'static str, value: &'static str) -> Arc<NodeSelector> {
        self.attributes.write().unwrap().insert(attribute, value);
        self.clone()
    }

    /// create a new node and make it this node's parent ; return the parent instead
    pub fn child(self: &Arc<Self>) -> Arc<NodeSelector> {
        let parent = NodeSelector::new();
        *self.parent.write().unwrap() = Some(parent.clone());
        parent
    }
}
