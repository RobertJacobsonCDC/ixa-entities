use crate::entity::{
    Entity, 
    EntityId,
    entity_store::EntityStore,
    property_list::PropertyList,
    property_store::PropertyStore,
    property::{Property, PropertyInitializationKind}
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

    pub fn get_property<E: Entity, P: Property<E>>(&self, entity_id: EntityId<E>) -> P {
        // ToDo(RobertJacobsonCDC): An alternative to the following is to always assume
        //       that `None` means "not set" for "explicit" properties, that is, assume
        //       that `get` is infallible for properties with a default constant. We
        //       take a more conservative approach here and check for internal errors.
        match P::initialization_kind() {
            PropertyInitializationKind::Explicit => {
                let property_store = self.property_store.get::<E, P>();
                // A user error can cause this unwrap to fail.
                property_store.get(entity_id).expect("attempted to get a property value with \"explicit\" initialization that was not set")
            }

            PropertyInitializationKind::Derived => {
                P::compute_derived(self, entity_id)
            }

            PropertyInitializationKind::Constant => {
                let property_store = self.property_store.get::<E, P>();
                // If this unwrap fails, it is an internal ixa error, not a user error.
                property_store.get(entity_id).expect("getting a property value with \"constant\" initialization should never fail")
            }
        }
    }

    pub fn set_property<E: Entity, P: Property<E>>(&self, entity_id: EntityId<E>, property_value: P) {
        let property_value_store = self.property_store.get::<E, P>();
        property_value_store.set(entity_id, property_value);
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
    fn add_an_entity() {
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

    #[test]
    fn get_and_set_property_explicit() {
        let mut context = Context::new();

        // Create a person with required Age property
        let person = context.add_entity((Age(25),));

        // Retrieve it
        let age: Age = context.get_property(person);
        assert_eq!(age, Age(25));

        // Change it
        context.set_property(person, Age(26));
        let age: Age = context.get_property(person);
        assert_eq!(age, Age(26));
    }

    #[test]
    fn get_property_with_constant_default() {
        let mut context = Context::new();

        // `Vaccinated` has a default value (false)
        let person = context.add_entity((Age(40),));

        // Even though we didn't set Vaccinated, it should exist with its default
        let vaccinated: Vaccinated = context.get_property(person);
        assert_eq!(vaccinated, Vaccinated(false));


        // Now override
        context.set_property(person, Vaccinated(true));
        let vaccinated: Vaccinated = context.get_property(person);
        assert_eq!(vaccinated, Vaccinated(true));
    }


    #[test]
    fn get_property_with_enum_default() {
        let mut context = Context::new();

        // InfectionStatus has a default of Susceptible
        let person = context.add_entity((Age(22),));
        let status: InfectionStatus = context.get_property(person);
        assert_eq!(status, InfectionStatus::Susceptible);
    }

}
