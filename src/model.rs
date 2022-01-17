use std::fs::File;
use std::io::{BufWriter,Write};
use std::path::Path;

use super::coordinate::Coordinates;
use super::tile::Tiles;
use super::wave::Waves;
use super::wave::WavesError;

pub struct Model {
    coordinates: Coordinates,
    tiles: Tiles,
    sample_dir: String,
    output_file: String,
}

impl Model {
    pub const FACE_COUNT: usize = 6;
    pub const LEFT: usize = 0;
    pub const RIGHT: usize = 1;
    pub const FRONT: usize = 2;
    pub const BACK: usize = 3;
    pub const DOWN: usize = 4;
    pub const UP: usize = 5;
    pub const HORIZONTAL_FACES: [usize; 4] = [Model::LEFT, Model::RIGHT, Model::FRONT, Model::BACK];
    pub const VERTICAL_FACES: [usize; 2] = [Model::DOWN, Model::UP];

    /// Constructs a `Model` for the Wave Function Collapse Algorithm
    pub fn new(
        sample_dir: &str,
        width: usize,
        depth: usize,
        height: usize,
        output_file: &str,
    ) -> Self {
        let coordinates = Coordinates::new(width, depth, height);
        let tiles = Tiles::new(sample_dir);
        let model = Self {
            coordinates: coordinates,
            tiles: tiles,
            sample_dir: sample_dir.to_string(),
            output_file: output_file.to_string(),
        };
        return model;
    }

    /// Runs the Wave Function Collapse Algorithm
    pub fn wfc(&self) -> Result<(), WavesError> {
        let graph = self.coordinates.graph();
        let constraints = self.tiles.constraints();
        let mut waves = Waves::new(graph.len(), self.tiles.len());

        let mut i = 0; // TODO: Remove when done debugging
        while !waves.are_collapsed() {
            println!("\n\nIteration {}", i);
            println!("Waves {:?}", waves);
            let wave = waves.min_entropy_wave();
            println!("Min entropy wave {:?}", wave);
            waves.observe(wave);
            println!("Observe {:?}", waves);
            waves.propogate(&constraints, &graph, wave)?;
            println!("Propogate {:?}", waves);
            i += 1;
        }
        println!("\n\nFinal {:?}", waves);
        self.render(waves);
        return Ok(());
    }

    // TODO: Move this function to the vox module
    pub fn render(&self, waves: Waves) {
        // TODO: Store fq path as tile name so we don't have to take in sample dir
        let file = File::create(&self.output_file).expect("Unable to create vox viewer file");
        let mut writer = BufWriter::new(file);
        writer.write("// Generated wfc output\n".as_bytes());
        let mv_import_size = self.coordinates.max_dimension_size() * self.tiles.size();
        let header = format!("mv_import {mv_import_size}\n", mv_import_size=mv_import_size);
        writer.write(header.as_bytes());
        for ((x, y, z), tiles) in self.coordinates.xyzs().iter().zip(waves.tiles()) {
            let x = x * self.tiles.size();
            let y = y * self.tiles.size();
            let z = z * self.tiles.size();
            for tile in tiles.iter() {
                let path = Path::new(&self.sample_dir)
                    .join(self.tiles.name(*tile))
                    .with_extension("vox");
                let absolute_path = path.canonicalize().unwrap();
                let absolute_path_str = absolute_path.to_str().unwrap();
                let tile = format!("{x} {y} {z} {path}\n", x=x, y=y, z=z, path=absolute_path_str);
                writer.write(tile.as_bytes());
            }
        }
    }
}
