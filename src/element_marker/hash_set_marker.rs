use ::element_marker::ElementMarker;
use ::std::collections::HashSet;
use ::id::Index;


/// Keeps an internal HashSet of all unused indices, which is optimized for rather full id-vecs
/// with not too many deleted elements at the same time
#[derive(Clone, Default)]
pub struct HashSetElementMarker {
    unused_indices: HashSet<Index>,
}

impl ElementMarker for HashSetElementMarker {
    fn with_element_capacity(size: usize) -> Self {
        Self::default() // does not depend on element count, but on unused-element-count
    }

    /// returns if the element was used prior to calling this fn
    fn mark_element_used(&mut self, index: Index, used: bool) -> bool {
        if used {
            self.unused_indices.remove(&index)

        } else {
            self.unused_indices.insert(index)
        }
    }

    fn element_is_used(&self, index: Index) -> bool {
        !self.unused_indices.contains(&index)
    }


    fn unused_elements(&self) -> Self::UnusedElementIter {
        // TODO this 'owning' iterator should borrow, as soon as 'lifetimes in associated types' becomes stable
        ClonedHashSetMarkerIter {
            into_iter: self.unused_indices.clone().into_iter()
        }
    }

    // TODO this 'owning' iterator should borrow, as soon as 'lifetimes in associated types' becomes stable
    type UnusedElementIter = ClonedHashSetMarkerIter;

    fn unused_element_count(&self) -> usize {
        self.unused_indices.len()
    }

    fn reserve_elements(&mut self, _element_count: usize) {
        // does not depend on element count, but on unused-element-count
    }

    fn shrink_to_fit(&mut self) {
        self.unused_indices.shrink_to_fit();
    }

    fn clear(&mut self) {
        self.unused_indices.clear();
    }
}

pub struct ClonedHashSetMarkerIter {
    /// TODO this 'owning' iterator should borrow, as soon as 'lifetimes in associated types' becomes stable
    into_iter: ::std::collections::hash_set::IntoIter<Index>,
}

impl ExactSizeIterator for ClonedHashSetMarkerIter {
    /* hash_set.into_iter implements ExactSizeIterator */
}

impl Iterator for ClonedHashSetMarkerIter {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.into_iter.size_hint()
    }
}
