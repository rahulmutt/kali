/// An interpolated template literal split into its literal chunks and the raw
/// source of each `${...}` expression. Invariant: `quasis.len() ==
/// expressions.len() + 1` (leading and trailing quasis may be empty).
pub struct TemplateLiteralSegments {
    pub quasis: Vec<String>,
    pub expressions: Vec<String>,
}

/// Splits a backtick-delimited template literal (delimiters included in
/// `text`) into quasis and raw `${...}` expression sources. Returns `None`
/// when `text` is not backtick-delimited or an interpolation has no closing
/// `}`.
pub fn split_template_literal(text: &str) -> Option<TemplateLiteralSegments> {
    let trimmed = text.trim();
    let inner = trimmed
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))?;

    let mut quasis = Vec::new();
    let mut expressions = Vec::new();
    let mut quasi = String::new();
    let mut index = 0usize;
    while index < inner.len() {
        let Some(relative) = inner[index..].find("${") else {
            quasi.push_str(&inner[index..]);
            break;
        };

        let chunk_start = index + relative;
        quasi.push_str(&inner[index..chunk_start]);
        quasis.push(std::mem::take(&mut quasi));

        let expression_start = chunk_start + 2;
        let expression_end = find_template_expression_end(inner, expression_start)?;
        expressions.push(inner[expression_start..expression_end].to_string());
        index = expression_end + 1;
    }
    quasis.push(quasi);

    Some(TemplateLiteralSegments {
        quasis,
        expressions,
    })
}

pub fn resolve_interpolated_template_literal(
    text: &str,
    mut resolve_expression: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    let segments = split_template_literal(text)?;
    let mut rendered = segments.quasis[0].clone();
    for (expression, quasi) in segments.expressions.iter().zip(&segments.quasis[1..]) {
        rendered.push_str(&resolve_expression(expression)?);
        rendered.push_str(quasi);
    }
    Some(rendered)
}

fn find_template_expression_end(text: &str, start: usize) -> Option<usize> {
    let mut depth = 1i32;
    let mut index = start;
    let mut string_delimiter: Option<char> = None;
    let mut escaped = false;

    while index < text.len() {
        let ch = text[index..].chars().next()?;
        let ch_len = ch.len_utf8();

        if let Some(delimiter) = string_delimiter {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                string_delimiter = None;
            }
            index += ch_len;
            continue;
        }

        match ch {
            '\'' | '"' | '`' => {
                string_delimiter = Some(ch);
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }

        index += ch_len;
    }

    None
}

#[cfg(test)]
#[path = "template_tests.rs"]
mod template_tests;
