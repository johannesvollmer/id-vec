use ::id::Index;

#[cfg(feature = "bit-vec-marker")]
pub mod bit_vec_marker;

// required because it is a default
pub mod hash_set_marker;

/// Used to test whether a specific element is deleted or used
pub trait ElementMarker : Default {
    fn with_element_capacity(size: usize) -> Self;

    /// Returns if the old value was used
    fn mark_element_used(&mut self, index: Index, used: bool) -> bool;

    /// Return true if the element is alive, false if it was deleted
    fn element_is_used(&self, index: Index) -> bool;


    /// Return Self::UnusedElementIter to iterate over all unused elements in this element_marker
    fn unused_elements(&self) -> Self::UnusedElementIter; // TODO use associated lifetime
    type UnusedElementIter: Sized + Iterator<Item = Index>; // TODO use associated lifetime

    fn unused_element_count(&self) -> usize;

    /// reserve space for _used_ elements in the id-vec
    fn reserve_elements(&mut self, new_element_count: usize);
    fn shrink_to_fit(&mut self);
    fn clear(&mut self);
}







