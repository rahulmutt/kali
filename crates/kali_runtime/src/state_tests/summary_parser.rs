use super::*;

#[test]
fn runtime_summary_parser_rejects_whitespace_padded_thread_script_urls() {
    let value = serde_json::json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": " https://e.co/padded.js ",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }
        ]
    });

    assert!(
        parse_thread_runtime_shutdown_report_value(Some(&value)).is_none(),
        "whitespace-padded scriptUrl should be rejected"
    );
}

#[test]
fn runtime_summary_parser_rejects_relative_thread_script_urls() {
    let value = serde_json::json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "worker.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false
            }
        ]
    });

    assert!(
        parse_thread_runtime_shutdown_report_value(Some(&value)).is_none(),
        "relative scriptUrl should be rejected"
    );
}
