use crate::{space::VecSpace, vector::Vector};

/// Iterator over all vectors in a [`VecSpace`]
pub struct VecSpaceIter<'a> {
    space: &'a VecSpace,
    pos: usize,
}

impl<'a> VecSpaceIter<'a> {
    #[inline]
    pub(crate) fn new(space: &'a VecSpace) -> Self {
        Self { space, pos: 0 }
    }
}

impl<'a> Iterator for VecSpaceIter<'a> {
    type Item = Vector<'a, 'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.space.get(self.pos)?;
        self.pos += 1;
        Some(vec)
    }
}
