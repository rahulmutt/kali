use crate::*;

#[test]
fn headers_request_and_response_round_trip_deterministically() {
    let headers = Headers::new();
    headers.append("Content-Type", "text/plain");
    headers.append("x-request-id", "alpha");
    headers.append("X-Request-Id", "beta");
    headers.set("Accept", "application/json");

    assert!(headers.has("content-type"));
    assert_eq!(headers.get("CONTENT-TYPE").as_deref(), Some("text/plain"));
    assert_eq!(headers.get("x-request-id").as_deref(), Some("alpha"));
    assert_eq!(
        headers.entries(),
        vec![
            ("content-type".to_string(), "text/plain".to_string()),
            ("x-request-id".to_string(), "alpha".to_string()),
            ("x-request-id".to_string(), "beta".to_string()),
            ("accept".to_string(), "application/json".to_string()),
        ]
    );

    let request = Request::with_parts(
        "https://example.com/api",
        "post",
        headers.clone(),
        "payload",
    )
    .expect("request");
    assert_eq!(request.method(), "POST");
    assert_eq!(request.url().as_str(), "https://example.com/api");
    assert_eq!(request.text().expect("request text"), "payload");
    assert_eq!(
        request.headers().get("accept").as_deref(),
        Some("application/json")
    );

    let response = Response::from_request(&request);
    assert_eq!(response.status(), 200);
    assert_eq!(response.status_text(), "OK");
    assert!(response.ok());
    assert_eq!(response.url().as_str(), "https://example.com/api");
    assert_eq!(response.text().expect("response text"), "payload");
    assert_eq!(
        response.headers().get("x-request-id").as_deref(),
        Some("alpha")
    );

    let echoed = fetch(&request);
    assert_eq!(echoed.text().expect("echo text"), "payload");
    assert_eq!(echoed.status(), 200);
}
