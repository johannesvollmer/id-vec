
use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::id::*;


/// create a new id_map by entering a series of values
macro_rules! id_map {
    ( $($element:expr),* ) => {
        IdMap::from_vec(vec![ $($element),* ])
    };
}


/// behaves like a hash map with automatic key creation (unique id automatically created)
/// internally utilizes a dense vector, and reuses deleted element slots
/// which results in ids being reused (be careful not to delete an element that is still referenced because that will be overwritten)
// TODO use rusts safety mechanisms to allow (but not enforce) stronger id lifetime safety? OwnedId<T>?
// TODO impl Eq, Clone, Debug, ...
pub struct IdMap<T> {
    /// packed dense vector, containing alive and dead elements.
    /// because removing the last element directly can be done efficiently,
    /// it is guaranteed that the last element is never unused.
    elements: Vec<T>,

    /// contains all unused ids which are allowed to be overwritten,
    /// will never contain the last ID, because the last id can be removed directly
    unused_indices: HashSet<Index>, // TODO if iteration is too slow, use both Vec<NextUnusedIndex> and BitVec
}



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
    pub fn from_vec(vec: Vec<T>) -> Self {
        IdMap {
            unused_indices: HashSet::new(), // no elements deleted
            elements: vec,
        }
    }


    /// Excludes deleted elements, and indices out of range
    fn contains(&self, element: Id<T>) -> bool {
        self.index_is_in_range(element.index_value())
          && self.index_is_currently_used(element.index_value())
    }

    /// Returns if this vector contains any deleted elements
    pub fn is_packed(&self) -> bool {
        self.unused_indices.is_empty()
    }



    /// Returns if this id is not deleted
    fn index_is_currently_used(&self, index: Index) -> bool {
        index + 1 == self.elements.len() || // last element is always used
            !self.unused_indices.contains(&index)
    }

    fn index_is_in_range(&self, index: Index) -> bool {
        index < self.elements.len()
    }

    #[inline(always)]
    fn debug_assert_id_validity(&self, element: Id<T>, validity: bool){
        debug_assert_eq!(self.contains(element), validity);
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


    /// Enable the specified id to be overwritten when a new element is inserted.
    /// This does not directly deallocate the element.
    /// Make sure that no ids pointing to that element exist after this call.
    pub fn remove(&mut self, element: Id<T>) {
        self.debug_assert_last_element_is_used();

        if self.index_is_in_range(element.index_value()) {

            // if exactly the last element, remove without inserting into unused_ids
            if element.index_value() + 1 == self.elements.len() {
                self.debug_assert_last_element_is_used();
                self.elements.pop();

                // remove all unused elements at the end of the vector
                // which may have been guarded by the (now removed) last element
                self.pop_back_unused();
                self.debug_assert_last_element_is_used();

            } else { // remove not-the-last element
                self.unused_indices.insert(element.index_value()); // may overwrite existing index
            }
        }

        self.debug_assert_id_validity(element, false);
        self.debug_assert_last_element_is_used();
    }

    /// Removes all elements, instantly deallocating
    fn clear(&mut self){
        self.elements.clear();
        self.unused_indices.clear();
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

    /// removes unused elements at the end of the internal vector
    /// and shrinks the internal vector itself
    /// may deallocate unused elements
    // TODO test
    pub fn shrink_to_fit(&mut self){
        self.elements.shrink_to_fit();
        self.unused_indices.shrink_to_fit(); // bottleneck? reinserts all elements into a new map
        self.debug_assert_last_element_is_used();
    }

    /// Removes an element from this map (the one element which is the least work to remove)
    /// may deallocate unused elements
    // TODO test
    pub fn pop(&mut self) -> Option<T> {
        self.debug_assert_last_element_is_used();

        let popped = self.elements.pop();
        self.pop_back_unused();

        self.debug_assert_last_element_is_used();
        popped
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

    pub fn len(&self) -> usize {
        debug_assert!(self.elements.len() >= self.unused_indices.len(), "More ids are not used than exist");
        self.elements.len() - self.unused_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Make this map have a continuous flow of indices, having no wasted allocation
    /// and calling remap(old_id, new_id) for every element that has been moved to a new Id
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
        self.debug_assert_id_validity(element, true);
        &self.elements[element.index_value()]
    }
}

impl<T> ::std::ops::IndexMut<Id<T>> for IdMap<T> {
    fn index_mut(&mut self, element: Id<T>) -> &mut T {
        self.debug_assert_id_validity(element, true);
        &mut self.elements[element.index_value()]
    }
}






fn iter_next(
    inclusive_front_index: &mut Index,
    exclusive_back_index: &mut Index,
    unused_ids: &HashSet<Index>
) -> Option<Index>
{
    while inclusive_front_index < exclusive_back_index &&
        unused_ids.contains(inclusive_front_index)
        {
            *inclusive_front_index += 1;
        }

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
    while exclusive_back_index > inclusive_front_index && unused_ids.contains(exclusive_back_index) {
        *exclusive_back_index -= 1;
    }

    if exclusive_back_index > inclusive_front_index {
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
        self.map.pop()
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
        self.map.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
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
        assert_eq!(map.len(), 4);
        assert_eq!(map.elements, vec![0, 1, 2, 5]);
    }

    #[test]
    pub fn test_from_vec(){
        let vec = vec![0, 1, 2, 5];
        let map = IdMap::from_vec(vec);
        assert_eq!(map.len(), 4);
        assert_eq!(map.elements, vec![0, 1, 2, 5]);
    }

    #[test]
    pub fn test_from_macro(){
        let map = id_map!(0, 1, 2, 5);

        assert_eq!(map.len(), 4);
        assert_eq!(map.elements, vec![0, 1, 2, 5]);
    }


    #[test]
    pub fn test_into_iterator(){
        let map = IdMap {
            elements: vec![0, 2, 3, 4],
            unused_indices: HashSet::new(),
        };

        assert_eq!(map.len(), 4);
        assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![4, 3, 2, 0]);
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

        assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![4, 3])
    }


    #[test]
    pub fn test_single_element(){
        let mut map = IdMap::new();

        let id_0 = map.insert(0); {
            assert_eq!(map.len(), 1, "map length after inserting");
            assert!(!map.is_empty(), "map emptiness after inserting");
            assert_eq!(map.get(id_0), Some(&0), "indexing `Some` after inserting ");
        }

        map.remove(id_0); {
            assert_eq!(map.get(id_0), None, "indexing `None` after deleting");
            assert_eq!(map.len(), 0, "map length after deleting");
            assert!(map.is_empty(), "map emptiness after deleting");
        }

        let id_1 = map.insert(1); {
            assert_eq!(map.get(id_1), Some(&1), "indexing `Some` after inserting into deleted slot");
            assert_eq!(map.get(id_0), Some(&1), "reusing unused id (old id pointing to new element)");
            assert_eq!(map.len(), 1, "map length after inserting into deleted slot");
            assert!(!map.is_empty(), "map emptiness after inserting into deleted slot");
        }
    }
    #[test]
    pub fn test_multiple_elements(){
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


    // TODO test repeated random removing and inserting

}