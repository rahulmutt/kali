pub fn resolve_interpolated_template_literal(
    text: &str,
    mut resolve_expression: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    let trimmed = text.trim();
    let Some(inner) = trimmed
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
    else {
        return None;
    };

    if !inner.contains("${") {
        return Some(inner.to_string());
    }

    let mut rendered = String::new();
    let mut index = 0usize;
    while index < inner.len() {
        let Some(relative) = inner[index..].find("${") else {
            rendered.push_str(&inner[index..]);
            break;
        };

        let chunk_start = index + relative;
        rendered.push_str(&inner[index..chunk_start]);

        let expression_start = chunk_start + 2;
        let Some(expression_end) = find_template_expression_end(inner, expression_start) else {
            return None;
        };

        let expression = &inner[expression_start..expression_end];
        rendered.push_str(&resolve_expression(expression)?);
        index = expression_end + 1;
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
