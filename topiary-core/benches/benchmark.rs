use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::io;
use topiary_core::{Language, Operation, TopiaryQuery, formatter};

fn setup() -> (String, Language) {
    let input = fs::read_to_string("../topiary-cli/tests/samples/input/nickel.ncl").unwrap();

    // The grammar is loaded dynamically via `topiary-config` rather than
    // depending on the `tree-sitter-nickel` crate directly.
    let config = topiary_config::Configuration::default();
    let grammar = config.get_language("nickel").unwrap().grammar().unwrap();

    let language: Language = Language {
        name: "nickel".to_owned(),
        formatting_query: TopiaryQuery::new(&grammar, topiary_queries::nickel()).unwrap(),
        grammar,
        indent: None,
        injection_query: None,
    };

    (input, language)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("format_nickel", |b| {
        let (input, language) = setup();
        // https://criterion-rs.github.io/book/user_guide/timing_loops.html#iter_with_large_drop
        b.iter_with_large_drop(|| {
            let mut input = input.as_bytes();
            let mut output = io::BufWriter::new(Vec::new());
            formatter(
                &mut input,
                &mut output,
                &language,
                Operation::Format {
                    skip_idempotence: true,
                    tolerate_parsing_errors: false,
                },
                None,
            )
            .unwrap();
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
