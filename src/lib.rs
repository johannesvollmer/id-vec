// extern crate num_traits;


#[macro_use]
pub mod vec;
pub mod id;
pub mod element_marker;

pub use vec::IdVec;
pub use id::Id;

#[cfg(test)]
mod examples {
    use super::*;

    #[test]
    fn example1() {
        let map = id_vec!("hello", "world");
        debug_assert!(map.contains_element(&"hello"));
        println!("{:?}", map);
    }

    #[test]
    fn example2() {
        let mut words = IdVec::new();

        let id_hello = words.insert("hello");
        let _id_world = words.insert("world");

        println!("{:?} -> {:?}", id_hello, words.get(id_hello));
        assert_eq!(words.get(id_hello), Some(&"hello"));

        words.remove(id_hello);
        assert_eq!(words.get(id_hello), None);
    }

}

