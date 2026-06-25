use crate::*;

#[test]
fn url_search_params_round_trips_values_and_serializes_deterministically() {
    let params = URLSearchParams::new();
    params.append("alpha", "1");
    params.append("beta", "two words");
    params.append("beta", "3");

    assert!(params.has("alpha"));
    assert_eq!(params.get("alpha").as_deref(), Some("1"));
    assert_eq!(
        params.get_all("beta"),
        vec!["two words".to_string(), "3".to_string()]
    );

    params.set("beta", "replacement");
    assert_eq!(params.get_all("beta"), vec!["replacement".to_string()]);
    params.delete("alpha");
    assert!(!params.has("alpha"));
    assert_eq!(
        params.entries(),
        vec![("beta".to_string(), "replacement".to_string())]
    );
    assert_eq!(params.to_string(), "beta=replacement");

    let parsed = URLSearchParams::from_query("alpha=1&beta=two+words&beta=3");
    assert_eq!(parsed.get("alpha").as_deref(), Some("1"));
    assert_eq!(
        parsed.get_all("beta"),
        vec!["two words".to_string(), "3".to_string()]
    );
}

#[test]
fn url_parser_can_parse_and_resolve() {
    let parsed = parse_url("https://example.com/path").expect("url");
    assert_eq!(parsed.as_str(), "https://example.com/path");

    let resolved = resolve_url("https://example.com/base/", "../child").expect("resolved");
    assert_eq!(resolved.as_str(), "https://example.com/child");
}

#[test]
fn url_object_round_trips_components() {
    let mut url = URL::new("https://example.com/base?alpha=1#fragment").expect("url");
    assert_eq!(url.as_str(), "https://example.com/base?alpha=1#fragment");
    assert_eq!(url.href(), "https://example.com/base?alpha=1#fragment");
    assert_eq!(url.protocol(), "https:");
    assert_eq!(url.pathname(), "/base");
    assert_eq!(url.search(), "?alpha=1");
    assert_eq!(url.hash(), "#fragment");
    assert_eq!(url.host(), Some("example.com"));

    url.set_pathname("/child");
    url.set_search("?beta=2");
    url.set_hash("section");
    assert_eq!(url.as_str(), "https://example.com/child?beta=2#section");
    assert_eq!(url.port(), None);
    assert_eq!(url.set_protocol("http:"), Ok(()));
    assert_eq!(url.as_str(), "http://example.com/child?beta=2#section");
    assert_eq!(url.set_host("example.org"), Ok(()));
    assert_eq!(url.set_port(Some(8080)), Ok(()));
    assert_eq!(url.as_str(), "http://example.org:8080/child?beta=2#section");

    let resolved = URL::resolve("https://example.com/base/", "../child").expect("resolved");
    assert_eq!(resolved.as_str(), "https://example.com/child");
    assert_eq!(
        resolved.clone().into_inner().as_str(),
        "https://example.com/child"
    );
}
