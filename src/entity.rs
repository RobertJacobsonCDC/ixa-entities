/*!

An implementor of `Entity` is a type that names a collection of related properties in analogy to a table in a database. The properties are analogous to the columns in the table, and the `EntityId<E>` type is analogous to the primary key of the table, the row number.

Right now an `Entity` type is just a zero-sized marker type. The static data associated with the type isn't used yet.

*/

use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

/// A type that can be named and used (copied, cloned) but not created outside of this crate.
/// In the `define_entity!` macro we define the alias `pub type MyEntityId = EntityId<MyEntity>`.
pub struct EntityId<E: Entity>(pub(crate) usize, PhantomData<E>);

impl<E: Entity> EntityId<E> {
    /// Only constructible from this crate.
    // pub(crate)
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }
}

/// All entities must implement this trait using the `define_entity!` macro.
pub trait Entity: Any + Default {
    fn name() -> &'static str
    where
        Self: Sized;

    fn type_id() -> TypeId
    where
        Self: Sized,
    {
        TypeId::of::<Self>()
    }

    /// The index of this item in the owner, which is initialized globally per type
    /// upon first access. We explicitly initialize this in a `ctor` in order to know
    /// how many [`Entity`] types exist globally when we construct any `EntityStore`.
    #[cfg(feature = "entity_store")]
    fn index() -> usize
    where
        Self: Sized;

    /// Creates a new boxed instance of the item.
    fn new_boxed() -> Box<Self> {
        Box::new(Default::default())
    }

    /// Standard pattern for downcasting to concrete types.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type BxEntity = Box<dyn Entity>;

/// Defines a zero-sized struct with the right derived traits and implements the `Entity` trait. If you already
/// have a type defined (struct, enum, etc.), you can use the `impl_entity!` macro instead.
#[macro_export]
macro_rules! define_entity {
    ($item_name:ident) => {
        #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
        pub struct $item_name;

        $crate::impl_entity!($item_name);
    };
}

/// Implements the `Entity` trait for the given existing type and defines a type alias
/// of the form `MyEntityId = EntityId<MyEntity>`. For simple zero-sized types, use the
/// `define_entity!` macro instead, which will define the struct and derive all the super traits.
///
/// This macro ensures the correct implementation of the `Entity` trait. The tricky bit is the implementation of
/// `Entity::index`, which requires synchronization in multithreaded runtimes. This is an instance of
/// _correctness via macro_.
#[macro_export]
macro_rules! impl_entity {
    ($item_name:ident) => {
        // Alias of the form `MyEntityId = EntityId<MyEntity>`
        $crate::paste::paste! {
            pub type [<$item_name Id>] = $crate::entity::EntityId<$item_name>;
        }

        impl $crate::entity::Entity for $item_name {
            fn name() -> &'static str
            where
                Self: Sized,
            {
                stringify!($item_name)
            }

            #[cfg(feature = "entity_store")]
            fn index() -> usize {
                // This static must be initialized with a compile-time constant expression.
                // We use `usize::MAX` as a sentinel to mean "uninitialized". This
                // static variable is shared among all instances of this concrete item type.
                static INDEX: std::sync::atomic::AtomicUsize =
                    std::sync::atomic::AtomicUsize::new(usize::MAX);

                // Fast path: already initialized.
                let index = INDEX.load(std::sync::atomic::Ordering::Relaxed);
                if index != usize::MAX {
                    return index;
                }

                // Slow path: initialize it.
                $crate::entity_store::initialize_entity_index(&INDEX)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        // Using `ctor` to initialize entities at program start-up means we know how many entities
        // there are at the time any `EntityStore` is created, which means we never have
        // to mutate `EntityStore` to initialize an `Entity` that hasn't yet been accessed.
        // (The mutation happens inside of a `OnceCell`, which we can already have ready
        // when we construct `EntityStore`.) In other words, we could do away with `ctor`
        // if we were willing to have a mechanism for interior mutability for `EntityStore`.
        #[cfg(feature = "entity_store")]
        $crate::paste::paste! {
            $crate::ctor::declarative::ctor!{
                #[ctor]
                fn [<_register_entity_$item_name:snake>]() {
                    $crate::entity_store::add_to_entity_registry::<$item_name>();
                }
            }
        }
    };
}
