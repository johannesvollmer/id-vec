use ::std::collections::HashSet;
use ::id::Index;


/// used to test whether a specific element is deleted or used
pub trait ElementMarker : Default {
    fn with_element_capacity(size: usize) -> Self;

    /// returns if the old value was used
    fn mark_element_used(&mut self, index: Index, used: bool) -> bool;
    fn element_is_used(&self, index: Index) -> bool;

    type UnusedElementIter: ExactSizeIterator + Iterator<Item = Index>;
    fn unused_elements(&self) -> Self::UnusedElementIter;

    fn reserve_elements(&mut self);
    fn shrink_to_fit(&mut self);
    fn clear(&mut self);
}
