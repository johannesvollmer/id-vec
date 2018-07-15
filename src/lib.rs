// extern crate num_traits;


#[macro_use]
pub mod map;
pub mod id;

pub use map::*;
pub use id::*;

#[cfg(test)]
mod examples {
    use super::*;

    #[test]
    fn example1() {
        let map = id_map!("hello", "world");
        debug_assert!(map.contains_element(&"hello"));
    }

    #[test]
    fn example2() {
        let mut words = IdMap::new();

        let id_hello = words.insert("hello");
        let _id_world = words.insert("world");

        println!("{:?} -> {:?}", id_hello, words.get(id_hello));
        assert_eq!(words.get(id_hello), Some(&"hello"));

        words.remove(id_hello);
        assert_eq!(words.get(id_hello), None);
    }
}

