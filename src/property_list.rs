/*!

A `PropertyList` is just a tuple of distinct properties. It is used in two distinct places:

1. as an initialization list for a new entity, and
2. as a query.

In both instances, the order in which the properties appear is unimportant in spite of the Rust language semantics of
tuple types.

We implement the `PropertyList` trait for tuples of `Property` types of lengths up to some max. The fundamental
capability of a property list is that it knows which `Property` types it contains and has getters and setters for
each property.

```rust
(Age(32), InfectionStatus::Susceptible, WorkplaceId(123))
```

*/

use std::any::TypeId;
use crate::property::Property;

pub trait PropertyList: Copy + 'static {
  fn get<P: Copy + 'static>(&self) -> Option<P>;
}

impl PropertyList for (&'static str, u8, u32) {
  fn get<P: Copy + 'static>(&self) -> Option<P> {
    let type_id = TypeId::of::<P>();
    if type_id == TypeId::of::<&'static str>() {
      Some(self.0)
    }
  }
}


pub trait Has<P: Property> {
  fn get(&self) -> Option<P>;
}

impl<P0: Property, P1: Property> Has<P0> for (P0, P1) {
  fn get(&self) -> Option<P0> {
    Some(self.0)
  }
}


impl<P0: Property, P1: Property> Has<P1> for (P0, P1) {
  fn get(&self) -> Option<P0> {
    Some(self.1)
  }
}
