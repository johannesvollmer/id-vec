

# IdMap

You can think of the IdMap as a vector that 
reuses slots. It inserts new elements into 
places where old elements were removed, 
instead of shifting all the remaining elements by one. 
This allows using indices to refer to elements, which
remain valid even after removing other elements from the vector. 

The goal of this specific library is being very minimal, 
both in resource usage and API complexity. 
As a consequence, it does not have a runtime system to detect the incorrect use of deleted ids. 
The user must take care to not use ids that have been deleted. 



## Usage

```rust
let mut words = IdMap::new();

let id_hello: Id<&str> = words.insert("hello");
let id_world: Id<&str> = words.insert("world");

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


__This library provides a container built specifically for that use case of
connected graph nodes.__


## Architecture

This project has two core structs: the map itself, and the id. 
The id is just a newtype wrapping and index, but it has a type parameter
to improve type safety for indices. The map internally is a vector, 
but it reuses deleted slots. It does so by storing the indices 
of deleted elements in a hash set, which is fast for insertion of new elements 
but may be not as fast as BitVec for indexing. 
