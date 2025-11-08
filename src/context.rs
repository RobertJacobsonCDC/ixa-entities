use crate::{entity_store::EntityStore, property_store::PropertyStore};

/// A minimalist stand-in for a `Context` object.
pub struct Context {
    #[cfg(feature = "entity_store")]
    pub entity_store: EntityStore,
    pub property_store: PropertyStore,
}

impl Context {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "entity_store")]
            entity_store: EntityStore::new(),
            property_store: PropertyStore::new(),
        }
    }
}
