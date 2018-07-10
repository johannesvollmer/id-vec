

# IdMap

The IdMap behaves similar to a `Map<Id, T>`, but automatically creates Ids. 

Internally, a `Vec<T>` is used, and `Id`s are just indices. 
The `Vec<T>` will reuse deleted slots. 

The goal of this specific library is being very minimal.
As a consequence, the user must take care to not use ids that have been deleted.


## Usage

```
let mut words = IdMap::new();

let id_hello = words.insert("hello");
let id_world = words.insert("world");

println!("{:?} -> {:?}", id_hello, words.get(id_hello));

```

## Motivation 

In Rust, Graphs can be quite a difficult architecture, 
because a straightforward `&'a T` would prevent any mutation of the graph. 
Using a `Ref<Cell<T>>` is considered un-idiomatic 
because it circumvents Rusts safety mechanisms.

One way to solve this problem is to use 
indices, and store all nodes of a graph in a vector.
Each node would store the indices of their connected nodes, 
instead of a direct reference to them.
This however requires all operations on Nodes to access the vector, 
which could feel quite unergonomic. Nevertheless, this
approach appears the most idiomatic and safe to me. 

This library aims to simplify the index-based approach by
introducing ids, which are essentially type-safe indices.
The safety is achieved by allowing only `Id<T>`s 
and not any unsigned number as index for a vector. 



## Architecture

This project has two core structs: the map itself, and the id. 
The id is just a newtype wrapping and index, but it has a type parameter
to improve type safety for indices. The map internally is a vector, 
but it reuses deleted slots. 
