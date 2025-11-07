/*!

A [`PropertyList`] is just a tuple of distinct properties of the same [`Entity`]. It is used in two distinct places:

1. as an initialization list for a new entity, and
2. as a query.

Both require that the properties are distinct and are properties of the
same entity. In both instances, the order in which the properties appear
is unimportant in spite of the Rust language semantics of tuple types.

We implement the [`PropertyList`] trait for tuples of [`Property`] types of lengths up to some max. The fundamental
capability of a property list is that it knows which [`Property`] types it contains and has getters and setters for
each property.


```rust
(Age(32), InfectionStatus::Susceptible, WorkplaceId(123))
```

*/

use std::any::TypeId;
use crate::entity::Entity;
use crate::property::Property;

pub trait PropertyList: Copy + 'static {
  /// Validates that
  /// 1. the properties are distinct and
  /// 2. the properties belong to the same Entity.
  /// If either does not hold, the error describes the problematic properties.
  fn validate() -> Result<(), String>;
}

// The empty `PropertyList`
impl PropertyList for () {
  fn validate() -> Result<(), String> {
    Ok(())
  }
}

// A single `Property` is a `PropertyList` of length 1
impl<P: Property> PropertyList for P {
  fn validate() -> Result<(), String> {
    Ok(())
  }
}

// A single `Property` tuple is a `PropertyList` of length 1
impl<P: Property> PropertyList for (P, ) {
  fn validate() -> Result<(), String> {
    Ok(())
  }
}

use seq_macro::seq;

#[macro_export]
macro_rules! impl_property_list {
    ($ct:literal) => {
        seq!(N in 1..=$ct {
            impl<#( P~N: Property,)*> PropertyList for (#(P~N, )*){
                fn validate() -> Result<(), String> {
                    // For `Property` distinctness check
                    let property_type_ids: [TypeId; $ct] = [#(P~N::type_id(),)*];

                    // For `Entity` consistency check
                    let expected_entity = P1::Entity::type_id();

                    // Now for each property `P~N`...
                    #(
                        // Check that this property is distinct from all subsequent properties.
                        for j in (N + 1)..$ct {
                            if property_type_ids[N] == property_type_ids[j] {
                                return Err(format!(
                                    "property {} appears in both position {} and {} in the property list",
                                    P~N::name(),
                                    N,
                                    j
                                ));
                            }
                        }
                        // Check that this property belongs to the same entity as the first property.
                        if expected_entity != P~N::Entity::type_id() {
                            return Err(format!(
                                "properties {} and {} are not properties of the same entity.",
                                P1::name(),
                                P~N::name()
                            ));
                        }
                    )*

                    Ok(())
                }
            }
        });
    };
}

// Generate impls for tuple lengths 2 through 10.
seq!(Z in 2..=5 {
    impl_property_list!(Z);
});

// impl_property_list!(2);
