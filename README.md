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
