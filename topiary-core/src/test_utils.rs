use prettydiff::text::{ContextConfig, diff_lines};

pub fn pretty_assert_eq(v1: &str, v2: &str) {
    if v1 != v2 {
        let diff = diff_lines(v1, v2);
        panic!(
            "\n{}",
            diff.format_with_context(
                Some(ContextConfig {
                    context_size: 2,
                    skipping_marker: "...",
                }),
                true,
            )
        )
    }
}
