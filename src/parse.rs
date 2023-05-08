use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::{space::VecSpace, vector::Vector};

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

    /* /// Parse into an existing [`VecSpace`]
    pub fn parse_into<F: AsRef<Path>>(
        &self,
        file: F,
        space: &mut VecSpace,
    ) -> Result<(), Box<dyn Error>> {
        let reader = BufReader::new(File::open(file)?);
        let mut lines = reader.lines();

        if self.parse_header {
            let header = lines.next().ok_or("No header")??;
            let (_, dim) = self.parse_header(&header)?;
            assert_eq!(dim, space.dim());
        }

        let mut buf = vec![];

        for line in lines {
            let line = line?;
            let vec = self.parse_vec(&line, &mut buf)?;
            space.insert(&vec)?;
        }

        Ok(())
    } */

    pub fn parse<R: Read>(&self, reader: R) -> Result<VecSpace, Box<dyn Error>> {
        let mut space = VecSpace::new(0);

        let mut float_buf = vec![];

        let mut parsed_header = false;
        let mut r = BufReader::new(reader);
        let mut line_buf = vec![];
        loop {
            line_buf.clear();
            if r.read_until(b'\n', &mut line_buf).unwrap() == 0 {
                if !parsed_header {
                    return Err("Nothing to parse".into());
                } else {
                    break;
                }
            }

            if !parsed_header {
                if self.parse_header {
                    let (_, dim) = self.parse_header(&line_buf)?;
                    space = VecSpace::new(dim);
                } else {
                    let first_vec = self.parse_vec(&line_buf, &mut float_buf)?;
                    space = VecSpace::new(first_vec.dim());
                    space.insert(&first_vec)?;
                }

                if self.index_terms {
                    space = space.with_termmap();
                }

                parsed_header = true;

                // Don't parse header as vector
                continue;
            }

            // Parse line and insert into space
            space.insert(&self.parse_vec(&line_buf, &mut float_buf).unwrap())?;
        }

        Ok(space)
    }

    /// Parses a word vector file.
    #[inline]
    pub fn parse_file<F: AsRef<Path>>(&self, file: F) -> Result<VecSpace, Box<dyn Error>> {
        self.parse(File::open(file)?)
    }

    /// Parses a single vec line
    fn parse_vec<'v, 't>(
        &self,
        line: &'t [u8],
        buf: &'v mut Vec<f32>,
    ) -> Result<Vector<'v, 't>, Box<dyn Error>> {
        if self.binary {
            self.parse_vec_bin(line, buf)
        } else {
            let line = std::str::from_utf8(line)?;
            self.parse_vec_txt(line, buf)
        }
    }

    /// Parses a word vector from txt format.
    fn parse_vec_txt<'v, 't>(
        &self,
        line: &'t str,
        buf: &'v mut Vec<f32>,
    ) -> Result<Vector<'v, 't>, Box<dyn Error>> {
        let term_vec_split = line.find(self.term_separator).ok_or("Invalid format")?;

        buf.clear();

        for i in line[term_vec_split + 1..line.len() - 1]
            .split(self.vec_separator)
            .map(|i| i.parse::<f32>())
        {
            buf.push(i?);
        }

        let term = &line[..term_vec_split];
        Ok(Vector::new(buf, &term))
    }

    /// Parses a word vector from bin format.
    fn parse_vec_bin<'v, 't>(
        &self,
        line: &'t [u8],
        _buf: &'v mut Vec<f32>,
    ) -> Result<Vector<'v, 't>, Box<dyn Error>> {
        println!("{line:?}");

        let space = line
            .iter()
            .enumerate()
            .find(|i| *i.1 == b' ')
            .ok_or("Wrong bin format")?
            .0;
        let term = std::str::from_utf8(&line[..space])?;
        print!("{term:?}");

        todo!()
    }

    fn parse_header(&self, line: &[u8]) -> Result<(usize, usize), Box<dyn Error>> {
        if self.binary {
            self.parse_header_bin(line)
        } else {
            let line = std::str::from_utf8(line)?.trim();
            self.parse_header_txt(line)
        }
    }

    fn parse_header_bin(&self, _line: &[u8]) -> Result<(usize, usize), Box<dyn Error>> {
        todo!()
        /* let mut split = line.split(' ');
        let count: usize = split.next().unwrap().parse()?;
        let dim: usize = split.next().unwrap().parse()?;
        Ok((count, dim)) */
    }

    fn parse_header_txt(&self, line: &str) -> Result<(usize, usize), Box<dyn Error>> {
        let mut split = line.split(' ');
        let count: usize = split.next().unwrap().parse()?;
        let dim: usize = split.next().unwrap().parse()?;
        Ok((count, dim))
    }
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
