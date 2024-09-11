#![allow(dead_code)]
#![allow(clippy::arc_with_non_send_sync)]

use std::rc::Rc;
use std::sync::{ RwLock, Arc };
use std::collections::{HashMap, HashSet};

use crate::xml::{ NodeAsync, Node };


type AttributeSelector = Vec<(String, Option<String>)>;
type PseudoclassSelector = HashMap<String, Rc<NodeSelector>>;

#[derive(Clone)]
pub struct NodeSelector {
    namespace: Option<String>,                  // namespace()
    
    type_name: Option<String>,                  // named()
    id: Option<String>,                         // is()
    classes: Rc<[String]>,                      // classes()
    attributes: AttributeSelector,              // with()
    parent: Option<Box<NodeSelector>>,          // child()
    
    is_universal: bool,                         // NodeSelector::any()
    pseudoclasses: PseudoclassSelector,         // variant()
    
    visited: Arc<RwLock<HashSet<*const Node>>>,
}

impl NodeSelector {
    pub fn any() -> NodeSelector {
        Self{
            namespace: None,

            type_name: None,
            id: None,
            classes: [].into(),
            attributes: AttributeSelector::new(),
            parent: None,
            
            is_universal: true,
            pseudoclasses: PseudoclassSelector::new(),

            visited: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn new() -> NodeSelector {
        Self{
            namespace: None,

            type_name: None,
            id: None,
            classes: [].into(),
            attributes: AttributeSelector::new(),
            parent: None,
            
            is_universal: false,
            pseudoclasses: PseudoclassSelector::new(),

            visited: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    // supply functions

    /// tag name
    pub fn named(mut self, type_name: String) -> NodeSelector {
        self.type_name = Some(type_name);
        self
    }

    /// id
    pub fn is(mut self, id: String) -> NodeSelector {
        self.id = Some(id);
        self
    }

    /// classes
    pub fn classes(mut self, classes: Vec<String>) -> NodeSelector {
        self.classes = classes[..].into();
        self
    }

    /// *an* attribute
    pub fn with(mut self, attribute: String, value: Option<String>) -> NodeSelector {
        self.attributes.push((attribute, value));
        self
    }

    /// create a new node and make it this node's parent ; return the parent instead
    pub fn child(self) -> NodeSelector {
        let mut child = NodeSelector::new();
        child.parent = Some(Box::new(self));
        child
    }

    // stub for pseudoclasses
    pub fn variant(self, _pseudoclass: String) -> NodeSelector {
        self
    }

    pub fn lock(self) -> Arc<RwLock<NodeSelector>> {
        Arc::new(RwLock::new(self))
    }

    pub fn match_immediate(&mut self, node_handle: NodeAsync) -> bool {
        if self.is_universal { return true; }

        {
            if self.visited.read().unwrap().contains(&node_handle.to_ptr()) {
                return false;
            }
        }

        {
            self.visited.write().unwrap().insert(node_handle.clone().to_ptr());
        }
        let node_guard = (*node_handle).read().unwrap();

        // TODO: Namespace checks

        if self.type_name.as_ref().is_some_and(|x| *x != node_guard.name) {
            return false;
        }

        let node_id = node_guard.attributes.get("id").cloned();
        if self.id.is_some() && self.id != node_id {
            return false;
        }

        let node_classes_result = node_guard.attributes.get("class");
        if ! self.classes.is_empty() {
            if node_classes_result.is_none() {
                return false;
            }
            else {
                let node_classes: Vec<&str> = node_classes_result.unwrap().split(" ").collect();
                for class in self.classes.iter() {
                    if ! node_classes.contains(&class.as_str()) {
                        return false;
                    }
                }
            }
        }

        if ! self.attributes.is_empty() {
            if node_guard.attributes.is_empty() {
                return false;
            }
            else {
                for (attribute, value) in self.attributes.iter() {
                    if (! node_guard.attributes.contains_key(attribute))
                            || value.as_ref().is_some_and(|x| x != node_guard.attributes.get(attribute).unwrap()) {
                        return false;
                    }
                }
            }
        }

        if self.parent.is_some() {
            if node_guard.parent.is_none() {
                return false;
            }
            else {
                let selector_parent = self.parent.as_mut().unwrap();
                let node_parent_handle = node_handle.read().unwrap().parent.as_ref().unwrap().upgrade().unwrap();
                let node_parent_turned: NodeAsync = node_parent_handle.into();
                let match_result = selector_parent.match_immediate(node_parent_turned.clone());

                return match_result;
            }
        }

        true
    }

    fn search(&mut self, node: NodeAsync) -> Vec<NodeAsync> {
        let mut results = Vec::new();

        let mut current_node = Some(node);

        while let Some(node_handle) = current_node {
            if self.match_immediate(node_handle.clone()) {
                results.push(node_handle.clone());
                println!("found match: {}", node_handle);
            }

            let parent = node_handle.read().unwrap().parent.clone();
            current_node = parent.map(|parent_handle| parent_handle.upgrade().unwrap().into());
        }

        results
    }

    pub fn clear_visited(&self) {
        self.visited.write().unwrap().clear();
        if self.parent.is_some() {
            self.parent.as_deref().unwrap().clear_visited();
        }
    }

    pub fn apply(&mut self, node: NodeAsync) -> Vec<NodeAsync> {
        self.clear_visited();
        let mut results = Vec::new();

        for leaf in node.get_leaves().iter() {
            results.append(&mut self.search(leaf.clone()));
        }
    
        results
    }
}

impl Default for NodeSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for NodeSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut result = String::new();

        result += self.type_name.as_deref().unwrap_or("");

        // TODO: more

        if self.parent.is_some() {
            let parent_str = self.parent.as_ref().unwrap().to_string();
            result = format!("{} > {}", parent_str, result);
        }

        write!(f, "{}", result)
    }
}
