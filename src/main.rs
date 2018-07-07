// extern crate num_traits;


pub mod map;
pub mod id;

pub use map::*;
pub use id::*;

fn main() {
    let mut words = IdMap::new();

    let id_hello = words.insert("hello");
    let id_world = words.insert("world");

    println!("{:?} -> {:?}", id_hello, words.get(id_hello));

    words.mark_unused(id_hello);
}
