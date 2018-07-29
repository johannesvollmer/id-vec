// extern crate num_traits;


#[macro_use]
pub mod vec;
pub mod id;

pub use vec::IdVec;
pub use id::Id;

#[cfg(test)]
mod examples {
    use super::*;

    #[test]
    fn nodes() {

        #[derive(Debug)]
        struct Node {
            parent: Option<Id<Node>>,
            name: String,
        }



        let mut nodes = IdVec::new();

        let root: Id<Node> = nodes.insert(Node {
            parent: None,
            name: String::from("Root"),
        });

        let orphan = nodes.insert(Node {
            parent: None,
            name: String::from("Orphan")
        });

        let child = nodes.insert(Node {
            parent: Some(root),
            name: String::from("Child")
        });

        println!("{:?}", nodes);




        // run a basic garbage collector,
        // keeping all objects which have a parent, and the root itself
        {
            nodes.retain(|node_id, node|{
                node_id == root || node.parent.is_some()
            });

            // attention! child_a was removed and thus the id becomes invalid!
            assert!(!nodes.contains_id(orphan));
            assert!(nodes.contains_id(root));
            assert!(nodes.contains_id(child));
        }



        // create a cyclic graph
        {
            // to dereference the node, we index into the id-vec
            let root_mut = &mut nodes[root];

            // creating a cycle is possible
            root_mut.parent = Some(child);
        }
    }


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

