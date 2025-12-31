use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;

static STORE: OnceCell<VectorStore> = OnceCell::new();
#[derive(Clone)]
pub struct VectorEntry {
    pub embedding: Vec<f32>,
    pub text: String,
    pub section: String,
}

#[derive(Clone)]
pub struct VectorStore {
    entries: Arc<Mutex<Vec<VectorEntry>>>,
}

impl VectorStore {
    pub fn init_global() -> Self {
        let store = Self {
            entries: Arc::new(Mutex::new(Vec::new())),
        };
        let _ = STORE.set(store.clone());
        store
    }

    pub fn global() -> Self {
        STORE.get().expect("VectorStore not initialized").clone()
    }

    pub fn add(&self, entry: VectorEntry) {
    self.entries.lock().unwrap().push(entry);
    }

    pub fn all(&self) -> Vec<VectorEntry> {
        self.entries.lock().unwrap().clone()
    }

    // pub fn len(&self) -> usize {
    //     self.entries.lock().unwrap().len()
    // }
}