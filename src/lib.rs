// extern crate num_traits;


pub mod map;
pub mod id;

pub use map::*;
pub use id::*;


#[cfg(test)]
fn example() {
    let mut words = IdMap::new();

    let id_hello = words.insert("hello");
    let _id_world = words.insert("world");

    println!("{:?} -> {:?}", id_hello, words.get(id_hello));
    assert_eq!(words.get(id_hello), Some(&"hello"));

    words.mark_unused(id_hello);
    assert_eq!(words.get(id_hello), None);

}
