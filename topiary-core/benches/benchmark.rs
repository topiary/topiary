use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::io;
use topiary_core::{Language, Operation, TopiaryQuery, formatter};

fn setup() -> (String, Language) {
    let input = fs::read_to_string("../topiary-cli/tests/samples/input/nickel.ncl").unwrap();
    let query_content = fs::read_to_string(format!(
        "../topiary-queries/queries/nickel/{}",
        topiary_queries::FORMATTING_QUERY
    ))
    .unwrap();
    let nickel = tree_sitter_nickel::LANGUAGE;

    let language: Language = Language {
        name: "nickel".to_owned(),
        query: TopiaryQuery::new(&nickel.into(), &query_content).unwrap(),
        grammar: tree_sitter_nickel::LANGUAGE.into(),
        indent: None,
    };

    (input, language)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("format_nickel", |b| {
        let (input, language) = setup();
        b.iter(|| {
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
            )
            .unwrap()
        });
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
