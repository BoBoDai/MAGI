//! Character budgets shared by the three stages.

use magi_core::{Error, Result};

use crate::{MAGI_CONTEXT_BUDGET, MAGI_QUESTION_BUDGET};

pub(crate) fn char_count(text: &str) -> usize {
    text.chars().count()
}

/// Trim and validate the user's question. Over-budget questions are rejected,
/// not silently truncated.
pub(crate) fn prepare_question(question: &str) -> Result<String> {
    let question = question.trim().to_string();
    if question.is_empty() {
        return Err(Error::InvalidInput("问题不能为空".into()));
    }
    if char_count(&question) > MAGI_QUESTION_BUDGET {
        return Err(Error::InvalidInput(format!(
            "问题过长（最多 {MAGI_QUESTION_BUDGET} 字）"
        )));
    }
    Ok(question)
}

/// Normalize the optional context, keeping the head and tail when it is too
/// long. Returns whether truncation happened.
pub(crate) fn prepare_context(context: &str) -> (String, bool) {
    let context = context.trim();
    if context.is_empty() {
        return (String::new(), false);
    }
    truncate_context(context, MAGI_CONTEXT_BUDGET)
}

pub(crate) fn truncate_context(context: &str, budget: usize) -> (String, bool) {
    let length = char_count(context);
    if length <= budget {
        return (context.to_string(), false);
    }

    let head: String = context.chars().take(budget * 3 / 4).collect();
    let tail: String = context
        .chars()
        .rev()
        .take(budget / 4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    (
        format!("{head}\n\n[…上下文过长，中间内容已省略…]\n\n{tail}"),
        true,
    )
}

/// Keep the head of a peer output within the prompt budget.
pub(crate) fn truncate_head(text: &str, budget: usize) -> String {
    if char_count(text) <= budget {
        return text.to_string();
    }

    let mut truncated: String = text.chars().take(budget).collect();
    truncated.push('…');
    truncated
}
