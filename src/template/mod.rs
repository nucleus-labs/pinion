mod error;

use std::cell::OnceCell;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock, Weak};

use crate::{AsyncHandle, Result};
pub use error::Error;

type StoreIndex = String;

#[derive(Debug)]
pub struct StoreEntry<'a> {
    store: Weak<RwLock<TemplateStore<'a>>>,
    index: StoreIndex,
    pub source: String,
}

pub type StoreEntryAsync<'a> = AsyncHandle<StoreEntry<'a>>;

#[derive(Debug)]
pub struct TemplateStore<'a> {
    pub env: AsyncHandle<minijinja::Environment<'a>>,
    indices: AsyncHandle<HashMap<StoreIndex, StoreEntryAsync<'a>>>,
    handle: OnceCell<Arc<RwLock<Self>>>,
}

impl<'a> StoreEntry<'a> {
    pub fn render(&self, context: minijinja::Value) -> Result<String> {
        let store_handle = self.store.upgrade().unwrap();
        let store_guard = store_handle.read().unwrap();
        let env_guard = store_guard.env.read().unwrap();
        match env_guard.get_template(&self.index) {
            Ok(template) => match template.render(context) {
                Ok(rendered) => Ok(rendered),
                Err(err) => Err(Error::RenderFailure(err.to_string()).into()),
            },
            Err(_) => todo!(),
        }
    }
}

impl<'a> TemplateStore<'a> {
    pub fn new() -> Arc<RwLock<TemplateStore<'a>>> {
        let store = Self {
            env: Arc::new(RwLock::new(minijinja::Environment::new())),
            #[allow(clippy::arc_with_non_send_sync)]
            indices: Arc::new(RwLock::new(HashMap::new())),
            handle: OnceCell::new(),
        };

        #[allow(clippy::arc_with_non_send_sync)]
        let arc: Arc<RwLock<Self>> = Arc::new(RwLock::new(store));

        arc.write().unwrap().handle.set(arc.clone()).unwrap();
        arc.clone()
    }

    pub fn get_handle(&self) -> Arc<RwLock<Self>> {
        self.handle.get().unwrap().clone()
    }

    pub fn append_raw(&self, index: StoreIndex, source: String) -> Result<StoreEntryAsync<'a>> {
        if self.has(&index) {
            Err(Error::AlreadyInStore(index.clone()).into())
        } else {
            #[allow(clippy::arc_with_non_send_sync)]
            let entry: StoreEntryAsync<'a> = Arc::new(RwLock::new(StoreEntry {
                store: Arc::downgrade(&self.get_handle()),
                index: index.clone(),
                source,
            }));

            let mut store_guard = self.indices.write().unwrap();
            store_guard.insert(index.clone(), entry.clone());

            let mut env_guard = self.env.write().unwrap();
            let result =
                env_guard.add_template_owned(index, entry.read().unwrap().source.clone());
            match result {
                Ok(_) => Ok(entry),
                Err(err) => Err(Error::Native(err).into()),
            }
        }
    }

    pub fn append_from_file(&self, index: StoreIndex, path: &std::path::Path) -> Result<StoreEntryAsync<'a>> {
        if self.has(&index) {
            Err(Error::AlreadyInStore(index.clone()).into())
        } else {
            let mut store_guard = self.indices.write().unwrap();

            match fs::read_to_string(path) {
                Ok(source) => {
                    #[allow(clippy::arc_with_non_send_sync)]
                    let entry: StoreEntryAsync<'a> = Arc::new(RwLock::new(StoreEntry {
                        store: Arc::downgrade(&self.get_handle()),
                        index: index.clone(),
                        source,
                    }));

                    store_guard.insert(index.clone(), entry.clone());

                    let mut env_guard = self.env.write().unwrap();
                    let result =
                        env_guard.add_template_owned(index, entry.read().unwrap().source.clone());
                    match result {
                        Ok(_) => Ok(entry),
                        Err(err) => Err(Error::Native(err).into()),
                    }
                }
                Err(_) => Err(Error::SourceReadFailure(path.into()).into()),
            }
        }
    }

    pub fn has(&self, index: &StoreIndex) -> bool {
        let indices_guard = self.indices.read().unwrap();
        indices_guard.contains_key(index)
    }

    pub fn get(&self, index: &StoreIndex) -> StoreEntryAsync<'a> {
        self.indices
            .read()
            .unwrap()
            .get(index)
            .unwrap_or_else(|| panic!("Tried to get non-existent index '{}'", index))
            .clone()
    }
}
