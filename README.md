# word_vec-rs
Memory efficient library to work with word vectors

# Example
```rust

let space = Word2VecParser::new()
    // Parse binary file
    .binary()
    // Index terms to find vectors faster.
    .index_terms(true)
    .parse_file("./GoogleNews-vectors-negative300.bin")
    .unwrap();
   
let hello = space.find_term("hello").unwrap();
let hi = space.find_term("hi").unwrap();
println!("{}", hello.cosine(&hi));

```

### Convert file format
```rust
// Load a space
let space = Word2VecParser::new()
    .binary()
    .index_terms(true)
    .parse_file("./GoogleNews-vectors-negative300.bin")
    .unwrap();

// export space to .vec file
let out = BufWriter::new(File::create("GoogleNews-vectors-negative300.vec").unwrap());
Exporter::new(out).export_space(&space).unwrap();

```
