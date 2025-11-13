# Entities Roadmap

## 1. Derived properties

Derived properties are what make events difficult. The transitive non-derived dependencies need to be tracked. These are known statically (with ctor magic), so they can be stored at the type level.

## 2. Events (`EntityCreatedEvent<E: Entity>`, `PropertyChangeEvent<E: Entity, P: Property<E>>`)

When a property changes, it may need to update an index that depends directly or indirectly on it.

## 3. Indexes

## 4. Queries
