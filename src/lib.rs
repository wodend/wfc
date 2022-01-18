mod model;
mod tile;
mod vox;
mod wave;

use model::Model;

pub fn run(sample_dir: &str, width: usize, depth: usize, height: usize, output_file: &str) {
    let model = Model::new(sample_dir, width, depth, height, output_file);
    loop {
        let wfc = model.wfc();
        match wfc {
            Ok(()) => {
                println!("Wave function collapse completed successfully, exiting");
                break;
            }
            Err(e) => {
                println!("Wave function collapse failed due to {:?}, retrying", e);
            }
        }
    }
}
