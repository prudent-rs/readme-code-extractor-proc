#[test]
fn test_load_file() {
    // rust-analyzer WILL show an error here. Ignore.
    let file_content = readme_code_extractor_proc::test_load_file!("file_1.txt");
    assert_eq!(file_content, "Hi from file_1.txt");
}
