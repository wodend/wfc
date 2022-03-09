use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::tile::Tiles;
use super::wave::Waves;
use super::wave::Contradiction;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
/// A face of a 3D tile.
pub enum Face {
    Left,
    Right,
    Front,
    Back,
    Down,
    Up,
}

/// A container for the necesary data to run `wfc`.
pub struct Model {
    sample_dir: String,
    width: usize,
    depth: usize,
    height: usize,
    output_file: String,
}

impl Model {
    /// Constructs a `Model` for the Wave Function Collapse Algorithm.
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

    /// Runs the Wave Function Collapse Algorithm.
    pub fn debug(&self) -> Result<(), ()> {
        let mut coordinates = Vec::new();
        let mut wave_graph = Vec::new();
        for z in 0..self.height {
            for y in 0..self.depth {
                for x in 0..self.width {
                    coordinates.push((x, y, z));
                    wave_graph.push(self.wave_edges(x, y, z));
                }
            }
        }
        let mut tiles = Tiles::from(&self.sample_dir).unwrap();
        tiles.generate_transformed_tiles();
        let constraints = tiles.constraints();
        let mut waves = Waves::new(&wave_graph, &constraints);

        while !waves.are_collapsed() {
            //println!("\n\nIteration {}", i);
            //println!("Waves {:?}", waves);
            let wave = waves.min_entropy_wave();
            //println!("Min entropy wave {:?}", wave);
            waves.observe(wave);
            //println!("Observe {:?}", waves);
            match waves.propogate(wave) {
                Ok(_) => (),
                Err(c) => {
                    let vox_paths = tiles.vox_paths();
                    println!("Contradiction!");
                    let coords = coordinates[c.wave];
                    println!("Cannot propogate contraints from x={} y={} z={}", coords.0, coords.1, coords.2);
                    println!("Given tiles:");
                    for tile in c.tiles {
                        println!("{:?}", vox_paths[tile]);
                    }
                    println!("On face {:?}", c.face);
                    return Err(());
                },
            };
        }
        //println!("\n\nFinal {:?}", waves);
        self.render(tiles.size(), tiles.vox_paths(), coordinates, waves.tiles());
        return Ok(());
    }

    /// Runs the Wave Function Collapse Algorithm.
    pub fn wfc(&self) -> Result<(), Contradiction> {
        let mut coordinates = Vec::new();
        let mut wave_graph = Vec::new();
        for z in 0..self.height {
            for y in 0..self.depth {
                for x in 0..self.width {
                    coordinates.push((x, y, z));
                    wave_graph.push(self.wave_edges(x, y, z));
                }
            }
        }
        let mut tiles = Tiles::from(&self.sample_dir).unwrap();
        tiles.generate_transformed_tiles();
        let constraints = tiles.constraints();
        let mut waves = Waves::new(&wave_graph, &constraints);

        while !waves.are_collapsed() {
            //println!("\n\nIteration {}", i);
            //println!("Waves {:?}", waves);
            let wave = waves.min_entropy_wave();
            //println!("Min entropy wave {:?}", wave);
            waves.observe(wave);
            //println!("Observe {:?}", waves);
            waves.propogate(wave)?;
            //println!("Propogate {:?}", waves);
        }
        //println!("\n\nFinal {:?}", waves);
        self.render(tiles.size(), tiles.vox_paths(), coordinates, waves.tiles());
        return Ok(());
    }

    /// Returns valid wave edges for an coordinate in the wave graph.
    fn wave_edges(&self, x: usize, y: usize, z: usize) -> Vec<(usize, Face)> {
        let mut edges = Vec::new();
        if x > 0 {
            let edge_wave = self.edge_wave(x - 1, y, z);
            edges.push((edge_wave, Face::Left));
        }
        if x < self.width - 1 {
            let edge_wave = self.edge_wave(x + 1, y, z);
            edges.push((edge_wave, Face::Right));
        }
        if y > 0 {
            let edge_wave = self.edge_wave(x, y - 1, z);
            edges.push((edge_wave, Face::Front));
        }
        if y < self.depth - 1 {
            let edge_wave = self.edge_wave(x, y + 1, z);
            edges.push((edge_wave, Face::Back));
        }
        if z > 0 {
            let edge_wave = self.edge_wave(x, y, z - 1);
            edges.push((edge_wave, Face::Down));
        }
        if z < self.height - 1 {
            let edge_wave = self.edge_wave(x, y, z + 1);
            edges.push((edge_wave, Face::Up));
        }
        return edges;
    }

    /// Returns the wave for a given coordinate.
    fn edge_wave(&self, x: usize, y: usize, z: usize) -> usize {
        return x + (y * self.width) + (z * self.width * self.depth);
    }

    /// Write a MagicaVoxel Viewer mv_import file to render the final waves.
    pub fn render(
        &self,
        tile_size: usize,
        vox_paths: &Vec<PathBuf>,
        coordinates: Vec<(usize, usize, usize)>,
        tiles: &Vec<HashSet<usize>>,
    ) {
        let file = File::create(&self.output_file).expect("Unable to create vox viewer file");
        let mut writer = BufWriter::new(file);
        writer
            .write("// Generated wfc output\n".as_bytes())
            .unwrap();
        let max_dimension_size = std::cmp::max(self.width, std::cmp::max(self.depth, self.height));
        let mv_import_size = max_dimension_size * tile_size;
        let header = format!(
            "mv_import {mv_import_size}\n",
            mv_import_size = mv_import_size
        );
        writer.write(header.as_bytes()).unwrap();
        for ((x, y, z), tiles) in coordinates.iter().zip(tiles) {
            let x = x * tile_size;
            let y = y * tile_size;
            let z = z * tile_size;
            for tile in tiles.iter() {
                let path = vox_paths[*tile].clone();
                let absolute_path = path.canonicalize().unwrap();
                let absolute_path_str = absolute_path.to_str().unwrap();
                let tile = format!(
                    "{x} {y} {z} {path}\n",
                    x = x,
                    y = y,
                    z = z,
                    path = absolute_path_str
                );
                writer.write(tile.as_bytes()).unwrap();
            }
        }
    }
}
