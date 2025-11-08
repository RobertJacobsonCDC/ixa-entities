/*!

An integration test that demonstrates the code works from client code external to the crate.

*/
#![allow(unused)]

#[cfg(feature = "entity_store")]
use ixa_entities::entity_store::EntityStore;
use ixa_entities::{
    define_entity, define_property, entity::EntityId, impl_property, property::Property,
    property_store::PropertyStore, property_value_store::PropertyValueStore, serde::Serialize,
};

define_entity!(Person);

define_property!(struct Age(u8), Person);
define_property!(
    enum InfectionStatus {
        Susceptible,
        Infected,
        Recovered,
    } = InfectionStatus::Susceptible,
    Person
);
define_property!(
    struct Vaccinated(bool) = Vaccinated(false),
    Person
);

fn main() {
    #[cfg(feature = "entity_store")]
    {
        let entity_store = EntityStore::new();

        let people = entity_store.get::<Person>();
        println!("Person: {:?}", people);
    }

    let my_age = Age(20);

    println!("My age: {:?}", my_age.get_display());

    let property_store = PropertyStore::new();

    {
        let ages: &PropertyValueStore<_, Age> = property_store.get();
        ages.set(PersonId::new(0), Age(12));
        ages.set(PersonId::new(1), Age(33));
        ages.set(PersonId::new(2), Age(44));

        let infection_statuses: &PropertyValueStore<_, InfectionStatus> = property_store.get();
        infection_statuses.set(PersonId::new(0), InfectionStatus::Susceptible);
        infection_statuses.set(PersonId::new(1), InfectionStatus::Susceptible);
        infection_statuses.set(PersonId::new(2), InfectionStatus::Infected);

        // Subsequent calls to `set` are enough to allow the compiler to infer the type of `vaccine_status`,
        // but leaving off the type annotation for the property should probably be discouraged.
        let vaccine_status: &PropertyValueStore<_, _> = property_store.get();
        vaccine_status.set(PersonId::new(0), Vaccinated(true));
        vaccine_status.set(PersonId::new(1), Vaccinated(false));
        vaccine_status.set(PersonId::new(2), Vaccinated(true));
    }

    // Verify that `get` returns the expected values
    {
        let ages: &PropertyValueStore<_, Age> = property_store.get();
        assert_eq!(ages.get(PersonId::new(0)), Some(Age(12)));
        assert_eq!(ages.get(PersonId::new(1)), Some(Age(33)));
        assert_eq!(ages.get(PersonId::new(2)), Some(Age(44)));

        let infection_statuses: &PropertyValueStore<_, InfectionStatus> = property_store.get();
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

        let vaccine_status: &PropertyValueStore<_, Vaccinated> = property_store.get();
        assert_eq!(vaccine_status.get(PersonId::new(0)), Some(Vaccinated(true)));
        assert_eq!(
            vaccine_status.get(PersonId::new(1)),
            Some(Vaccinated(false))
        );
        assert_eq!(vaccine_status.get(PersonId::new(2)), Some(Vaccinated(true)));
    }
}
