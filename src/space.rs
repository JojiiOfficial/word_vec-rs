use crate::{as_vector::AsVectorRef, vector::Vector};
use std::collections::HashMap;

/// A memory optimized vector space that can handle a lot of high dimensional word vecs.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VecSpace {
    /// A big vector for vector data. Since all vectors have the same dimension we can simply
    /// calculate where data for a given vector lays.
    vec_data: Vec<f32>,

    /// A list of all terms
    words: Vec<String>,

    /// The dimension of the vector space.
    dimension: usize,

    /// Index for terms to their ID.
    term_map: Option<HashMap<String, u32>>,
}

impl VecSpace {
    /// Create a new empty word vector space with a given dimensions.
    #[inline]
    pub fn new(dimension: usize) -> Self {
        Self {
            vec_data: vec![],
            words: vec![],
            dimension,
            term_map: None,
        }
    }

    /// Enables mapping for terms to vectors. This requires more memory but makes searching for
    /// terms faster. Existing terms will be indexed when calling this function.
    #[inline]
    pub fn with_termmap(mut self) -> Self {
        self.term_map = Some(HashMap::new());

        if !self.is_empty() {
            self.index_terms();
        }

        self
    }

    /// Amount of vectors in the word vec space.
    #[inline]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns `true` if there is no vec in the vec space.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the dimension of the VecSpace.
    #[inline]
    pub fn dim(&self) -> usize {
        self.dimension
    }

    /// Inserts a word vector into the vecspace. Returns an error if the dimensions don't match.
    pub fn insert<'v, 't, R: AsVectorRef<'v, 't>>(&mut self, vec: R) -> Result<(), String> {
        let vec = vec.as_vec_ref();
        if vec.dim() != self.dimension {
            return Err(format!(
                "Tried to insert vec with dimension {} into space with dimension {}",
                vec.dim(),
                self.dim()
            ));
        }

        if let Some(term_map) = self.term_map.as_mut() {
            term_map.insert(vec.term().to_string(), self.words.len() as u32);
        }

        self.vec_data.extend_from_slice(vec.data());
        self.words.push(vec.term().to_string());
        Ok(())
    }

    /// Gets a vector with a given ID from the space.
    pub fn get(&self, pos: usize) -> Option<Vector> {
        let vec_idx = pos * self.dimension;
        let word = self.words.get(pos)?;
        let vec_data = self.vec_data.get(vec_idx..vec_idx + self.dimension)?;
        Some(Vector::new(vec_data, word))
    }

    pub fn top_k<'v, 't, R: AsVectorRef<'v, 't>>(&self, vec: &R, k: usize) -> Vec<(f32, Vector)> {
        let mut out = vec![];

        for v in (0..self.len()).map(|i| self.get(i).unwrap()) {
            let sim = v.cosine(vec);
            out.push((sim, v));
        }

        out.sort_by(|a, b| a.0.total_cmp(&b.0).reverse());
        out.truncate(k);
        out
    }

    /// Searches for a given term in the space
    #[inline]
    pub fn find_term<S: AsRef<str>>(&self, term: S) -> Option<Vector> {
        self.get(self.find_term_idx(term.as_ref())?)
    }

    /// Clears the vectors from the space.
    pub fn clear(&mut self) {
        self.vec_data.clear();
        self.words.clear();
        if let Some(term_map) = self.term_map.as_mut() {
            term_map.clear();
        }
    }

    /// Returns the vec ID of the given term
    #[inline]
    fn find_term_idx(&self, term: &str) -> Option<usize> {
        self.term_map.as_ref()?.get(term).map(|i| *i as usize)
    }

    /// Indexes the existing vectors.
    fn index_terms(&mut self) {
        let mut map = self.term_map.take().unwrap_or_default();
        map.clear();

        for (pos, term) in self.words.iter().cloned().enumerate() {
            map.insert(term, pos as u32);
        }

        self.term_map = Some(map);
    }
}

impl<'v, 't, V> Extend<V> for VecSpace
where
    V: AsVectorRef<'v, 't>,
{
    fn extend<T: IntoIterator<Item = V>>(&mut self, iter: T) {
        for i in iter {
            let i = i.as_vec_ref();
            if self.insert(i).is_err() {
                panic!(
                    "Tried to insert vector with dimension of {} into space with dimension of {}",
                    i.dim(),
                    self.dim(),
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::VecSpace;
    use crate::vector::Vector;

    fn get_vectors() -> [Vector<'static, 'static>; 3] {
        [
            Vector::new(&[1.0, 0.07, 23.1], "a"),
            Vector::new(&[0.13, 3.19, 3.12], "b"),
            Vector::new(&[3.193, 3.1, 32.1], "c"),
        ]
    }

    fn get_space() -> VecSpace {
        let mut space = VecSpace::new(3);
        space.extend(get_vectors().iter());
        space
    }

    #[test]
    fn test_space_get() {
        let space = get_space();
        let vectors = get_vectors();

        for (pos, exp_vec) in vectors.iter().enumerate() {
            let vec = space.get(pos).unwrap();
            assert_eq!(vec, *exp_vec);
        }
    }

    #[test]
    fn test_space_find() {
        // test indexing after inserting.
        let space = get_space().with_termmap();
        let vectors = get_vectors();

        for exp_vec in vectors {
            let vec = space.find_term(exp_vec.term()).unwrap();
            assert_eq!(vec, exp_vec);
        }

        // test indexing while inserting.
        let mut space = VecSpace::new(3).with_termmap();
        let vectors = get_vectors();
        space.extend(vectors);
        for exp_vec in vectors {
            let vec = space.find_term(exp_vec.term()).unwrap();
            assert_eq!(vec, exp_vec);
        }
    }
}
