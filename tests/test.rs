#[test]
fn test_knot() {
    let sample_dir= "tests/samples/knot";
    let width = 10;
    let height = 10;
    let output_path = "tests/output/knot.png";
    wfc::run(sample_dir, width, height, output_path);
    assert!(true);
}