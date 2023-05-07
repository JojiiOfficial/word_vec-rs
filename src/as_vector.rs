use crate::vector::Vector;

/// Trait to borrow any type as Vector.
pub trait AsVectorRef<'v, 't> {
    fn as_vec_ref(&self) -> Vector<'v, 't>;
}
