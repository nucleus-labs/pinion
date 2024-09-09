#![allow(dead_code)]

pub mod template;
pub mod selector;
pub mod xml;

use std::sync::{ RwLock, Arc };

type AsyncHandle<T> = Arc<RwLock<T>>;

#[derive(Debug)]
pub enum QuillError {
    Template(template::Error),
    Xml(xml::Error),
    
    Usage(String),
    Generic(String),
}

// fn children_match(children: Vec<xml::NodeAsync>, selection: selector::NodeSelector) -> bool {
//     todo!()
// }

// impl selector::MatchSelector for xml::Node {
//     fn matches(&self, selection: &selector::NodeSelector) -> bool {
//         todo!()
//     }
// }

// impl selector::MatchSelector for xml::StoreEntry {
//     fn matches(&self, selection: &selector::NodeSelector) -> bool {
//         for node in self.nodes.iter() {
//             if ! node.read().unwrap().matches(selection) {
//                 return false;
//             }
//         }

//         true
//     }
// }
