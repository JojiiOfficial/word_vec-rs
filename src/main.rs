pub mod as_vector;
pub mod error;
pub mod export;
pub mod iter;
pub mod parse;
pub mod space;
pub mod vector;

use parse::Word2VecParser;
use space::VecSpace;
use std::time::Instant;
use vector::OwnedVector;

fn main() {
    let start = Instant::now();
    let _en_space = Word2VecParser::new()
        .binary()
        .parse_file("./GoogleNews-vectors-negative300.bin")
        .unwrap();
    println!("loading took: {:?}", start.elapsed());
    loop {}
}

pub fn main2() {
    let en_space = Word2VecParser::new()
        .index_terms(true)
        .parse_file("./enja.para.lang0.vec")
        .unwrap();

    let ja_space = Word2VecParser::new()
        .index_terms(true)
        .parse_file("./enja.para.lang1.vec")
        .unwrap();

    println!("Loaded");
    let mut buf = String::new();
    loop {
        std::io::stdin().read_line(&mut buf).unwrap();
        let txt = buf.trim();
        if txt.is_empty() {
            buf.clear();
            continue;
        }

        print_top_k(&en_space, txt, &ja_space, 10);

        buf.clear();
    }
}

fn print_top_k(src_space: &VecSpace, term: &str, space: &VecSpace, k: usize) {
    let subterms: Vec<_> = term
        .split(' ')
        .filter_map(|i| src_space.find_term(i))
        .collect();

    if subterms.is_empty() {
        println!("Term {term:?} not found");
        return;
    }

    let mut qvec: OwnedVector = borrowme::ToOwned::to_owned(&subterms[0]);
    for i in 1..subterms.len() {
        qvec = qvec + subterms[i];
    }

    let start = Instant::now();
    let top = space.top_k(k, |o| qvec.cosine(o));
    let dur = start.elapsed();

    println!("Top k={k} for {term:?} (in: {dur:?}):");

    for (sim, vec) in top {
        println!("- {} ({})", vec.term(), sim);
    }

    println!();
}
