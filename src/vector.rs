use std::ops::Add;

use crate::as_vector::AsVectorRef;
use borrowme::borrowme;
use nalgebra::DVectorView;

/// A single WordVector
#[borrowme]
#[borrowed_attr(derive(Copy))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[owned_attr(cfg_attr(feature = "serde", derive(serde::Deserialize)))]
pub struct Vector<'v, 't> {
    #[borrowme(borrow_with = Vec::as_slice)]
    data: &'v [f32],
    term: &'t str,
}

impl<'v, 't> Vector<'v, 't> {
    #[inline]
    pub fn new(data: &'v [f32], term: &'t str) -> Self {
        Self { data, term }
    }

    #[inline]
    pub fn data(&self) -> &[f32] {
        self.data
    }

    #[inline]
    pub fn term(&self) -> &str {
        self.term
    }

    #[inline]
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Calculates the cosine similarity between two words.
    pub fn cosine<'v2, 't2, R>(&self, other: &R) -> f32
    where
        R: AsVectorRef<'v2, 't2>,
    {
        let other = other.as_vec_ref();

        let dot = self.dot(&other);
        if dot == 0.0 {
            return 0.0;
        }

        let div = self.length() * other.length();
        if div == 0.0 {
            return 0.0;
        }

        dot / div
    }

    /// Calculates the dot product of two vectors
    pub fn dot<'v2, 't2, R>(&self, other: &R) -> f32
    where
        R: AsVectorRef<'v2, 't2>,
    {
        // self.vec().dot(&other.as_vec_ref().vec())
        let other = other.as_vec_ref();
        self.data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    #[inline]
    pub fn vec(&self) -> DVectorView<'_, f32> {
        DVectorView::from_slice(self.data, 1)
    }

    /// Calculates the 2-norm
    #[inline]
    pub fn length(&self) -> f32 {
        // self.vec().norm()
        self.data.iter().map(|i| i.powi(2)).sum::<f32>().sqrt()
    }
}

impl OwnedVector {
    #[inline]
    pub fn new_raw(data: Vec<f32>, term: String) -> Self {
        Self { data, term }
    }

    #[inline]
    pub fn new(data: &[f32], term: &str) -> Self {
        borrowme::ToOwned::to_owned(&Vector::new(data, term))
    }

    /// Returns a reference to the data of the owned vector.
    #[inline]
    pub fn as_ref(&self) -> Vector {
        Vector::new(&self.data, &self.term)
    }

    #[inline]
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    #[inline]
    pub fn term(&self) -> &str {
        &self.term
    }

    #[inline]
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Calculates the 2-norm
    #[inline]
    pub fn length(&self) -> f32 {
        self.as_ref().length()
    }

    /// Calculates the 2-norm
    #[inline]
    pub fn dot<'v, 't, R: AsVectorRef<'v, 't>>(&self, other: &R) -> f32 {
        self.as_ref().dot(other)
    }

    /// Calculates the cosine similarity between two vectors.
    #[inline]
    pub fn cosine<'v2, 't2, R>(&self, other: &R) -> f32
    where
        R: AsVectorRef<'v2, 't2>,
    {
        self.as_ref().cosine(other)
    }
}

impl<'v, 't, 'v2, 't2, T> Add<T> for Vector<'v, 't>
where
    T: AsVectorRef<'v2, 't2>,
{
    type Output = OwnedVector;

    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.as_vec_ref();
        assert_eq!(self.dim(), rhs.dim());

        let data: Vec<_> = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|i| i.0 + i.1)
            .collect();

        OwnedVector::new_raw(data, format!("{} {}", self.term, rhs.term()))
    }
}

impl<'v, 't, T> Add<T> for OwnedVector
where
    T: AsVectorRef<'v, 't>,
{
    type Output = OwnedVector;

    fn add(self, rhs: T) -> Self::Output {
        self.as_ref() + rhs
    }
}

impl<'v, 't> AsVectorRef<'v, 't> for &Vector<'v, 't> {
    #[inline]
    fn as_vec_ref(&self) -> Vector<'v, 't> {
        **self
    }
}

impl<'v, 't> AsVectorRef<'v, 't> for Vector<'v, 't> {
    #[inline]
    fn as_vec_ref(&self) -> Vector<'v, 't> {
        *self
    }
}

impl<'a> AsVectorRef<'a, 'a> for &'a OwnedVector {
    #[inline]
    fn as_vec_ref(&self) -> Vector<'a, 'a> {
        self.as_ref()
    }
}
