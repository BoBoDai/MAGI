use crate::MAGI_QUESTION_BUDGET;
use crate::budget::{prepare_context, prepare_question, truncate_context, truncate_head};

#[test]
fn truncates_long_context_keeping_head_and_tail() {
    let (text, truncated) = truncate_context("abcdefghij", 8);

    assert!(truncated);
    assert!(text.starts_with("abcdef"));
    assert!(text.ends_with("ij"));
    assert!(text.contains("中间内容已省略"));
}

#[test]
fn leaves_short_context_untouched() {
    let (text, truncated) = prepare_context("  short  ");
    assert!(!truncated);
    assert_eq!(text, "short");
}

#[test]
fn empty_context_stays_empty() {
    let (text, truncated) = prepare_context("   ");
    assert!(!truncated);
    assert!(text.is_empty());
}

#[test]
fn truncates_peer_output_with_ellipsis() {
    let text = truncate_head("abcdefghij", 4);
    assert_eq!(text, "abcd…");
}

#[test]
fn trims_questions_and_rejects_long_ones() {
    assert_eq!(prepare_question("  hello  ").unwrap(), "hello");
    assert!(prepare_question("   ").is_err());

    let long = "问".repeat(MAGI_QUESTION_BUDGET + 1);
    assert!(prepare_question(&long).is_err());
}
