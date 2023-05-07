use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{space::VecSpace, vector::OwnedVector};

/// Parser for Word2Vec's .vec files.
#[derive(Clone, Copy, Debug)]
pub struct Word2VecParser {
    // File options
    has_header: bool,
    term_separator: char,
    vec_separator: char,

    // Vec space options
    index_terms: bool,
}

impl Word2VecParser {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn no_header(mut self) -> Self {
        self.has_header = false;
        self
    }

    /// Use a custom term<->Vec separator character.
    pub fn cust_term_separator(mut self, sep: char) -> Self {
        self.term_separator = sep;
        self
    }

    /// Use a custom Vec item <->Vec item separator character.
    pub fn cust_vec_separator(mut self, sep: char) -> Self {
        self.vec_separator = sep;
        self
    }

    /// Whether to index the words for faster term->vec lookup.
    pub fn index_terms(mut self, index: bool) -> Self {
        self.index_terms = index;
        self
    }

    pub fn parse_into<F: AsRef<Path>>(
        &self,
        file: F,
        space: &mut VecSpace,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reader = BufReader::new(File::open(file)?);
        let mut lines = reader.lines();
        if self.has_header {
            lines.next();
        }

        for line in lines {
            let line = line?;
            let vec = self.parse_vec(&line)?;
            space.insert(&vec)?;
        }

        Ok(())
    }

    pub fn parse<F: AsRef<Path>>(&self, file: F) -> Result<VecSpace, Box<dyn std::error::Error>> {
        let reader = BufReader::new(File::open(file)?);
        let mut lines = reader.lines();

        let mut space;

        if self.has_header {
            let header = lines.next().ok_or_else(|| "No header")??;
            let (_, dim) = Self::parse_header(&header)?;
            space = VecSpace::new(dim);
        } else {
            let line = lines.next().unwrap()?;
            let first_vec = self.parse_vec(&line)?;
            space = VecSpace::new(first_vec.dim());
            space.insert(&first_vec)?;
        }

        if self.index_terms {
            space = space.with_termmap();
        }

        for line in lines {
            let line = line?;
            let vec = self.parse_vec(&line)?;
            space.insert(&vec)?;
        }

        Ok(space)
    }

    /// Parses a single vec line
    fn parse_vec(&self, line: &str) -> Result<OwnedVector, Box<dyn std::error::Error>> {
        let term_vec_split = line
            .find(self.term_separator)
            .ok_or_else(|| "Invalid format")?;

        let term = &line[..term_vec_split];

        let vec_items = line[term_vec_split + 1..]
            .split(self.vec_separator)
            .map(|i| i.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(OwnedVector::new(&vec_items, term))
    }

    fn parse_header(line: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let mut split = line.split(' ');
        let count: usize = split.next().unwrap().parse()?;
        let dim: usize = split.next().unwrap().parse()?;
        Ok((count, dim))
    }
}

impl Default for Word2VecParser {
    fn default() -> Self {
        Self {
            has_header: true,
            term_separator: ' ',
            vec_separator: ' ',
            index_terms: false,
        }
    }
}
