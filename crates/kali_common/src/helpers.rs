pub(crate) fn join_semicolon_terminated_segments(segments: &[&str]) -> String {
    let mut source = segments.join("; ");
    source.push(';');
    source
}

pub(crate) fn join_zero_probe_aliases(aliases: &[&'static str]) -> String {
    join_semicolon_terminated_segments(aliases)
}

pub(crate) fn join_const_binding_lines(bindings: &[(&'static str, &'static str)]) -> String {
    let lines = bindings
        .iter()
        .map(|(name, alias)| format!("const {name} = {alias}"))
        .collect::<Vec<_>>();
    let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    join_semicolon_terminated_segments(&line_refs)
}

pub(crate) fn ordered_unique_union(slices: &[&[&'static str]]) -> Vec<&'static str> {
    let total_len = slices.iter().map(|slice| slice.len()).sum();
    let mut aliases = Vec::with_capacity(total_len);
    let mut seen = std::collections::HashSet::with_capacity(total_len);

    for alias in slices.iter().flat_map(|slice| slice.iter().copied()) {
        if seen.insert(alias) {
            aliases.push(alias);
        }
    }

    aliases
}
