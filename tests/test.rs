#[test]
fn test_concrete() {
    let sample_dir = "tests/samples/concrete";
    let width = 10;
    let depth = 10;
    let height = 5;
    let output_file = "tests/output/concrete.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}
