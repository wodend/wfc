// #[test]
// fn test_knot() {
//     let sample_dir = "tests/samples/knot";
//     let width = 10;
//     let height = 10;
//     let output_path = "tests/output/knot.png";
//     wfc::run(sample_dir, width, height, output_path);
//     assert!(true);
// }
//
// #[test]
// fn test_stairs() {
//     let sample_dir = "tests/samples/stairs";
//     let width = 10;
//     let height = 10;
//     let output_path = "tests/output/stairs.png";
//     wfc::run(sample_dir, width, height, output_path);
//     assert!(true);
// }

#[test]
fn test_concrete() {
    let sample_dir = "tests/samples/concrete";
    let width = 10;
    let depth = 9;
    let height = 8;
    let output_file = "tests/output/concrete.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}
