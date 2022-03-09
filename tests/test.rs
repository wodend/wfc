#[test]
fn test_concrete() {
    let sample_dir = "tests/samples/concrete";
    let width = 16;
    let depth = 16;
    let height = 5;
    let output_file = "tests/output/concrete.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}

#[test]
fn test_concrete_2() {
    let sample_dir = "tests/samples/concrete_2";
    let width = 16;
    let depth = 16;
    let height = 16; //16;
    let output_file = "tests/output/concrete_2.txt";
    wfc::debug(sample_dir, width, depth, height, output_file);
    assert!(true);
}

#[test]
fn test_concrete_3() {
    let sample_dir = "tests/samples/concrete_3";
    let width = 32;
    let depth = 32;
    let height = 5;
    let output_file = "tests/output/concrete_3.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}

#[test]
fn test_chaos_fortress() {
    let sample_dir = "tests/samples/chaos_fortress";
    let width = 20;
    let depth = 20;
    let height = 5;
    let output_file = "tests/output/chaos_fortress.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}

#[test]
fn test_stairs() {
    let sample_dir = "tests/samples/stairs";
    let width = 16;
    let depth = 16;
    let height = 8;
    let output_file = "tests/output/stairs.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}

#[test]
fn test_abstract() {
    let sample_dir = "tests/samples/abstract";
    let width = 16;
    let depth = 16;
    let height = 16;
    let output_file = "tests/output/abstract.txt";
    wfc::run(sample_dir, width, depth, height, output_file);
    assert!(true);
}
