/*!

An `EntityKeyedMap<E, T>` is a `Vec<T>` that uses `EntityId<E>` values as indices instead of
`usize` indices.

## Motivation

An `EntityId<E: Entity>` is an opaque type providing a handle to a particular entity.
Internally, it wraps a `usize` index into the vectors of property values associated to the
entity: the property values of the entity are those stored at the index `EntityId<E>.0`.
However, to enforce _referential integrity_ (you cannot attempt to retrieve a property value
for a nonexistent entity) and _domain separation_ (you cannot use a `PersonId` in place of
a `SettingId`), client code is not allowed to create or destructure `EntityId<E>` values
directly. (Instead, `EntityId<E>` values are created and managed by the `EntityStore`.)

However, there are situations where you want to have a data structure in client
code that uses `EntityId<E>` values in some way, and having access to the underlying
`usize` index can realize efficiency gains. We can bridge the gap somewhat by providing
some building-block data structures that use `EntityId<E>` values internally.

## Open questions

Should this data structure be more like a `Vec<T>` or a `HashMap<EntityId<E>, T>`?
There's a use-case for both, but I think you design the data structure to be one
or the other. Maybe you have two different data structures, one for each use-case?

A [`PropertyValueStore<E, P>`] is like a `HashMap<EntityId<E>, T>` but specialized for `T: Copy`, because it uses a
[`ValueVec<Option<T>>`] under the hood.

### If it were like a `Vec<T>`

- This would indirectly leak the inner `usize` index for `EntityId<E>` values, though probably not in a useful way.
- `Deref` to `Vec<T>`
- `Deref` to `&[T]`
- Useful for storing values for an entire population of entities.
- Gives you lots of nice machinery for iterating, chunking, searching, etc., that is defined on slices.

### `HashMap<EntityId<E>, T>`

- The structure would actually wrap `Vec<Option<T>>`. "Slots" would be empty (`None`) unless the value has been set.
- Use API like [`PropertyValueStore<E, P>`] that grows the underlying `Vec` as needed.
- Useful for storing noncontiguous values, but perhaps not space efficient.

*/

use delegate::delegate;

use crate::entity::{Entity, EntityId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EntityKeyedMap<E: Entity, T> {
    inner: Vec<T>,
    _marker: std::marker::PhantomData<E>,
}

impl<E: Entity, T> EntityKeyedMap<E, T> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            _marker: std::marker::PhantomData,
        }
    }

    // Core operations that require access to the internal index.
    #[inline]
    pub fn get(&self, entity_id: EntityId<E>) -> Option<&T> {
        self.inner.get(entity_id.0)
    }

    #[inline]
    pub fn get_mut(&mut self, entity_id: EntityId<E>) -> Option<&mut T> {
        self.inner.get_mut(entity_id.0)
    }

    // Delegate common Vec-like methods.
    delegate! {
        to self.inner {
            pub fn len(&self) -> usize;
            pub fn is_empty(&self) -> bool;
            pub fn capacity(&self) -> usize;
            pub fn reserve(&mut self, additional: usize);
            pub fn reserve_exact(&mut self, additional: usize);
            pub fn shrink_to_fit(&mut self);
            pub fn shrink_to(&mut self, min_capacity: usize);
            pub fn clear(&mut self);
            pub fn iter(&self) -> std::slice::Iter<'_, T>;
            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T>;
            // pub fn as_slice(&self) -> &[T];
            // pub fn as_mut_slice(&mut self) -> &mut [T];
        }
    }
}

impl<E: Entity, T> Into<Vec<T>> for EntityKeyedMap<E, T> {
    #[inline]
    fn into(self) -> Vec<T> {
        self.inner
    }
}

impl<E: Entity, T> std::ops::Index<EntityId<E>> for EntityKeyedMap<E, T> {
    type Output = T;

    #[inline]
    fn index(&self, id: EntityId<E>) -> &Self::Output {
        &self.inner[id.0]
    }
}

impl<E: Entity, T> std::ops::IndexMut<EntityId<E>> for EntityKeyedMap<E, T> {
    #[inline]
    fn index_mut(&mut self, id: EntityId<E>) -> &mut Self::Output {
        &mut self.inner[id.0]
    }
}
