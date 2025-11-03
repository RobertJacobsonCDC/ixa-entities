//! Macros for implementing properties.




/// Defines a property with the following parameters:
/// * `$property`: A name for the identifier type of the property
/// * `$entity`: The entity type this property is associated with
/// * `default_const`: (Optional) A constant initial value. If it is not defined, calling `get_property`
///   on the property without explicitly setting a value first will panic.
#[macro_export]
macro_rules! impl_property {
    // T with constant default value
    ($property:ident, $entity:ident, $default_const:expr) => {
        $crate::impl_property_with_options!($property, $entity, default_const = $default_const,);
    };

    // T without constant default value
    ($property:ident, $entity:ident) => {
        $crate::impl_property_with_options!($property, $entity);
    };
}
pub use impl_property;

use crate::property::{Property, PropertyInitializationKind};

/// Defines a property type with optional named configuration parameters. The named parameters
/// need to be supplied in the order listed below even if some of them are not used.
///
/// # Parameters
/// - `$property`: The identifier for the type implementing [`Property`].
/// - `$entity`: The entity type this property is associated with.
/// - Optional parameters (each may be omitted; defaults will be used):
///   - `canonical_value = <type>` — If the type stored in the index differs from the property's value type.
///   - `initialization_kind = <expr>` — Initialization strategy; defaults to `PropertyInitializationKind::Explicit`.
///   - `is_required = <bool>` — Whether new entities must explicitly set this property; defaults to `false`.
///   - `compute_derived_fn = <expr>` — Function used to compute derived properties; defaults to `None`.
///   - `default_const = <expr>` — Constant default value if the property has one; defaults to `None`.
///   - `display_impl = <expr>` — Function converting the canonical value to a string; defaults to `|v| format!("{v:?}")`.
#[macro_export]
macro_rules! impl_property_with_options {
    (
        $property:ident,
        $entity:ident
        $(, canonical_value = $canonical_value:ty)?
        $(, initialization_kind = $initialization_kind:expr)?
        $(, is_required = $is_required:expr)?
        $(, compute_derived_fn = $compute_derived_fn:expr)?
        $(, default_const = $default_const:expr)?
        $(, display_impl = $display_impl:expr)?
    ) => {
        $crate::__impl_property_common!(
            $property,
            $entity,
            $crate::impl_property_with_options!(@unwrap_or_ty $($canonical_value)?, $property),
            $crate::impl_property_with_options!(@unwrap_or $($initialization_kind)?, $crate::property::PropertyInitializationKind::Explicit),
            $crate::impl_property_with_options!(@unwrap_or $($is_required)?, false),
            $crate::impl_property_with_options!(@unwrap_or $($compute_derived_fn)?, |_, _| panic!("property {} is not derived", stringify!($property)) ),
            $crate::impl_property_with_options!(@unwrap_or $($default_const)?, panic!("property {} has no default value", stringify!($property))),
            $crate::impl_property_with_options!(@unwrap_or $($display_impl)?, |v| format!("{v:?}"))
        );
    };

    // Helpers for defaults, a pair per macro parameter type (`expr`, `ty`).
    (@unwrap_or $value:expr, $_default:expr) => { $value };
    (@unwrap_or, $default:expr) => { $default };

    (@unwrap_or_ty $ty:ty, $_default:ty) => { $ty };
    (@unwrap_or_ty, $default:ty) => { $default };
}

/// Internal macro used to define common boilerplate for property types that
/// implement the [`Property`] trait. The `impl_property_with_options`
/// macro provides a more ergonomic interface for this macro.
///
/// # Parameters
///
/// * `$property` — The name of the concrete type implementing [`Property`].
/// * `$entity` — The entity type this property is associated with.
/// * `$canonical_value` — The canonical type stored in the index if it differs
///   from the property’s own value type.
/// * `$initialization_kind` — The [`PropertyInitializationKind`] describing how
///   this property is initialized (e.g. `Constant`, `Dynamic`, `Derived`, etc.).
/// * `$is_required` — A boolean indicating whether new entities must have this
///   property explicitly set at creation time.
/// * `$compute_derived_fn` — A function or closure used to compute the property’s
///   value if it is derived from other properties.
/// * `$default_const` — The constant default value if the property has one.
/// * `$display_impl` — A function that takes a canonical value and returns a
///   string representation of the property.
#[macro_export]
macro_rules! __impl_property_common {
    (
        $property:ident,           // The name of the type we are implementing `Property` for
        $entity:ident,             // The entity type this property is associated with
        $canonical_value:ty,       // If the type stored in the index is different from Self, the name of that type
        $initialization_kind:expr, // The kind of initialization this property has
        $is_required:expr,         // Do we require that new entities have this property explicitly set?
        $compute_derived_fn:expr,  // If the property is derived, the function that computes the value
        $default_const:expr,       // If the property has a constant default initial value, the default value
        $display_impl:expr         // A function that takes a canonical value and returns a string representation of this property
    ) => {
        impl $crate::property::Property for $property {
            type Entity = $entity;
            type CanonicalValue = $canonical_value;

            fn initialization_kind() -> $crate::property::PropertyInitializationKind {
                $initialization_kind
            }

            fn is_required() -> bool {
                $is_required
            }

            fn compute_derived(
                _context: &$crate::Context,
                _entity_id: $crate::entity::EntityId<Self::Entity>,
            ) -> Self::CanonicalValue {
                $compute_derived_fn(_context, _entity_id)
            }

            fn default_const() -> Self {
                $default_const
            }

            fn make_canonical(&self) -> Self::CanonicalValue {
                *self
            }
            fn make_uncanonical(value: Self::CanonicalValue) -> Self {
                value
            }
            fn name() -> &'static str {
                stringify!($property)
            }
            fn get_display(&self) -> String {
                $display_impl(self)
            }
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
                $crate::property_store::initialize_property_index(&INDEX)
            }
        }

        // Using `ctor` to initialize properties at program start-up means we know how many properties
        // there are at the time any `PropertyStore` is created, which means we never have
        // to mutate `PropertyStore` to initialize a `Property` that hasn't yet been accessed.
        // (The mutation happens inside of a `OnceCell`, which we can already have ready
        // when we construct `PropertyStore`.) In other words, we could do away with `ctor`
        // if we were willing to have a mechanism for interior mutability for `PropertyStore`.
        $crate::paste::paste! {
            $crate::ctor::declarative::ctor!{
                #[ctor]
                fn [<_register_property_$property:snake>]() {
                    $crate::property_store::add_to_property_registry::<$property>();
                }
            }
        }
    };
}

/*
/// Defines a derived property with the following parameters:
/// * `$property`: A name for the identifier type of the property
/// * `$value`: The type of the property's value
/// * `[$($dependency),+]`: A list of person properties the derived property depends on
/// * `[$($dependency),*]`: A list of global properties the derived property depends on (optional)
/// * `$calculate`: A closure that takes the values of each dependency and returns the derived value
/// * `$display`: A closure that takes the value of the derived property and returns a string representation
/// * `$hash_fn`: A function that can compute the hash of values of this property
#[macro_export]
macro_rules! __define_derived_property_common {
    (
        $derived_property:ident,
        $entity:ty,
        $value:ty,
        $canonical_value:ty,
        $compute_canonical_impl:expr,
        $compute_uncanonical_impl:expr,

        $at_dependency_registration:expr,

        [$($dependency:ident),*],
        [$($global_dependency:ident),*],

        |$($param:ident),+| $derive_fn:expr,

        $display_impl:expr,

        $hash_fn:expr,

        $type_id_impl:expr
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $derived_property;

        impl $crate::people::Property for $derived_property {
            type Value = $value;
            type CanonicalValue = $canonical_value;

            fn initialization_kind() -> $crate::people::PropertyInitializationKind {
                $crate::people::PropertyInitializationKind::Derived
            }

            fn compute(context: &$crate::context::Context, person_id: $crate::people::PersonId) -> Self::Value {
                #[allow(unused_imports)]
                use $crate::global_properties::ContextGlobalPropertiesExt;
                #[allow(unused_parens)]
                let ($($param,)*) = (
                    $(context.get_property(person_id, $dependency)),*,
                    $(
                        context.get_global_property_value($global_dependency)
                            .expect(&format!("Global property {} not initialized", stringify!($global_dependency)))
                    ),*
                );
                #[allow(non_snake_case)]
                (|$($param),+| $derive_fn)($($param),+)
            }

            fn compute_immutable(context: &$crate::context::Context, person_id: $crate::people::PersonId) -> Self::Value {
                #[allow(unused_imports)]
                use $crate::global_properties::ContextGlobalPropertiesExt;
                #[allow(unused_parens)]
                let ($($param,)*) = (
                    $(context.get_property_immutable(person_id, $dependency)),*,
                    $(
                        // Right now `get_global_property_value` is always an immutable operation.
                        context.get_global_property_value($global_dependency)
                            .expect(&format!("Global property {} not initialized", stringify!($global_dependency)))
                    ),*
                );
                #[allow(non_snake_case)]
                (|$($param),+| $derive_fn)($($param),+)
            }

            fn make_canonical(value: Self::Value) -> Self::CanonicalValue {
                ($compute_canonical_impl)(value)
            }
            fn make_uncanonical(value: Self::CanonicalValue) -> Self::Value {
                ($compute_uncanonical_impl)(value)
            }
            fn is_derived() -> bool { true }
            fn dependencies() -> Vec<Box<dyn $crate::people::PropertyHolder>> {
                vec![$(
                    Box::new($dependency) as Box<dyn $crate::people::PropertyHolder>
                ),*]
            }
            fn register_dependencies(context: &$crate::context::Context) {
                $at_dependency_registration
                $(context.register_property::<$dependency>();)+
            }
            fn get_instance() -> Self {
                $derived_property
            }
            fn name() -> &'static str {
                stringify!($derived_property)
            }
            fn get_display(value: &Self::CanonicalValue) -> String {
                $display_impl(value)
            }
            fn hash_property_value(value: &Self::CanonicalValue) -> u128 {
                ($hash_fn)(value)
            }
            fn type_id() -> std::any::TypeId {
                $type_id_impl
            }
        }
    };
}
*/

/*
/// Defines a derived property with the following parameters:
/// * `$property`: A name for the identifier type of the property
/// * `$value`: The type of the property's value
/// * `[$($dependency),+]`: A list of person properties the derived property depends on
/// * `[$($dependency),*]`: A list of global properties the derived property depends on (optional)
/// * $calculate: A closure that takes the values of each dependency and returns the derived value
#[macro_export]
macro_rules! define_derived_property {
    (
        $derived_property:ident,
        $value:ty,
        [$($dependency:ident),*],
        [$($global_dependency:ident),*],
        |$($param:ident),+| $derive_fn:expr
    ) => {
        $crate::__define_derived_property_common!(
            $derived_property,
            $value,
            $value,
            |v| v,
            |v| v,
            {/* empty*/},
            [$($dependency),*],
            [$($global_dependency),*],
            |$($param),+| $derive_fn,
            |&value| format!("{:?}", value),
            $crate::hashing::hash_serialized_128,
            std::any::TypeId::of::<Self>()
        );
    };

    // Empty global dependencies
    (
        $derived_property:ident,
        $value:ty,
        [$($dependency:ident),*],
        |$($param:ident),+| $derive_fn:expr
    ) => {
        $crate::__define_derived_property_common!(
            $derived_property,
            $value,
            $value,
            |v| v,
            |v| v,
            {/* empty*/},
            [$($dependency),*],
            [],
            |$($param),+| $derive_fn,
            |&value| format!("{:?}", value),
            $crate::hashing::hash_serialized_128,
            std::any::TypeId::of::<Self>()
        );
    };
}
pub use define_derived_property;
*/

/*
#[macro_export]
macro_rules! define_multi_property {
    (
        $property:ident,
        ( $($dependency:ident),+ )
    ) => {
        // $crate::sorted_property_impl!(( $($dependency),+ ));
        $crate::paste::paste! {
            $crate::__define_derived_property_common!(
                // Name
                $property,

                // `Property::Value` type
                ( $(<$dependency as $crate::people::Property>::Value),+ ),

                // `Property::CanonicalValue` type
                $crate::sorted_value_type!(( $($dependency),+ )),

                // Function to transform a `Property::Value` to a `Property::CanonicalValue`
                $property::reorder_by_tag,

                // Function to transform a `Property::CanonicalValue` to a `Property::Value`
                $property::unreorder_by_tag,

                // Code that runs at dependency registration time
                {
                    let type_ids = &mut [$($dependency::type_id()),+ ];
                    type_ids.sort();
                    $crate::people::register_type_ids_to_muli_property_id(type_ids, Self::type_id());
                },

                // Property dependency list
                [$($dependency),+],

                // Global property dependency list
                [],

                // A function that takes the values of each dependency and returns the derived value
                |$( [<_ $dependency:lower>] ),+| {
                    ( $( [<_ $dependency:lower>] ),+ )
                },

                // A function that takes a canonical value and returns a string representation of it.
                |values_tuple: &Self::CanonicalValue| {
                    // ice tThe string representation uses the original (unsorted) ordering.
                    let values_tuple: Self::Value = Self::unreorder_by_tag(*values_tuple);
                    let mut displayed = String::from("(");
                    let ( $( [<_ $dependency:lower>] ),+ ) = values_tuple;
                    $(
                        displayed.push_str(<$dependency as $crate::Property>::get_display(
                            & <$dependency as $crate::Property>::make_canonical([<_ $dependency:lower>])
                        ).as_str());
                        displayed.push_str(", ");
                    )+
                    displayed.truncate(displayed.len() - 2);
                    displayed.push_str(")");
                    displayed
                },

                // A function that computes the hash of a value of this property
                $crate::hashing::hash_serialized_128,

                // The Type ID of the property.
                // The type ID of a multi-property is the type ID of the SORTED tuple of its
                // components. This is so that tuples with the same component types in a different
                // order will have the same type ID.
                std::any::TypeId::of::<$crate::sorted_tag!(( $($dependency),+ ))>()
            );
            $crate::impl_make_canonical!($property, ( $($dependency),+ ));
        }
    };
}
pub use define_multi_property;
*/
