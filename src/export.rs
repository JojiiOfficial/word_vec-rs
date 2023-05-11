use crate::{space::VecSpace, vector::Vector};
use std::io::Write;

pub const DEFAULT_WRITE_HEADER: bool = true;
pub const DEFAULT_TERM_SEP: char = ' ';
pub const DEFAULT_VEC_SEP: char = ' ';

/// Exporter for vectors
#[derive(Debug, Clone, Copy)]
pub struct Exporter<W> {
    // Options
    term_separator: char,
    vec_separator: char,
    binary: bool,

    // Where to write the data to
    writer: W,
    header_written: bool,
}

impl<W> Exporter<W> {
    /// Create a new vector exporter with default configurations and a writer to which the vectors
    /// will be written to.
    #[inline]
    pub fn new(w: W) -> Self {
        Self {
            term_separator: DEFAULT_TERM_SEP,
            vec_separator: DEFAULT_VEC_SEP,
            binary: false,
            writer: w,
            header_written: false,
        }
    }

    /// Exports the data into binary word2vec format.
    pub fn use_binary(mut self) -> Self {
        self.binary = true;
        self
    }
}

impl<W: Write> Exporter<W> {
    /// Exports an entire [`VecSpace`]
    pub fn export_space(self, space: &VecSpace) -> Result<usize, std::io::Error> {
        self.export_space_filtered(space, |_| true)
    }

    /// Exports all vectors from a [`VecSpace`] for which the given filter function returns
    /// `true`
    pub fn export_space_filtered<F>(
        mut self,
        space: &VecSpace,
        filter: F,
    ) -> Result<usize, std::io::Error>
    where
        F: Fn(&Vector) -> bool,
    {
        let mut n = 0;

        let len = space.len();
        let dim = space.dim();
        n += self.write_header(len, dim)?;

        // In txt format, vectors always prepend a '\n' but in binary this is not necessary, so add
        // one after the header as this is needed for binary too.
        if self.binary {
            n += self.writer.write(b"\n")?;
        }

        n += self.export_vectors(space.iter().filter(|i| (filter)(i)))?;

        Ok(n)
    }

    /// Export all given vectors. You have to call `write_header` first.
    ///
    /// # Panics:
    /// Panics if `write_header` is true but none has been written
    pub fn export_vectors<'a, 'b, I>(&mut self, iter: I) -> Result<usize, std::io::Error>
    where
        I: IntoIterator<Item = Vector<'a, 'b>>,
    {
        if !self.header_written {
            panic!("Expecetd header to be written");
        }

        let mut n = 0;

        for i in iter.into_iter() {
            n += self.write_vector(i)?;
        }

        Ok(n)
    }

    /// Exports a given vector
    fn write_vector(&mut self, vec: Vector) -> Result<usize, std::io::Error> {
        if self.binary {
            self.write_vector_bin(vec)
        } else {
            self.write_vector_txt(vec)
        }
    }

    /// Write a single vector in bin format.
    fn write_vector_bin(&mut self, vec: Vector) -> Result<usize, std::io::Error> {
        let mut n = 0;
        n += self.writer.write(vec.term().as_bytes())?;
        n += self.writer.write(&[b' '])?;
        for v in vec.data() {
            self.writer.write(&v.to_le_bytes())?;
        }
        Ok(n)
    }

    /// Write a single vector in txt format.
    fn write_vector_txt(&mut self, vec: Vector) -> Result<usize, std::io::Error> {
        let mut n = 0;
        n += self.writer.write(b"\n")?;
        // Term itself
        n += self.writer.write(vec.term().as_bytes())?;
        // Term separator
        n += self
            .writer
            .write(self.term_separator.to_string().as_bytes())?;

        for (pos, v) in vec.data().iter().enumerate() {
            if pos > 0 {
                n += self
                    .writer
                    .write(self.vec_separator.to_string().as_bytes())?;
            }

            n += self.writer.write(v.to_string().as_bytes())?;
        }

        Ok(n)
    }

    /// Writes the header line.
    fn write_header(&mut self, dim: usize, len: usize) -> Result<usize, std::io::Error> {
        self.header_written = true;
        let mut n = 0;
        n += self.writer.write(dim.to_string().as_bytes())?;
        n += self.writer.write(b" ")?;
        n += self.writer.write(len.to_string().as_bytes())?;
        Ok(n)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::Word2VecParser;
    use std::io::Cursor;

    #[test]
    fn test_txt_export() {
        let vecs = [
            Vector::new(&[1.2, 2.0, 4.4], "term1"),
            Vector::new(&[2.3, 1.0, 3.4], "term3"),
            Vector::new(&[3.1, 9.4, 3.0], "term3"),
        ];
        let mut space = VecSpace::new(3);
        space.extend(vecs);

        let mut buf: Vec<u8> = vec![];

        Exporter::new(&mut buf).export_space(&space).unwrap();

        let parsed = Word2VecParser::new().parse(Cursor::new(&buf)).unwrap();

        assert_eq!(space, parsed);
    }

    #[test]
    fn test_bin_export() {
        let vecs = [
            Vector::new(&[1.2, 2.0, 4.4], "term1"),
            Vector::new(&[2.3, 1.0, 3.4], "term3"),
            Vector::new(&[3.1, 9.4, 3.0], "term3"),
        ];
        let mut space = VecSpace::new(3);
        space.extend(vecs);

        let mut buf: Vec<u8> = vec![];

        Exporter::new(&mut buf)
            .use_binary()
            .export_space(&space)
            .unwrap();

        let parsed = Word2VecParser::new()
            .binary()
            .parse(Cursor::new(&buf))
            .unwrap();

        assert_eq!(space, parsed);
    }
}
