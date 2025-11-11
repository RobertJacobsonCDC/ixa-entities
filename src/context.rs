use crate::{
    entity::{Entity, EntityId},
    entity_store::EntityStore,
    property_list::PropertyList,
    property_store::PropertyStore,
};

/// A minimalist stand-in for a `Context` object.
pub struct Context {
    pub entity_store: EntityStore,
    pub property_store: PropertyStore,
}

impl Context {
    pub fn new() -> Self {
        Self {
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

        // Assign the properties in the list to the new entity.
        property_list.set_values_for_entity(new_entity_id.clone(), &self.property_store);

        new_entity_id
    }
}


#[cfg(test)]
mod tests {
    use crate::{define_entity, define_property, impl_property};
    use super::*;

    define_entity!(Person);

    define_property!(struct Age(u8), Person, is_required = true);

    define_property!(
        enum InfectionStatus {
            Susceptible,
            Infected,
            Recovered,
        },
        Person,
        default_const = InfectionStatus::Susceptible
    );

    define_property!(
        struct Vaccinated(bool),
        Person,
        default_const = Vaccinated(false)
    );


    #[test]
    fn add_an_entity(){
        let mut context = Context::new();
        let person = context.add_entity((
            Age(12),
            InfectionStatus::Susceptible,
            Vaccinated(true),
        ));
        println!("{:?}", person);

        let person = context.add_entity((
            Age(34),
            Vaccinated(true),
        ));
        println!("{:?}", person);

        // Age is the only required property
        let person = context.add_entity((
            Age(120),
        ));
        println!("{:?}", person);
    }

    #[test]
    #[should_panic(expected = "initialization list is missing required properties")]
    fn add_an_entity_without_required_properties(){
        let mut context = Context::new();
        let person1 = context.add_entity((
            InfectionStatus::Susceptible,
            Vaccinated(true),
        ));
        println!("{:?}", person1);
    }
}
