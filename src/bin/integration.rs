/*!

An integration test that demonstrates the code works from client code external to the crate.

*/
#![allow(unused)]

use ixa_entities::entity_store::EntityStore;
use ixa_entities::{
    define_entity, define_property, entity::EntityId, impl_property, property::Property,
    property_store::PropertyStore, property_value_store::PropertyValueStore, serde::Serialize,
};

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


fn main() {

    let my_age = Age(20);

    println!("My age: {:?}", my_age.get_display());

    let mut context = ixa_entities::Context::new();

    let person1 = context.add_entity((
        Age(12),
        InfectionStatus::Susceptible,
        Vaccinated(true),
        ));
    let person2 = context.add_entity((
        Age(33),
        InfectionStatus::Susceptible,
        Vaccinated(false),
        ));
    let person3 = context.add_entity((
        Age(44),
        InfectionStatus::Infected,
        Vaccinated(true),
    ));


    // Verify that `get` returns the expected values
    {
        let ages: &PropertyValueStore<_, Age> = context.property_store.get();
        assert_eq!(ages.get(PersonId::new(0)), Some(Age(12)));
        assert_eq!(ages.get(PersonId::new(1)), Some(Age(33)));
        assert_eq!(ages.get(PersonId::new(2)), Some(Age(44)));

        let infection_statuses: &PropertyValueStore<_, InfectionStatus> = context.property_store.get();
        assert_eq!(
            infection_statuses.get(PersonId::new(0)),
            Some(InfectionStatus::Susceptible)
        );
        assert_eq!(
            infection_statuses.get(PersonId::new(1)),
            Some(InfectionStatus::Susceptible)
        );
        assert_eq!(
            infection_statuses.get(PersonId::new(2)),
            Some(InfectionStatus::Infected)
        );

        let vaccine_status: &PropertyValueStore<_, Vaccinated> = context.property_store.get();
        assert_eq!(vaccine_status.get(PersonId::new(0)), Some(Vaccinated(true)));
        assert_eq!(
            vaccine_status.get(PersonId::new(1)),
            Some(Vaccinated(false))
        );
        assert_eq!(vaccine_status.get(PersonId::new(2)), Some(Vaccinated(true)));
    }
}
