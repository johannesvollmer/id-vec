
use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::id::*;


/// Create a new id_map by entering a series of values
macro_rules! id_map {
    ( $($element:expr),* ) => {
        IdMap::from_vec(vec![ $($element),* ])
    };
}


/// Inserting elements into this map yields a persistent, type-safe Index to that new element.
#[derive(Clone, Debug)] // manual impl: Eq, PartialEq
pub struct IdMap<T> {
    /// Packed dense vector, containing alive and dead elements.
    /// Because removing the last element directly can be done efficiently,
    /// it is guaranteed that the last element is never unused.
    elements: Vec<T>,

    /// Contains all unused ids which are allowed to be overwritten,
    /// will never contain the last ID, because the last id can be removed directly
    unused_indices: HashSet<Index>, // TODO if iteration is too slow, use both Vec<NextUnusedIndex> and BitVec
}


// TODO use rusts safety mechanisms to allow (but not enforce) stronger id lifetime safety? OwnedId<T>?
// TODO impl Eq, Clone, Debug, ...

impl<T> IdMap<T> {
    /// Does not allocate heap memory
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(Vec::with_capacity(capacity))
    }

    /// Create a map containing these elements.
    /// Directly uses the specified vector,
    /// so no allocation is made calling this function.
    pub fn from_vec(elements: Vec<T>) -> Self {
        IdMap {
            unused_indices: HashSet::new(), // no elements deleted
            elements,
        }
    }



    /// reserve space for more elements, avoiding frequent reallocation
    pub fn reserve(&mut self, additional: usize){
        self.elements.reserve(additional)
    }

    /// Returns if this id is not deleted (does not check if index is inside vector range)
    fn index_is_currently_used(&self, index: Index) -> bool {
        index + 1 == self.elements.len() || // fast return for last element is always used
            !self.unused_indices.contains(&index)
    }

    fn index_is_in_range(&self, index: Index) -> bool {
        index < self.elements.len()
    }

    #[inline(always)]
    fn debug_assert_id_validity(&self, element: Id<T>, validity: bool){
        debug_assert!(
            self.contains(element) == validity,
            "Expected {:?} validity to be {}, but was not", element, validity
        );
    }
    
    #[inline(always)]
    fn debug_assert_last_element_is_used(&self){
        if !self.is_empty() {
            debug_assert!(
                self.contains(Id::from_index(self.elements.len() - 1)),
                "IdMap has invalid state: Last element is unused."
            );
        }
    }



    pub fn len(&self) -> usize {
        debug_assert!(self.elements.len() >= self.unused_indices.len(), "More ids are unused than exist");
        self.elements.len() - self.unused_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Excludes deleted elements, and indices out of range
    pub fn contains(&self, element: Id<T>) -> bool {
        self.index_is_in_range(element.index_value())
            && self.index_is_currently_used(element.index_value())
    }

    /// Returns if the internal vector contains any deleted elements
    pub fn is_packed(&self) -> bool {
        self.unused_indices.is_empty()
    }



    /// Enable the specified id to be overwritten when a new element is inserted.
    /// This does not directly deallocate the element.
    /// Make sure that no ids pointing to that element exist after this call.
    /// Ignores invalid and deleted ids.
    pub fn remove(&mut self, element: Id<T>) {
        self.debug_assert_last_element_is_used();

        if self.index_is_in_range(element.index_value()) {

            // if exactly the last element, remove without inserting into unused_ids
            if element.index_value() + 1 == self.elements.len() {
                self.elements.pop();

                // remove all unused elements at the end of the vector
                // which may have been guarded by the (now removed) last element
                self.pop_back_unused();

            } else { // remove not-the-last element
                self.unused_indices.insert(element.index_value()); // may overwrite existing index
            }
        }

        self.debug_assert_id_validity(element, false);
        self.debug_assert_last_element_is_used();
    }

    /// Removes an id and the associated element.
    /// See `pop_element` for more information.
    pub fn pop(&mut self) -> Option<(Id<T>, T)> {
        self.debug_assert_last_element_is_used();

        let popped = self.elements.pop().map(|element|{
            (Id::from_index(self.elements.len()), element)
        });

        self.pop_back_unused();

        self.debug_assert_last_element_is_used();
        popped
    }

    /// Removes an element from this map, returns the element:
    /// Removes the one element which is the least work to remove, the one with the highest id.
    /// May deallocate unused elements. Returns None if this map is empty.
    pub fn pop_element(&mut self) -> Option<T> {
        self.pop().map(|(_, element)| element)
    }

    /// Recover from possibly invalid state
    /// by removing any non-used elements from the back of the vector
    fn pop_back_unused(&mut self){
        if self.elements.len() == self.unused_indices.len() {
            self.clear();

        } else {
            while !self.elements.is_empty() // prevent overflow at len() - 1
                && self.unused_indices.remove(&(self.elements.len() - 1)) {

                self.elements.pop(); // pop the index that has just been removed from the unused-set
            }
        }

        self.debug_assert_last_element_is_used();
    }

    /// Associate the specified element with a currently unused id.
    /// This may overwrite (thus drop) unused elements.
    pub fn insert(&mut self, element: T) -> Id<T> {
        let id = Id::from_index({
            if let Some(previously_unused_index) = self.unused_indices.iter().next().map(|i| *i) {
                self.debug_assert_id_validity(Id::from_index(previously_unused_index), false);
                self.unused_indices.remove(&previously_unused_index);
                self.elements[previously_unused_index] = element;
                previously_unused_index
            } else {
                self.elements.push(element);
                self.elements.len() - 1
            }
        });

        self.debug_assert_last_element_is_used();
        self.debug_assert_id_validity(id, true);
        id
    }



    /// Return a reference to the element that this id points to
    pub fn get(&self, element: Id<T>) -> Option<&T> {
        if self.index_is_currently_used(element.index_value()) {
            self.elements.get(element.index_value())
        } else { None }
    }

    /// Return a mutable reference to the element that this id points to
    pub fn get_mut<'s>(&'s mut self, element: Id<T>) -> Option<&'s mut T> {
        if self.index_is_currently_used(element.index_value()) {
            self.elements.get_mut(element.index_value())
        } else { None }
    }


    /// Swap the elements pointed to. Panic on invalid Id parameter.
    pub fn swap_elements(&mut self, id1: Id<T>, id2: Id<T>){
        self.debug_assert_id_validity(id1, true);
        self.debug_assert_id_validity(id2, true);
        self.elements.swap(id1.index_value(), id2.index_value());
    }

    /// Removes all elements, instantly deallocating
    pub fn clear(&mut self){
        self.elements.clear();
        self.unused_indices.clear();
        debug_assert!(self.is_empty());
    }

    /// removes unused elements at the end of the internal vector
    /// and shrinks the internal vector itself
    // TODO test
    pub fn shrink_to_fit(&mut self){
        self.elements.shrink_to_fit();
        self.unused_indices.shrink_to_fit(); // bottleneck? reinserts all elements into a new map
        self.debug_assert_last_element_is_used();
    }

    /// Make this map have a continuous flow of indices, having no wasted allocation
    /// and calling remap(old_id, new_id) for every element that has been moved to a new Id
    // TODO test
    // #[must_use]
    pub fn pack<F>(&mut self, remap: F) where F: Fn(&mut Self, Id<T>, Id<T>) {
        let unused_indices = ::std::mem::replace(
            &mut self.unused_indices,
            HashSet::new() // does not allocate
        );

        for unused_index in unused_indices.into_iter() {
            // unused_index may have already been removed in a previous iteration, so check
            if self.index_is_in_range(unused_index){
                self.debug_assert_last_element_is_used();
                let last_used_element_index = self.elements.len() - 1;

                self.elements.swap(last_used_element_index, unused_index);
                self.elements.pop(); // pop the (last & unused) element
                self.pop_back_unused(); // pop not-anymore-guarded unused elements

                remap(self, Id::from_index(last_used_element_index), Id::from_index(unused_index));
            }
        }

        self.shrink_to_fit();
    }




    /// Used for immutable access to ids and elements
    pub fn iter<'s>(&'s self) -> Iter<'s, T> {
        Iter {
            inclusive_front_index: 0,
            exclusive_back_index: self.elements.len(),
            storage: self
        }
    }

    // pub fn iter_mut<'s>(&'s mut self) -> IterMut cannot be implemented safely
    // because it would require multiple mutable references

    pub fn into_elements(self) -> IntoElements<T> {
        IntoElements { map: self }
    }

    pub fn drain_elements(&mut self) -> DrainElements<T> {
        DrainElements { map: self }
    }

    /// Used for immutable direct access to all used elements
    pub fn elements<'s>(&'s self) -> ElementIter<'s, T> {
        ElementIter { iter: self.iter() }
    }

    /// Used for immutable indirect access
    pub fn ids<'s>(&'s self) -> IdIter<'s, T> {
        IdIter { iter: self.iter() }
    }

    /// Used for full mutable access, but allowing inserting and deleting while iterating.
    /// The iterator will keep an independent state, in order to un-borrow the underlying map.
    /// This may be more expensive than `iter`,
    /// because it needs to clone the internal set of unused ids.
    pub fn get_ids(&self) -> OwnedIdIter<T> {
        OwnedIdIter {
            inclusive_front_index: 0,
            exclusive_back_index: self.elements.len(),
            unused_ids: self.unused_indices.clone(), // TODO without clone // TODO try copy-on-write?
            marker: ::std::marker::PhantomData,
        }
    }


    /// Compares if two id-maps contain the same ids, ignoring elements.
    /// Complexity of O(n)
    pub fn ids_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self.ids().all(|id| other.contains(id))
    }

    /// Compares if two id-maps contain the same elements, ignoring ids.
    /// Worst case complexity of O(n^2)
    pub fn elements_eq(&self, other: &Self) -> bool where T: PartialEq {
        self.len() == other.len() && self.elements().all(|element| {
            other.contains_element(element)
        })
    }

    /// Worst case complexity of O(n)
    pub fn contains_element(&self, element: &T) -> bool where T: PartialEq {
        // cannot use self.elements.contains() because it contains removed elements
        self.find_id_of_element(element).is_some()
    }

    /// Worst case complexity of O(n)
    pub fn find_id_of_element(&self, element: &T) -> Option<Id<T>> where T: PartialEq {
        self.iter().find(|&(_, e)| element == e)
            .map(|(id, _)| id)
    }

}


// enable using .collect() on an iterator to construct self
impl<T> ::std::iter::FromIterator<T> for IdMap<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        IdMap::from_vec(iter.into_iter().collect())
    }
}

// enable using .collect() on self
impl<T> ::std::iter::IntoIterator for IdMap<T> {
    type Item = T;
    type IntoIter = IntoElements<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_elements()
    }
}

impl<T> From<Vec<T>> for IdMap<T> {
    fn from(vec: Vec<T>) -> Self {
        IdMap::from_vec(vec)
    }
}



impl<T> ::std::ops::Index<Id<T>> for IdMap<T> {
    type Output = T;
    fn index(&self, element: Id<T>) -> &T {
        debug_assert!(self.contains(element), "Indexing with invalid Id: `{:?}` ", element);
        &self.elements[element.index_value()]
    }
}

impl<T> ::std::ops::IndexMut<Id<T>> for IdMap<T> {
    fn index_mut(&mut self, element: Id<T>) -> &mut T {
        debug_assert!(self.contains(element), "Indexing-Mut with invalid Id: `{:?}` ", element);
        &mut self.elements[element.index_value()]
    }
}


/// Equality means: The same Ids pointing to the same elements, ignoring deleted elements.
/// Complexity of O(n)
impl<T> Eq for IdMap<T> where T: Eq {}
impl<T> PartialEq for IdMap<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter()
            .zip(other.iter()) // use iterators to automatically ignore deleted elements
            .all(|((id_a, element_a), (id_b, element_b))| {
                id_a == id_b && element_a == element_b
            })
    }
}









fn iter_next(
    inclusive_front_index: &mut Index,
    exclusive_back_index: &mut Index,
    unused_ids: &HashSet<Index>
) -> Option<Index>
{
    // skip unused elements
    while inclusive_front_index < exclusive_back_index &&
        unused_ids.contains(inclusive_front_index)
    {
        *inclusive_front_index += 1;
    }

    // consume next element
    let index = *inclusive_front_index;
    *inclusive_front_index += 1;

    if index < *exclusive_back_index {
        Some(index)
    } else { None }
}

fn iter_next_back(
    inclusive_front_index: &mut Index,
    exclusive_back_index: &mut Index,
    unused_ids: &HashSet<Index>
) -> Option<Index>
{
    // skip unused elements
    while *exclusive_back_index > *inclusive_front_index
        && unused_ids.contains(&(*exclusive_back_index - 1))
    {
        *exclusive_back_index -= 1;
    }

    // consume next element
    // back_index - 1 now points to exactly the next_back element
    if *exclusive_back_index > *inclusive_front_index {
        *exclusive_back_index -= 1;
        Some(*exclusive_back_index)

    } else {
        None
    }
}




pub struct Iter<'s, T: 's> {
    inclusive_front_index: Index,
    exclusive_back_index: Index,
    storage: &'s IdMap<T>,
}

impl<'s, T: 's> Iterator for Iter<'s, T> {
    type Item = (Id<T>, &'s T);

    fn next(&mut self) -> Option<Self::Item> {
        iter_next(
            &mut self.inclusive_front_index,
            &mut self.exclusive_back_index,
            &self.storage.unused_indices
        ).map(|index|{
            let id = Id::from_index(index);
            (id, &self.storage[id])
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_remaining = self.exclusive_back_index - self.inclusive_front_index;
        let unused_elements = self.storage.unused_indices.len();
        let min_remaining = max_remaining.checked_sub(unused_elements).unwrap_or(0);
        (min_remaining, Some(max_remaining))
    }
}

impl<'s, T: 's> DoubleEndedIterator for Iter<'s, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        iter_next_back(
            &mut self.inclusive_front_index,
            &mut self.exclusive_back_index,
            &self.storage.unused_indices
        ).map(|index|{
            let id = Id::from_index(index);
            (id, &self.storage[id])
        })
    }
}



pub struct ElementIter<'s, T: 's> {
    iter: Iter<'s, T>,
}

impl<'s, T: 's> Iterator for ElementIter<'s, T> {
    type Item = &'s T;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.iter.next().map(|(_, element)| element)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'s, T: 's> DoubleEndedIterator for ElementIter<'s, T> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        self.iter.next_back().map(|(_, element)| element)
    }
}


/// Note: always iterates backwards, because it just calls IdMap.pop()
pub struct IntoElements<T> {
    map: IdMap<T>, // map.unused_ids will be updated to allow len() and speed up remaining lookups
}

impl<T> ::std::iter::ExactSizeIterator for IntoElements<T> {}
impl<T> Iterator for IntoElements<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.map.pop_element()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}


/// Note: always iterates backwards, because it just calls IdMap.pop()
pub struct DrainElements<'s, T: 's> {
    map: &'s mut IdMap<T>,
}

impl<'s, T: 's> ::std::iter::ExactSizeIterator for DrainElements<'s, T> {}
impl<'s, T: 's> Iterator for DrainElements<'s, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.map.pop_element()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}

impl<'s, T: 's> Drop for DrainElements<'s, T> {
    fn drop(&mut self) {
        self.map.clear();
    }
}




pub struct IdIter<'s, T: 's> {
    iter: Iter<'s, T>,
}

impl<'s, T: 's> Iterator for IdIter<'s, T> {
    type Item = Id<T>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.iter.next().map(|(id, _)| id) // relies on compiler optimization for performance
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'s, T: 's> DoubleEndedIterator for IdIter<'s, T> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        self.iter.next_back().map(|(id, _)| id)
    }
}






pub struct OwnedIdIter<T> {
    inclusive_front_index: Index,
    exclusive_back_index: Index,
    unused_ids: HashSet<Index>,
    marker: ::std::marker::PhantomData<T>,
}

impl<T> Iterator for OwnedIdIter<T> {
    type Item = Id<T>;

    fn next(&mut self) -> Option<Id<T>> {
        iter_next(
            &mut self.inclusive_front_index,
            &mut self.exclusive_back_index,
            &self.unused_ids
        ).map(|index|
            Id::from_index(index)
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_remaining = self.exclusive_back_index - self.inclusive_front_index;
        let unused_elements = self.unused_ids.len();
        let min_remaining = max_remaining.checked_sub(unused_elements).unwrap_or(0);
        (min_remaining, Some(max_remaining))
    }
}

impl<T> DoubleEndedIterator for OwnedIdIter<T> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        iter_next_back(
            &mut self.inclusive_front_index,
            &mut self.exclusive_back_index,
            &self.unused_ids
        ).map(|index|
            Id::from_index(index)
        )
    }
}












#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_from_iterator(){
        let vec = vec![0, 1, 2, 5];
        let map = vec.into_iter().collect::<IdMap<_>>();
        assert_eq!(map.elements, vec![0, 1, 2, 5]);
    }

    #[test]
    pub fn test_from_vec(){
        let vec = vec![0, 1, 2, 5];
        let map = IdMap::from_vec(vec);
        assert_eq!(map.elements, vec![0, 1, 2, 5]);
    }

    #[test]
    pub fn test_from_macro(){
        let map = id_map!(0, 1, 2, 5);
        assert_eq!(map.elements, vec![0, 1, 2, 5]);
    }

    #[test]
    pub fn test_insert_and_remove_single_element(){
        let mut map = IdMap::new();

        let id_0 = map.insert(0); {
            assert_eq!(map.len(), 1, "map length after inserting");
            assert!(!map.is_empty(), "map emptiness after inserting");
            assert!(map.contains(id_0), "containing `0` after inserting `0`");
            assert_eq!(map.get(id_0), Some(&0), "indexing `Some` after inserting `0`");
        }

        map.remove(id_0); {
            assert_eq!(map.get(id_0), None, "indexing `None` after deleting");
            assert_eq!(map.len(), 0, "map length after deleting");
            assert!(!map.contains(id_0), "not containing `0` after removing `0`");
            assert!(map.is_empty(), "map emptiness after deleting");
        }

        let id_1 = map.insert(1); {
            assert!(map.contains(id_0), "containing overwritten `0` after inserting `1` into deleted slot");
            assert!(map.contains(id_1), "containing `1` after inserting `1` into deleted slot");
            assert_eq!(map.get(id_1), Some(&1), "indexing `Some` after inserting into deleted slot");
            assert_eq!(map.get(id_0), Some(&1), "reusing unused id (old id pointing to new element)");
            assert_eq!(map.len(), 1, "map length after inserting into deleted slot");
            assert!(!map.is_empty(), "map emptiness after inserting into deleted slot");
        }
    }

    #[test]
    pub fn test_insert_and_remove_multiple_elements(){
        let mut map = IdMap::new();
        let len = 42;

        for i in 0..42 {
            assert!(!map.contains(Id::from_index(i)), "unused it being invalid");
            let id = map.insert(i);
            assert!(map.contains(id), "used id being valid");
        }

        assert_eq!(map.len(), len, "map length after inserting multiple elements");

        while let Some(remaining_id) = map.ids().next() {
            assert!(map.contains(remaining_id), "used id being valid");
            map.remove(remaining_id);
            assert!(!map.contains(remaining_id), "unused it being invalid");
        }
    }

    #[test]
    pub fn test_pop(){
        let mut map = id_map!(0, 2, 5);
        assert_eq!(map.pop(), Some((Id::from_index(2), 5)), "`pop()` returning the last element");
        assert!(map.is_packed(), "`pop()`not inserting into `unused_ids`");

        map.remove(Id::from_index(0));
        assert!(!map.is_empty());
        assert!(!map.is_packed());

        assert_eq!(map.pop(), Some((Id::from_index(1), 2)));
        assert!(map.is_empty(), "`pop()` clearing the map");
        assert!(map.is_packed(), "`pop()` removing unused ids at the back");

        assert_eq!(map.pop(), None, "`pop()` returning `None` from map");
        assert!(map.is_empty());
    }

    #[test]
    pub fn test_into_iterator(){
        let map = IdMap {
            elements: vec![0, 2, 3, 4],
            unused_indices: HashSet::new(),
        };

        assert_eq!(
            map.into_iter().collect::<Vec<_>>(),
            vec![4, 3, 2, 0],
            "into_iterator contains all elements"
        );
    }

    #[test]
    pub fn test_drain(){
        let mut map = id_map!(0, 1, 2, 3);
        assert_eq!(map.drain_elements().next(), Some(3));
        assert!(map.is_empty(), "aborting drain clears map");
    }

    #[test]
    pub fn test_into_iterator_with_deleted_elements(){
        let mut map = IdMap::new();
        let zero = map.insert(0);
        let two = map.insert(2);
        map.insert(3);
        map.insert(4);

        map.remove(zero);
        map.remove(two);

        assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![4, 3], "into_iter containing only non-removed elements")
    }

    #[test]
    pub fn test_elements_iter(){
        let mut map = id_map!(0, 1, 2, 5);

        map.remove(Id::from_index(1));
        assert_eq!(map.len(), 3, "removing decrements len");
        assert!(!map.is_packed());

        assert_eq!(
            map.elements().collect::<Vec<_>>(),
            vec![&0, /*deleted 1,*/ &2, &5],
            "iter non-removed elements"
        );

        assert_eq!(
            map.elements().rev().collect::<Vec<_>>(),
            vec![&5, &2, /*deleted 1,*/ &0],
            "double ended element iter"
        );

        assert_eq!(
            map.ids()
                .map(|id| id.index_value())
                .collect::<Vec<_>>(),

            vec![0, /*deleted 1,*/ 2, 3],
            "iter non-removed ids"
        );

        assert_eq!(
            map.ids().rev()
                .map(|id| id.index_value())
                .collect::<Vec<_>>(),

            vec![3, 2, /*deleted 1,*/ 0],
            "double ended id iter"
        );

        assert_eq!(
            map.get_ids()
                .map(|id| {
                    map.remove(id);
                    id.index_value()
                })
                .collect::<Vec<_>>(),

            vec![0, /*deleted 1,*/ 2, 3],
            "owning id iter"
        );
    }


    /// Eq considers maps equal which have
    /// the same ids pointing to the same elements
    #[test]
    pub fn test_eq(){
        let mut map1 = id_map!(0,2,2,4,4);
        let mut map2 = id_map!(1,2,3,4,5);

        map1.remove(Id::from_index(0));
        map1.remove(Id::from_index(2));
        map1.remove(Id::from_index(4));
        assert_ne!(map1, map2);

        map2.remove(Id::from_index(4));
        map2.remove(Id::from_index(0));
        map2.remove(Id::from_index(2));
        assert_eq!(map1, map2);
    }


    #[test]
    pub fn test_elements_eq(){
        let     map1 = id_map!(3,4,2,5,1);
        let mut map2 = id_map!(1,2,3,4,5);
        assert!(map1.elements_eq(&map2));

        map2.pop();
        assert!(!map1.elements_eq(&map2));
    }

    #[test]
    pub fn test_ids_eq(){
        let mut map1 = id_map!(3,4,2,5,1);
        let mut map2 = id_map!(1,2,3,4,5);

        map1.remove(Id::from_index(0));
        map1.remove(Id::from_index(3));
        assert!(!map1.ids_eq(&map2));

        map2.remove(Id::from_index(0));
        map2.remove(Id::from_index(3));
        assert!(map1.ids_eq(&map2));
    }

    #[test]
    pub fn test_swap(){
        let mut map = id_map!(1,2,3);

        map.swap_elements(
            Id::from_index(0),
            Id::from_index(1),
        );

        assert_eq!(map.elements, vec![2, 1, 3]);
    }


    // TODO test repeated random removing and inserting

}