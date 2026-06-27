use std::path::Path;

use crate::AccessRule;

impl AccessRule {
    pub fn is_enabled(&self) -> bool {
        match self {
            AccessRule::Deny(false) => false,
            AccessRule::Deny(true) => true,
            AccessRule::AllowList(entries) => !entries.is_empty(),
        }
    }

    pub(crate) fn allows_path(&self, candidate: &Path, base_dir: &Path) -> bool {
        self.allows_candidate(&candidate.to_string_lossy(), base_dir, PatternKind::Path)
    }

    pub(crate) fn allows_candidate(
        &self,
        candidate: &str,
        base_dir: &Path,
        kind: PatternKind,
    ) -> bool {
        match self {
            AccessRule::Deny(false) => false,
            AccessRule::Deny(true) => true,
            AccessRule::AllowList(patterns) => {
                if patterns.is_empty() {
                    return false;
                }

                let candidate = normalize_text(candidate);
                patterns.iter().any(|pattern| {
                    let resolved = resolve_pattern(pattern, base_dir, kind);
                    glob_match(&resolved, &candidate)
                })
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum PatternKind {
    Path,
    Url,
    Exact,
}

fn resolve_pattern(pattern: &str, base_dir: &Path, kind: PatternKind) -> String {
    match kind {
        PatternKind::Exact => normalize_text(pattern),
        PatternKind::Url => normalize_text(pattern),
        PatternKind::Path => {
            let candidate = Path::new(pattern);
            let resolved = if candidate.is_absolute() || pattern.contains("://") {
                candidate.to_path_buf()
            } else {
                base_dir.join(candidate)
            };
            normalize_text(&resolved.to_string_lossy())
        }
    }
}

fn normalize_text(text: &str) -> String {
    text.replace('\\', "/")
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern = normalize_text(pattern);
    let text = normalize_text(text);
    glob_match_inner(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_inner(pattern: &[u8], text: &[u8]) -> bool {
    let mut pi = 0usize;
    let mut ti = 0usize;
    let mut star: Option<(usize, usize, bool)> = None;

    while ti < text.len() {
        if pi < pattern.len() {
            match pattern[pi] {
                b'*' => {
                    let is_double = pi + 1 < pattern.len() && pattern[pi + 1] == b'*';
                    let next_pi = if is_double { pi + 2 } else { pi + 1 };
                    star = Some((next_pi, ti, is_double));
                    pi = next_pi;
                    continue;
                }
                ch if ch == text[ti] => {
                    pi += 1;
                    ti += 1;
                    continue;
                }
                _ => {}
            }
        }

        if let Some((next_pi, star_text, is_double)) = star {
            if !is_double && text[star_text] == b'/' {
                return false;
            }
            if star_text < text.len() {
                star = Some((next_pi, star_text + 1, is_double));
                ti = star_text + 1;
                pi = next_pi;
                continue;
            }
        }

        return false;
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        if pi + 1 < pattern.len() && pattern[pi + 1] == b'*' {
            pi += 2;
        } else {
            pi += 1;
        }
    }

    pi == pattern.len()
}
