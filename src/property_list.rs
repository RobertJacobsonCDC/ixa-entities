/*!

A [`PropertyList<E>`] is just a tuple of distinct properties of the same [`Entity`] `E`. It
is used in two distinct places: as an initialization list for a new entity, and as a query.

Both use cases have the following two constraints:

1. The properties are properties of the same entity.
2. The properties are distinct.

We enforce the first constraint with the type system by only implementing `PropertyList<E>`
for tuples of types implementing `Property<E>` (of length up to some max). Using properties
for mismatched entities will result in a nice compile-time error at the point of use.

Unfortunately, the second constraint has to be enforced at runtime. We implement `PropertyList::validate()` to do this.

For both use cases, the order in which the properties appear is
unimportant in spite of the Rust language semantics of tuple types.

*/

use std::any::TypeId;
use seq_macro::seq;

use crate::{entity::Entity, property::Property};

pub trait PropertyList<E: Entity>: Copy + 'static {
    /// Validates that the properties are distinct. If not, returns a string describing the problematic properties.
    fn validate() -> Result<(), String>;
}

// The empty tuple is an empty `PropertyList<E>` for every `E: Entity`.
impl<E: Entity> PropertyList<E> for () {
  fn validate() -> Result<(), String> {
    Ok(())
  }
}

// ToDo: Why does the following trigger a "conflicting implementation" error?
// A single `Property` is a `PropertyList` of length 1
// impl<E: Entity, P: Property<E>> PropertyList<E> for P {
//     fn validate() -> Result<(), String> {
//         Ok(())
//     }
// }

// A single `Property` tuple is a `PropertyList` of length 1
impl<E: Entity, P: Property<E>> PropertyList<E> for (P,) {
    fn validate() -> Result<(), String> {
        Ok(())
    }
}


#[macro_export]
macro_rules! impl_property_list {
    ($ct:literal) => {
        seq!(N in 1..=$ct {
            impl<E: Entity, #( P~N: Property<E>,)*> PropertyList<E> for (#(P~N, )*){
                fn validate() -> Result<(), String> {
                    // For `Property` distinctness check
                    let property_type_ids: [TypeId; $ct] = [#(P~N::type_id(),)*];

                    for i in 0..$ct - 1 {
                        for j in (i + 1)..$ct {
                            if property_type_ids[i] == property_type_ids[j] {
                                return Err(format!(
                                    "the same property appears in both position {} and {} in the property list",
                                    i,
                                    j
                                ));
                            }
                        }
                    }

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
