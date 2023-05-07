use parse::Word2VecParser;
use space::VecSpace;
use vector::OwnedVector;

pub mod as_vector;
pub mod parse;
pub mod space;
pub mod vector;

fn main() {
    let en_space = Word2VecParser::new()
        .index_terms(true)
        .parse("./en.vec")
        .unwrap();

    let ja_space = Word2VecParser::new()
        .index_terms(true)
        .parse("./jp.vec")
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

        print_top_k(&ja_space, txt, &en_space, 10);

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

    println!("Top k={k} for {term:?}:");
    let top = space.top_k(&&qvec, k);
    for (sim, vec) in top {
        println!("- {} ({})", vec.term(), sim);
    }

    println!();
}
