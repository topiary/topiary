use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::{io, sync::Arc};
use topiary_config::Configuration;
use topiary_core::{
    InjectionQuery, Language, LanguageResolver, Operation, TopiaryQuery, formatter_str,
};

const OCAMLLEX_FORMATTING_QUERY: &str =
    include_str!("../../topiary-queries/queries/ocamllex/formatting.scm");
const OCAMLLEX_INJECTION_QUERY: &str =
    include_str!("../../topiary-queries/queries/ocamllex/injections.scm");
const OCAML_FORMATTING_QUERY: &str =
    include_str!("../../topiary-queries/queries/ocaml/formatting.scm");

fn language_from_config(
    config: &Configuration,
    name: &str,
    formatting_query_content: &str,
    injection_query_content: Option<&str>,
) -> Language {
    let config_language = config.get_language(name).unwrap();
    let grammar = config_language.grammar().unwrap();

    Language {
        name: name.to_owned(),
        formatting_query: TopiaryQuery::new(&grammar, formatting_query_content).unwrap(),
        injection_query: injection_query_content
            .map(|query_content| InjectionQuery::new(&grammar, query_content).unwrap()),
        grammar,
        indent: config_language.indent(),
    }
}

fn input_with_actions(count: usize) -> String {
    let actions = (0..count)
        .map(|i| {
            format!(r#"  | "token{i}" {{ let values=[1;2;3] in List.map (fun x->x+{i}) values }}"#)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"{{
  type token = Value of int list | Eof
}}

rule token = parse
{actions}
  | eof {{ Eof }}
"#
    )
}

fn format_ocamllex(input: &str, language: &Language, resolve: Option<&LanguageResolver<'_>>) {
    let mut output = io::BufWriter::new(Vec::new());
    formatter_str(
        input,
        &mut output,
        language,
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: false,
        },
        resolve,
    )
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let config = Configuration::default();
    let baseline = language_from_config(&config, "ocamllex", OCAMLLEX_FORMATTING_QUERY, None);
    let injected = language_from_config(
        &config,
        "ocamllex",
        OCAMLLEX_FORMATTING_QUERY,
        Some(OCAMLLEX_INJECTION_QUERY),
    );
    let ocaml = Arc::new(language_from_config(
        &config,
        "ocaml",
        OCAML_FORMATTING_QUERY,
        None,
    ));

    let mut group = c.benchmark_group("injections_ocamllex");

    for count in [1, 10, 100] {
        let input = input_with_actions(count);

        group.bench_with_input(BenchmarkId::new("baseline", count), &input, |b, input| {
            b.iter(|| format_ocamllex(input, &baseline, None));
        });

        group.bench_with_input(BenchmarkId::new("injected", count), &input, |b, input| {
            b.iter(|| {
                format_ocamllex(
                    input,
                    &injected,
                    Some(&|name| Ok((name == "ocaml").then_some(ocaml.clone()))),
                )
            });
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
