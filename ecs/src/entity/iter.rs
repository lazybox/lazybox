use std::slice;
use bit_set;

use policy::{Id, IdSet};
use entity::Accessor;

/// Creates an accessor iterator from a slice of `Id`.
///
/// This is unsafe because ids might be refering to a removed entity
pub unsafe fn accessors_from_slice(ids: &[Id]) -> SliceIter {
    SliceIter { inner: ids.iter() }
}

pub struct SliceIter<'a> {
    inner: slice::Iter<'a, Id>,
}

impl<'a> Iterator for SliceIter<'a> {
    type Item = Accessor<'a>;

    fn next(&mut self) -> Option<Accessor<'a>> {
        self.inner.next().map(|&id| unsafe { Accessor::new_unchecked(id) })
    }
}

/// Creates an accessor iterator from a HashSet of `Id`.
///
/// This is unsafe because ids might be refering to a removed entity
pub unsafe fn accessors_from_set(ids: &IdSet) -> SetIter {
    SetIter { inner: ids.iter() }
}


/// An `Id` Set iterator
pub struct SetIter<'a> {
    inner: bit_set::Iter<'a, u32>,
}

impl<'a> Iterator for SetIter<'a> {
    type Item = Accessor<'a>;

    #[inline]
    fn next(&mut self) -> Option<Accessor<'a>> {
        self.inner.next().map(|id| unsafe { Accessor::new_unchecked(id as Id) })
    }
}