
use minijinja as Jinja;

pub use Jinja::{ context, Value };

use std::collections::HashMap;
use std::sync::{ RwLock, Arc, Weak };
use std::ffi::{ OsString, OsStr };
use std::fs;

use crate::AsyncHandle;

type StoreIndex = String;

#[derive(Debug)]
pub enum Error {
    Native(String),
    SourceReadFailure(OsString),
    AlreadyInStore(StoreIndex),
    RenderFailure(String),
}

#[derive(Debug)]
pub struct StoreEntry<'a>
{
    store: Weak<Store<'a>>,
    index: StoreIndex,
    pub source: String,
}

pub type StoreEntryAsync<'a> = AsyncHandle<StoreEntry<'a>>;

pub struct Store<'a> {
    pub env: AsyncHandle<Jinja::Environment<'a>>,
    indices: AsyncHandle<HashMap<String, StoreEntryAsync<'a>>>,
}

impl<'a> StoreEntry<'a> {
    pub fn render(self: Arc<StoreEntry<'a>>, context: Value) -> Result<String, Error> {
        let store_handle = self.store.upgrade().unwrap();
        let env_guard = store_handle.env.read().unwrap();
        match env_guard.get_template(&self.index) {
            Ok(template) => {
                match template.render(context) {
                    Ok(rendered) => Ok(rendered),
                    Err(err) => Err(Error::RenderFailure(err.to_string()))
                }
            },
            Err(_) => todo!()
        }
    }
}

impl<'a> Store<'a> {
    pub fn new() -> Arc<Store<'a>> {
        Store{
            env: Arc::new(RwLock::new(Jinja::Environment::new())),
            indices: Arc::new(RwLock::new(HashMap::new())),
        }.into()
    }

    pub fn append(self: &Arc<Self>, index: StoreIndex, path: &'a OsStr) -> Result<StoreEntryAsync<'a>, Error> {
        if self.has(index.clone()) {
            Err(Error::AlreadyInStore(index.clone()))
        }
        else {
            let mut store_guard = self.indices.write().unwrap();
    
            match fs::read_to_string(path) {
                Ok(source) => {
                    let entry: StoreEntryAsync<'a> = Arc::new(RwLock::new(StoreEntry{
                        store: Arc::downgrade(self),
                        index: index.clone(),
                        source,
                    }));

                    store_guard.insert(index.clone(), entry.clone());
    
                    let mut env_guard = self.env.write().unwrap();
                    let result = env_guard.add_template_owned(index, entry.read().unwrap().source.clone());
                    match result {
                        Ok(_) => Ok(entry),
                        Err(err) => Err(Error::Native(err.to_string())),
                    }
                },
                Err(_) => Err(Error::SourceReadFailure(path.into())),
            }
        }
    }

    pub fn has(self: &Arc<Self>, index: StoreIndex) -> bool {
        let indices_guard = self.indices.read().unwrap();
        return indices_guard.contains_key(&index);
    }

    pub fn get(self: &Arc<Self>, index: StoreIndex) -> StoreEntryAsync<'a> {
        self.indices.read().unwrap().get(&index)
            .expect(format!("Tried to get non-existent index '{}'", index).as_str())
            .clone()
    }
}
