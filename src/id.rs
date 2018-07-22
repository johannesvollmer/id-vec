pub type Index = usize; // TODO make this a type parameter
use ::vec::IdVec;

/// used as a key to access an instance inside a Storage<T>.
/// internally only an integer index (but with greater type safety)
// manually implemented hash, clone, copy, ..
pub struct Id<T> {
    index: Index,
    _marker: ::std::marker::PhantomData<T>,
}


impl<T> Id<T> {
    pub fn from_index(index: Index) -> Self {
        Id { index, _marker: ::std::marker::PhantomData, }
    }

    /// convienience function which allows writing the index first, and the storage afterwards
    /// example: the_selected_entity.of(entities);
    pub fn of<'s>(self, vec: &'s IdVec<T>) -> &'s T {
        &vec[self]
    }

    /// convienience function which allows writing the index first, and the storage afterwards
    /// example: the_selected_entity.of(entities);
    pub fn of_mut<'s>(self, vec: &'s mut IdVec<T>) -> &'s mut T {
        &mut vec[self]
    }

    pub fn index_value(self) -> Index {
        self.index
    }
}





impl<T> Eq for Id<T> {}
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Id<T>) -> bool {
        self.index == other.index
    }
}
impl<T> Copy for Id<T> {}
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> ::std::hash::Hash for Id<T> {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.index);
    }
}
impl<T> ::std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "Id#{:?}", self.index)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn internal_index(){
        for index in 0..32 {
            let id : Id<f32> = Id::from_index(index);
            let eq_id : Id<f32> = Id::from_index(index);
            let non_eq_id : Id<f32> = Id::from_index(index + 1);

            assert_eq!(id, eq_id);
            assert_ne!(id, non_eq_id);
            assert_eq!(id.index_value(), index);
        }
    }
}