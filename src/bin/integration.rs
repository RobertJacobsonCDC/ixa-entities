#![allow(unused)]

use ixa_entities::{
    Context, define_entity,
    entity::EntityId,
    entity_store::EntityStore,
    property::{Property, PropertyInitializationKind},
    serde::Serialize,
};

define_entity!(People);

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Serialize)]
struct Age(u8);


fn main() {
    let entity_store = EntityStore::new();

    let people = entity_store.get::<People>();
    println!("Person: {:?}", people);

    let my_age = Age(20);

    println!("My age: {:?}", my_age.get_display());
}
