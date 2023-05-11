use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    str,
};

use crate::{error::Error, space::VecSpace, vector::Vector};

/// Parser for Word2Vec's .vec files.
#[derive(Clone, Copy, Debug)]
pub struct Word2VecParser {
    // File options
    parse_header: bool,
    term_separator: char,
    vec_separator: char,
    binary: bool,

    // Vec space options
    index_terms: bool,
}

impl Word2VecParser {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse from binary format.
    pub fn binary(mut self) -> Self {
        self.binary = true;
        self
    }

    /// Don't treat the first line as header.
    pub fn no_header(mut self) -> Self {
        self.parse_header = false;
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

    pub fn parse<R: Read>(&self, reader: R) -> Result<VecSpace, Error> {
        let mut space = VecSpace::new(0);

        let mut parsed_header = false;
        let mut line_buf = vec![];
        let mut float_buf = vec![];

        let mut r = BufReader::new(reader);

        loop {
            line_buf.clear();

            if !parsed_header {
                if r.read_until(b'\n', &mut line_buf).unwrap() == 0 {
                    return Err(Error::InvalidVectorFormat)?;
                }

                let (_, dim) = self.parse_header(&line_buf)?;
                space = VecSpace::new(dim);
                float_buf.reserve_exact(dim);

                if self.index_terms {
                    space = space.with_termmap();
                }

                parsed_header = true;

                // Don't parse header as vector
                continue;
            }

            // Parse line and insert into space
            let vec = self.parse_vec(&mut r, &mut float_buf, &mut line_buf, space.dim());
            if vec == Err(Error::EOF) {
                break;
            }
            space.insert(&vec?)?;
        }

        Ok(space)
    }

    /// Parses a word vector file.
    #[inline]
    pub fn parse_file<F: AsRef<Path>>(&self, file: F) -> Result<VecSpace, Error> {
        self.parse(File::open(file)?)
    }

    /// Parses a single vec line
    fn parse_vec<'v, 't, R: BufRead>(
        &self,
        r: &mut R,
        vbuf: &'v mut Vec<f32>,
        line_buf: &'t mut Vec<u8>,
        vec_len: usize,
    ) -> Result<Vector<'v, 't>, Error> {
        vbuf.clear();
        line_buf.clear();

        if self.binary {
            self.parse_vec_bin(r, vbuf, line_buf, vec_len)
        } else {
            if r.read_until(b'\n', line_buf)? == 0 {
                return Err(Error::EOF);
            }
            let line = str::from_utf8(line_buf)?;
            self.parse_vec_txt(line, vbuf)
        }
    }

    /// Parses a word vector from txt format.
    fn parse_vec_txt<'v, 't>(
        &self,
        line: &'t str,
        buf: &'v mut Vec<f32>,
    ) -> Result<Vector<'v, 't>, Error> {
        let term_vec_split = line
            .find(self.term_separator)
            .ok_or(Error::InvalidVectorFormat)?;

        for i in line[term_vec_split + 1..line.len() - 1]
            .split(self.vec_separator)
            .map(|i| i.parse::<f32>())
        {
            buf.push(i.map_err(fmt_err)?);
        }

        let term = &line[..term_vec_split];
        Ok(Vector::new(buf, &term))
    }

    /// Parses a word vector from bin format.
    fn parse_vec_bin<'v, 't, R: BufRead>(
        &self,
        r: &mut R,
        vbuf: &'v mut Vec<f32>,
        rbuf: &'t mut Vec<u8>,
        vec_len: usize,
    ) -> Result<Vector<'v, 't>, Error> {
        if r.read_until(b' ', rbuf)? == 0 {
            return Err(Error::EOF);
        }

        let term = str::from_utf8(rbuf)?;

        let mut float_buf = [0u8; 4];
        for _ in 0..vec_len {
            r.read_exact(&mut float_buf)?;
            vbuf.push(f32::from_le_bytes(float_buf.try_into().map_err(fmt_err)?));
        }

        Ok(Vector::new(vbuf, term))
    }

    #[inline]
    fn parse_header(&self, line: &[u8]) -> Result<(usize, usize), Error> {
        if self.binary {
            self.parse_header_bin(line)
        } else {
            let line = str::from_utf8(line)?.trim();
            self.parse_header_txt(line)
        }
    }

    fn parse_header_bin(&self, line: &[u8]) -> Result<(usize, usize), Error> {
        let space = line
            .iter()
            .enumerate()
            .find(|i| *i.1 == b' ')
            .ok_or(Error::InvalidVectorFormat)?
            .0;

        let count = str::from_utf8(&line[..space])?;
        let len = str::from_utf8(&line[space + 1..line.len() - 1])?;

        let count: usize = count.parse().unwrap();
        let len: usize = len.parse().unwrap();

        Ok((count, len))
    }

    fn parse_header_txt(&self, line: &str) -> Result<(usize, usize), Error> {
        let mut split = line.split(' ');
        let mut next_nr = || {
            split
                .next()
                .and_then(|i| i.parse::<usize>().ok())
                .ok_or(Error::InvalidVectorFormat)
        };
        let count = next_nr()?;
        let dim = next_nr()?;
        Ok((count, dim))
    }
}

#[inline]
fn fmt_err<T>(_: T) -> Error {
    Error::InvalidVectorFormat
}

impl Default for Word2VecParser {
    fn default() -> Self {
        Self {
            parse_header: true,
            term_separator: ' ',
            vec_separator: ' ',
            index_terms: false,
            binary: false,
        }
    }
}
