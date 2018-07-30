use ::element_marker::ElementMarker;
use ::bit_vec::BitVec;
use ::id::Index;

/// Keeps an internal BitVec of all unused indices, which is optimized for rather empty id-vecs
/// with many deleted elements at the same time
#[derive(Clone, Default)]
pub struct BitVecElementMarker {
    used_indices: BitVec<u128>,
    unused_elements_len: usize,
}

impl ElementMarker for BitVecElementMarker {
    fn with_element_capacity(size: usize) -> Self {
        BitVecElementMarker {
            used_indices: BitVec::with_capacity(size),
            unused_elements_len: 0,
        }
    }

    /// returns if the element was used prior to calling this fn
    fn mark_element_used(&mut self, index: Index, mark_used: bool) -> bool {
        let was_used_before = self.element_is_used(index);

        if mark_used != was_used_before {
//           TODO if !mark_used { self.unused_elements_len += 1 }

            self.reserve_elements(index);
            while self.used_indices.len() < index - 1 { // - 1 because we are going to set the last index or not enter the loop at all
                self.used_indices.push(false);
            }

            debug_assert!(index < self.used_indices.len(), "BitVecMarker has not enough bits");
            self.used_indices.set(index, mark_used);
        }

        was_used_before
    }


    fn element_is_used(&self, index: Index) -> bool {
        self.used_indices.get(index).unwrap_or(false)
    }


    fn unused_elements(&self) -> Self::UnusedElementIter {
        // TODO this 'owning' iterator should borrow, as soon as 'lifetimes in associated types' becomes stable
        ClonedBitVecMarkerIter {
            used_element_bits: self.used_indices.clone(),
        }
    }

    // TODO this 'owning' iterator should borrow, as soon as 'lifetimes in associated types' becomes stable
    type UnusedElementIter = ClonedBitVecMarkerIter;

    fn unused_element_count(&self) -> usize {
        self.unused_elements_len
    }

    fn reserve_elements(&mut self, element_count: usize) {
        self.used_indices.reserve(element_count)
    }

    fn shrink_to_fit(&mut self) {
        self.used_indices.shrink_to_fit();
    }

    fn clear(&mut self) {
        self.used_indices.clear();
    }
}

pub struct ClonedBitVecMarkerIter {
    /// TODO this 'owning' iterator should borrow, as soon as 'lifetimes in associated types' becomes stable
    used_element_bits: BitVec,
    next: Index,
}

impl ExactSizeIterator for ClonedBitVecMarkerIter {
    /* hash_set.into_iter implements ExactSizeIterator */
}

impl Iterator for ClonedBitVecMarkerIter {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next < self.used_element_bits.len() && self.used_element_bits[self.next] {
            self.next += 1; // skip used elements
        }

        if self.next < self.used_element_bits.len() {
            debug_assert!(!self.used_element_bits.get(next), "bit vec iter element being used");
            let current = next;
            self.next += 1;
            Some(current)

        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.used_element_bits.len() - self.next; // TODO -1 ??
        (rem, Some(rem))
    }
}

