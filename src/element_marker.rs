use ::std::collections::HashSet;
use ::id::Index;


/// used to test whether a specific element is deleted or used
pub trait ElementMarker : Default {
    fn with_element_capacity(size: usize) -> Self;

    /// returns if the old value was used
    fn mark_element_used(&mut self, index: Index, used: bool) -> bool;
    fn element_is_used(&self, index: Index) -> bool;

    type UnusedElementIter<'s>: ExactSizeIterator + Iterator<Item = &'s Index>; // TODO fix lifetime issues differently?
    fn unused_elements(&self) -> Self::UnusedElementIter;

    fn reserve_elements(&mut self);
    fn shrink_to_fit(&mut self);
    fn clear(&mut self);
}




/// Keeps an internal HashSet of all unused indices, which is optimized for rather full id-vecs
/// with not too much deleted elements at the same time
#[derive(Clone, Default)]
pub struct HashSetElementMarker {
    unused_indices: HashSet<Index>,
}

impl ElementMarker for HashSetElementMarker {
    fn with_element_capacity(size: usize) -> Self {
        Self::default() // does not depend on element count, but on unused-element-count
    }

    /// returns if the element was used prior to calling this fn
    fn mark_element_used(&mut self, index: usize, used: bool) -> bool {
        if used {
            self.unused_indices.remove(&index)

        } else {
            self.unused_indices.insert(index)
        }
    }

    fn element_is_used(&self, index: usize) -> bool {
        !self.unused_indices.contains(&index)
    }

    type UnusedElementIter<'s> = ::std::collections::hash_set::Iter<'s, usize>;

    fn unused_elements(&self) -> Self::UnusedElementIter {
        self.unused_indices.iter()
    }

    fn reserve_elements(&mut self) {
        // does not depend on element count, but on unused-element-count
    }

    fn shrink_to_fit(&mut self) {
        self.unused_indices.shrink_to_fit();
    }

    fn clear(&mut self) {
        self.unused_indices.clear();
    }
}