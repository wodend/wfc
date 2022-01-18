use serde::{Deserialize, Serialize};

use super::tile::Tiles;
use super::wave::Waves;
use super::wave::WavesError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Face {
    Left,
    Right,
    Front,
    Back,
    Down,
    Up,
}

pub struct Model {
    sample_dir: String,
    width: usize,
    depth: usize,
    height: usize,
    output_file: String,
}

impl Model {
    /// Constructs a `Model` for the Wave Function Collapse Algorithm
    pub fn new(
        sample_dir: &str,
        width: usize,
        depth: usize,
        height: usize,
        output_file: &str,
    ) -> Self {
        let model = Self {
            sample_dir: sample_dir.to_string(),
            width: width,
            depth: depth,
            height: height,
            output_file: output_file.to_string(),
        };
        return model;
    }

    pub fn wfc(&self) -> Result<(), WavesError> {
        let mut tiles = Tiles::from(&self.sample_dir).unwrap();
        tiles.generate_transformed_tiles();
        // let tile_set = tiles.tile_set(); // TODO: Add tile set as input to waves
        let mut coordinates = Vec::new();
        let mut graph = Vec::new();
        for z in 0..self.height {
            for y in 0..self.depth {
                for x in 0..self.width {
                    coordinates.push((x, y, z));
                    graph.push(self.edges(x, y, z));
                }
            }
        }
        println!("{:?}", coordinates);
        println!("{:?}", graph);
        println!("center {:?}", graph[self.edge_wave(1, 1, 1)]);
        let mut waves = Waves::new(graph.len(), tiles.len());
        let constraints = tiles.constraints();

        let mut i = 0; // TODO: Remove when done debugging
        while !waves.are_collapsed() {
            //println!("\n\nIteration {}", i);
            //println!("Waves {:?}", waves);
            let wave = waves.min_entropy_wave();
            //println!("Min entropy wave {:?}", wave);
            waves.observe(wave);
            //println!("Observe {:?}", waves);
            waves.propogate(&constraints, &graph, wave)?;
            //println!("Propogate {:?}", waves);
            i += 1;
        }
        //println!("\n\nFinal {:?}", waves);
        tiles.render(waves.tiles(), coordinates, &self.output_file);
        return Ok(());
    }

    fn edges(&self, x: usize, y: usize, z: usize) -> Vec<(usize, Face)> {
        let mut edges = Vec::new();
        if x > 0 {
            let edge_wave = self.edge_wave(x-1, y, z);
            edges.push((edge_wave, Face::Left));
        }
        if x < self.width - 1 {
            let edge_wave = self.edge_wave(x+1, y, z);
            edges.push((edge_wave, Face::Right));
        }
        if y > 0 {
            let edge_wave = self.edge_wave(x, y-1, z);
            edges.push((edge_wave, Face::Front));
        }
        if y < self.depth - 1 {
            let edge_wave = self.edge_wave(x, y+1, z);
            edges.push((edge_wave, Face::Back));
        }
        if z > 0 {
            let edge_wave = self.edge_wave(x, y, z-1);
            edges.push((edge_wave, Face::Down));
        }
        if z < self.height - 1 {
            let edge_wave = self.edge_wave(x, y, z+1);
            edges.push((edge_wave, Face::Up));
        }
        return edges;
    }

    fn edge_wave(&self, x: usize, y: usize, z: usize) -> usize {
        return x + (y * self.width) + (z * self.width * self.depth);
    }
}
