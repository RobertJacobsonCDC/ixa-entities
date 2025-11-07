# Notes about `Entity` Implementation

## `ValueVec`

### Options for `!Drop`

#### `std::mem::ManuallyDrop<T>`

> A wrapper to inhibit the compiler from automatically calling T’s destructor. This wrapper is 0-cost.


#### Compiler error on `std::mem::needs_drop::<V>()`

We can prevent construction of `ValueVec`s for values with destructors by using the `needs_drop` function:

```rust
impl<V> ValueVec<V> {
    pub fn new() -> Self {
        // Panics if `V` has a destructor.
        assert!(!needs_drop::<V>(), "Type must not have a destructor");
        
        ValueVec {
            values: Vec::new(),
        }
    }
}
```

### Homework assignments

- Can we further constrain `V` to be `!Drop` (and/or `!Clone`) ?
- Where is the compiler inserting `Drop`? Can we only implement mutating primitives such that
  `Drop` is not called? (If we never "delete" something. Is `Drop` called when overwritten?)
- If we provide a trivial implementation of `Drop` for all `Property` types, does that solve
  the problem? (Could there be a hidden `Drop` impl called somehow by virtue of what the type is?)



## Types

### `struct EntityId<E: Entity>(usize, PhantomData<E>)`

- The entity ID should know the `Entity` type so that a `PersonId` can't be used as a `SettingId`.
- The original `PersonId` type is opaque–it cannot be created or destructured outside of the `ixa` crate.  To achieve the same thing, we do this: `EntityId<E: Entity>(usize, PhantomData<E>)`


## Properties

Right now we store: `Vec<Option<Property>>`

But maybe we should only store: `Vec<Property>`

If the value is allowed to be not set, it should be an `Option<Property>`. The difference is, do we enforce this at 
the API level, or do we make client code deal with it?

