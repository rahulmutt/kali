use crate::*;

#[test]
fn blob_collects_bytes_and_text() {
    let blob = Blob::new(
        ["hello ".as_bytes(), "world".as_bytes()],
        Some("text/plain".to_string()),
    );
    assert_eq!(blob.size(), 11);
    assert_eq!(blob.mime_type(), Some("text/plain"));
    assert_eq!(blob.bytes(), b"hello world");
    assert_eq!(blob.text().expect("blob text"), "hello world");
}

#[test]
fn file_wraps_blob_metadata() {
    let file = File::new(
        "report.txt",
        ["hello ".as_bytes(), "world".as_bytes()],
        Some("text/plain".to_string()),
        42,
    );
    assert_eq!(file.name(), "report.txt");
    assert_eq!(file.last_modified(), 42);
    assert_eq!(file.size(), 11);
    assert_eq!(file.bytes(), b"hello world");
    assert_eq!(file.blob().mime_type(), Some("text/plain"));
    assert_eq!(file.text().expect("file text"), "hello world");
}

#[test]
fn file_reader_reads_blob_and_file_payloads() {
    let blob = Blob::new(
        ["reader payload".as_bytes()],
        Some("text/plain".to_string()),
    );
    let file = File::new("reader.txt", ["reader payload".as_bytes()], None, 7);

    let mut reader = FileReader::new();
    assert_eq!(reader.ready_state(), FileReaderState::Empty);

    assert_eq!(
        reader.read_as_text(&blob).expect("blob text"),
        "reader payload"
    );
    assert_eq!(reader.ready_state(), FileReaderState::Done);
    assert_eq!(reader.result_bytes(), Some(b"reader payload".as_slice()));

    reader.clear();
    assert_eq!(reader.ready_state(), FileReaderState::Empty);
    assert!(reader.result_bytes().is_none());

    assert_eq!(reader.read_file_as_bytes(&file), b"reader payload");
    assert_eq!(reader.ready_state(), FileReaderState::Done);
    assert_eq!(
        reader.read_file_as_text(&file).expect("file text"),
        "reader payload"
    );
}

#[test]
fn blob_and_file_stream_baselines_preserve_bytes() {
    let blob = Blob::new(
        ["stream ".as_bytes(), "payload".as_bytes()],
        Some("text/plain".to_string()),
    );
    let file = File::new(
        "stream.txt",
        ["stream ".as_bytes(), "payload".as_bytes()],
        None,
        9,
    );

    let blob_stream = blob.stream();
    assert_eq!(blob_stream.chunks(), vec![b"stream payload".to_vec()]);
    assert_eq!(blob_stream.bytes(), b"stream payload");
    assert_eq!(
        blob_stream.text().expect("blob stream text"),
        "stream payload"
    );
    assert!(!blob_stream.is_closed());

    let file_stream = file.stream();
    assert_eq!(file_stream.bytes(), b"stream payload");
    assert_eq!(
        file_stream.text().expect("file stream text"),
        "stream payload"
    );
}

#[test]
fn form_data_records_entries_and_preserves_order() {
    let blob = Blob::new(["form payload".as_bytes()], Some("text/plain".to_string()));
    let file = File::new("form.txt", ["file payload".as_bytes()], None, 13);
    let form = FormData::new();

    form.append("alpha", "1");
    form.append("beta", blob.clone());
    form.append("beta", file.clone());

    assert!(form.has("alpha"));
    assert_eq!(
        form.get("alpha").expect("alpha entry").value(),
        &FormDataValue::Text("1".to_string())
    );
    assert_eq!(form.get_all("beta").len(), 2);
    assert_eq!(
        form.get_all("beta")[0].value(),
        &FormDataValue::Blob(blob.clone())
    );
    assert_eq!(
        form.get_all("beta")[1].value(),
        &FormDataValue::File(file.clone())
    );

    form.set("beta", "replacement");
    assert_eq!(form.get_all("beta").len(), 1);
    assert_eq!(
        form.get("beta").expect("beta entry").value(),
        &FormDataValue::Text("replacement".to_string())
    );

    form.delete("alpha");
    assert!(!form.has("alpha"));
    assert_eq!(form.entries().len(), 1);
}
