use super::*;

#[test]
fn a_plain_text_is_returned_borrowed() {
    let text = "binding `x` in `f` is used as both a string and a number";
    assert!(matches!(
        display_names_in(text),
        std::borrow::Cow::Borrowed(_)
    ));
}

#[test]
fn every_rename_suffix_is_stripped() {
    assert_eq!(
        display_names_in("binding `x{b3}` in `f{b12}` and `y{b0}`"),
        "binding `x` in `f` and `y`"
    );
}

#[test]
fn a_monomorphized_clone_keeps_its_own_suffix() {
    assert_eq!(display_names_in("`h{b1}${0}`"), "`h${0}`");
}

#[test]
fn braces_that_are_not_a_rename_suffix_are_kept() {
    assert_eq!(
        display_names_in("{b} {b1} x{bb1} x{b1x}"),
        "{b} {b1} x{bb1} x{b1x}"
    );
}
