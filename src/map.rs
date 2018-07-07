
use ::std::collections::HashMap;
use ::std::collections::HashSet;

use ::id::*;



/// behaves like a hash map with automatic key creation (unique id automatically created)
/// internally utilizes a dense vector, and reuses deleted element slots
/// which results in ids being reused (be careful not to delete an element that is still referenced because that will be overwritten)
// TODO use rusts safety mechanisms to allow (but not enforce) stronger id lifetime safety? OwnedId<T>?
// TODO impl Eq, Clone, Debug, ...
pub struct IdMap<T> {
    /// packed dense vector, containing alive and dead elements
    elements: Vec<T>,

    /// contains all unused ids which are allowed to be overwritten
    currently_unused_ids: HashSet<Index>, // TODO if iteration is too slow, use both Vec<NextUnusedIndex> and BitVec
}



impl<T> IdMap<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        IdMap {
            elements: Vec::with_capacity(capacity),
            currently_unused_ids: HashSet::new(),
        }
    }

    pub fn contains(&self, element: Id<T>) -> bool {
        element.index < self.elements.len()
            && self.id_is_currently_used(element)
    }

    pub fn is_packed(&self) -> bool {
        self.currently_unused_ids.is_empty()
    }

    fn id_is_currently_used(&self, element: Id<T>) -> bool {
        !self.currently_unused_ids.contains(&element.index)
    }

    fn id_is_currently_valid(&self, element: Id<T>) -> bool {
        self.id_is_currently_used(element)
            && element.index < self.elements.len()
    }

    #[inline(always)]
    fn debug_assert_id_is_in_use(&self, element: Id<T>, used: bool){
        debug_assert_eq!(self.id_is_currently_used(element), used);
    }

    #[inline(always)]
    fn debug_assert_id_is_valid(&self, element: Id<T>, used: bool){
        debug_assert_eq!(self.id_is_currently_valid(element), used);
    }


    /// enable the specified id to be reused
    /// (does not directly deallocate the element, but will be overwritten later)
    /// there should not exist any ids pointing to that element after this call, because this id may be reused
    pub fn mark_unused(&mut self, element: Id<T>) {
        self.debug_assert_id_is_in_use(element, true);
        self.currently_unused_ids.insert(element.index);
    }

    /// associate the specified element with a currently unused id
    /// (may overwrite (thus drop) unused contents)
    pub fn insert(&mut self, element: T) -> Id<T> {
        Id::from_index({
            if let Some(previously_unused_index) = self.currently_unused_ids.iter().next().map(|i| *i) {
                self.debug_assert_id_is_in_use(Id::from_index(previously_unused_index), false);
                self.currently_unused_ids.remove(&previously_unused_index);
                self.elements[previously_unused_index] = element;
                previously_unused_index
            } else {
                self.elements.push(element);
                self.elements.len() - 1
            }
        })
    }

    pub fn get(&self, element: Id<T>) -> Option<&T> {
        if self.id_is_currently_used(element) {
            self.elements.get(element.index)
        } else { None }
    }

    pub fn get_mut<'s>(&'s mut self, element: Id<T>) -> Option<&'s mut T> {
        if self.id_is_currently_used(element) {
            self.elements.get_mut(element.index)
        } else { None }
    }

    /// used for immutable access to ids and elements
    pub fn iter<'s>(&'s self) -> Iter<'s, T> {
        Iter {
            inclusive_front_index: 0,
            exclusive_back_index: self.elements.len(),
            storage: self
        }
    }

    /// used for immutable direct access to all used elements
    pub fn elements<'s>(&'s self) -> ElementIter<'s, T> {
        ElementIter { iter: self.iter() }
    }

    /// used for immutable indirect access
    pub fn ids<'s>(&'s self) -> IdIter<'s, T> {
        IdIter { iter: self.iter() }
    }

    /// used for mutable access
    /// may be more expensive than `iter` because it needs to clone the internal set of unused ids in order to unborrow &self
    pub fn ids_mut(&self) -> IdIterMut<T> {
        IdIterMut {
            inclusive_front_index: 0,
            exclusive_back_index: self.elements.len(),
            unused_ids: self.currently_unused_ids.clone(), // TODO without clone // TODO try copy-on-write?
            marker: ::std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len() - self.currently_unused_ids.len()
    }

    /// returns a (map[old_id] -> new_id) for the caller to correct any ids that may have changed
    // TODO test
    pub fn packed_vec<'s>(&'s self) -> (Vec<&'s T>, HashMap<Id<T>, Index>) {
        let mut remap = HashMap::new();
        let mut vec = Vec::with_capacity(self.len());

        for (id, element) in self.iter() {
            remap.insert(id, vec.len());
            vec.push(element);
        }

        (vec, remap)
    }

    /*#[must_use]
    pub fn pack(&mut self) -> HashMap<Id<T>, Id<T>> {
        let mut remap = HashMap::new();

        for unused_id in self.currently_unused_ids.drain() {
            let last_id = self.elements.len() - 1;
            if unused_id != last_id {
                // swap the unused and the last element and pop the last element
                self.currently_unused_ids.swap(last_id, unused_id);
                self.elements.pop(); // remove the unused element

                remap.insert(Id::from_index(unused_id), Id::from_index(last_id));
            }
        }

        remap
    }*/

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
    storage: &'s IdMap<T>
}

impl<'s, T: 's> Iterator for Iter<'s, T> {
    type Item = (Id<T>, &'s T);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        iter_next(
            &mut self.inclusive_front_index,
            &mut self.exclusive_back_index,
            &self.storage.currently_unused_ids
        ).map(|index|{
            let id = Id::from_index(index);
            (id, &self.storage[id])
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_remaining = self.exclusive_back_index - self.inclusive_front_index;
        let unused_elements = self.storage.currently_unused_ids.len();
        let min_remaining = max_remaining.checked_sub(unused_elements).unwrap_or(0);
        (min_remaining, Some(max_remaining))
    }
}

impl<'s, T: 's> DoubleEndedIterator for Iter<'s, T> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        iter_next_back(
            &mut self.inclusive_front_index,
            &mut self.exclusive_back_index,
            &self.storage.currently_unused_ids
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






pub struct IdIterMut<T> {
    inclusive_front_index: Index,
    exclusive_back_index: Index,
    unused_ids: HashSet<Index>,
    marker: ::std::marker::PhantomData<T>,
}

impl<T> Iterator for IdIterMut<T> {
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

impl<T> DoubleEndedIterator for IdIterMut<T> {
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



impl<T> ::std::ops::Index<Id<T>> for IdMap<T> {
    type Output = T;
    fn index(&self, element: Id<T>) -> &T {
        self.debug_assert_id_is_valid(element, true);
        &self.elements[element.index]
    }
}

impl<T> ::std::ops::IndexMut<Id<T>> for IdMap<T> {
    fn index_mut(&mut self, element: Id<T>) -> &mut T {
        self.debug_assert_id_is_valid(element, true);
        &mut self.elements[element.index]
    }
}