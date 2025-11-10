use crate::{
    entity::{Entity, EntityId},
    entity_store::EntityStore,
    property_list::PropertyList,
    property_store::PropertyStore,
};

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

    pub fn add_entity<E: Entity, PL: PropertyList<E>>(&mut self, property_list: PL) -> EntityId<E> {
        // Check that the properties in the list are distinct.
        if let Err(msg) = PL::validate() {
            panic!("invalid property list: {}", msg);
        }
        // Check that all required properties are present.
        if !PL::contains_required_properties() {
            panic!("initialization list is missing required properties");
        }

        // Now that we know we will succeed, we create the entity.
        let new_entity_id = self.entity_store.new_entity_id::<E>();

        new_entity_id
    }
}
