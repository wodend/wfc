use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::model::Face;
use super::vox::Vox;


// tmp
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    tile_size: usize,
    tile_configs: Vec<TileConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TileConfig {
    name: String,
    connectors: HashMap<Face, Connector>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct Connector {
    id: usize,
    symmetry: Symmetry,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
enum Symmetry {
    Normal,
    Inverse,
    Symmetrical,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Connectors {
    left: Connector,
    right: Connector,
    front: Connector,
    back: Connector,
    down: Connector,
    up: Connector,
}

impl Connectors {
    /// Returns a new connectors rotated `rotation` degrees about the z axis
    fn rotated(&self, rotation: &Rotation) -> Self {
        return match rotation {
            Rotation::R0 => Self {
                left: self.left.clone(),
                right: self.right.clone(),
                front: self.front.clone(),
                back: self.back.clone(),
                down: self.down.clone(),
                up: self.up.clone(),
            },
            Rotation::R90 => Self {
                left: self.back.clone(),
                right: self.front.clone(),
                front: self.left.clone(),
                back: self.right.clone(),
                down: self.down.clone(),
                up: self.up.clone(),
            },
            Rotation::R180 => Self {
                left: self.right.clone(),
                right: self.left.clone(),
                front: self.back.clone(),
                back: self.front.clone(),
                down: self.down.clone(),
                up: self.up.clone(),
            },
            Rotation::R270 => Self {
                left: self.front.clone(),
                right: self.back.clone(),
                front: self.right.clone(),
                back: self.left.clone(),
                down: self.down.clone(),
                up: self.up.clone(),
            },
        };
    }
}

#[derive(Debug)]
pub struct Tiles {
    size: usize,
    vox_paths: Vec<PathBuf>,
    rotations: Vec<Rotation>,
    connectors: HashMap<Face, Vec<Connector>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Tiles {
    pub fn from(sample_dir: &str) -> std::io::Result<Self> {
        let sample_dir = Path::new(sample_dir).to_path_buf();
        let config_path = sample_dir.join("config.json");
        let config_json = fs::read_to_string(config_path).unwrap();
        let config = serde_json::from_str::<Config>(&config_json).unwrap();
        let mut vox_paths = Vec::new();
        let mut rotations = Vec::new();
        let mut connectors = HashMap::from([
            (Face::Left, Vec::new()),
            (Face::Right, Vec::new()),
            (Face::Front, Vec::new()),
            (Face::Back, Vec::new()),
            (Face::Down, Vec::new()),
            (Face::Up, Vec::new()),
        ]);
        for tile_config in config.tile_configs {
            vox_paths.push(sample_dir.join(tile_config.name).with_extension("vox"));
            rotations.push(Rotation::R0);
            connectors
                .get_mut(&Face::Left)
                .unwrap()
                .push(tile_config.connectors.get(&Face::Left).unwrap().clone());
            connectors
                .get_mut(&Face::Right)
                .unwrap()
                .push(tile_config.connectors.get(&Face::Right).unwrap().clone());
            connectors
                .get_mut(&Face::Front)
                .unwrap()
                .push(tile_config.connectors.get(&Face::Front).unwrap().clone());
            connectors
                .get_mut(&Face::Back)
                .unwrap()
                .push(tile_config.connectors.get(&Face::Back).unwrap().clone());
            connectors
                .get_mut(&Face::Down)
                .unwrap()
                .push(tile_config.connectors.get(&Face::Down).unwrap().clone());
            connectors
                .get_mut(&Face::Up)
                .unwrap()
                .push(tile_config.connectors.get(&Face::Up).unwrap().clone());
        }
        let tiles = Self {
            size: config.tile_size,
            vox_paths: vox_paths,
            rotations: rotations,
            connectors: connectors,
        };
        return Ok(tiles);
    }

    pub fn generate_transformed_tiles(&mut self) {
        let mut generated_count = 0;
        let mut generated_vox_paths = Vec::new();
        let mut generated_rotations = Vec::new();
        let mut generated_connectors = Vec::new();
        for (tile, vox_path) in self.vox_paths.iter().enumerate() {
            let config_connectors = self.connectors(tile);
            let mut previous_connectors = HashSet::new();
            previous_connectors.insert(config_connectors.clone());
            let config_vox = Vox::open(vox_path).unwrap();
            let config_tile_name = vox_path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .split("-")
                .collect::<Vec<&str>>()[2];
            let vox_extension = vox_path.extension().unwrap();
            let rotations = [Rotation::R90, Rotation::R180, Rotation::R270];
            for rotation in rotations {
                let connectors = config_connectors.rotated(&rotation);
                if !previous_connectors.contains(&connectors) {
                    previous_connectors.insert(connectors.clone());
                    let generated_tile_name = format!(
                        "generated-{generated_count}-{config_tile_name}_{rotation:?}",
                        generated_count = generated_count,
                        config_tile_name = config_tile_name,
                        rotation = rotation,
                    );
                    let generated_vox_path = vox_path
                        .with_file_name(generated_tile_name)
                        .with_extension(vox_extension);
                    let vox = config_vox.rotated(&rotation);
                    vox.write(&generated_vox_path).unwrap();
                    generated_count += 1;
                    generated_vox_paths.push(generated_vox_path);
                    generated_rotations.push(rotation);
                    generated_connectors.push(connectors.clone());
                }
            }
        }
        for tile in 0..generated_count {
            self.add(
                generated_vox_paths[tile].clone(),
                generated_rotations[tile].clone(),
                generated_connectors[tile].clone(),
            );
        }
    }

    /// Returns connectors for a tile
    fn connectors(&self, tile: usize) -> Connectors {
        return Connectors {
            left: self.connectors.get(&Face::Left).unwrap()[tile].clone(),
            right: self.connectors.get(&Face::Right).unwrap()[tile].clone(),
            front: self.connectors.get(&Face::Front).unwrap()[tile].clone(),
            back: self.connectors.get(&Face::Back).unwrap()[tile].clone(),
            down: self.connectors.get(&Face::Down).unwrap()[tile].clone(),
            up: self.connectors.get(&Face::Up).unwrap()[tile].clone(),
        };
    }

    fn add(&mut self, vox_path: PathBuf, rotation: Rotation, connectors: Connectors) {
        self.vox_paths.push(vox_path);
        self.rotations.push(rotation);
        self.connectors
            .get_mut(&Face::Left)
            .unwrap()
            .push(connectors.left);
        self.connectors
            .get_mut(&Face::Right)
            .unwrap()
            .push(connectors.right);
        self.connectors
            .get_mut(&Face::Front)
            .unwrap()
            .push(connectors.front);
        self.connectors
            .get_mut(&Face::Back)
            .unwrap()
            .push(connectors.back);
        self.connectors
            .get_mut(&Face::Down)
            .unwrap()
            .push(connectors.down);
        self.connectors
            .get_mut(&Face::Up)
            .unwrap()
            .push(connectors.up);
    }

    pub fn len(&self) -> usize {
        return self.vox_paths.len();
    }
    
    pub fn constraints(&self) -> HashMap<Face, Vec<HashSet<usize>>> {
        let mut constraints = HashMap::new();
        let faces = [Face::Left, Face::Right, Face::Front, Face::Back, Face::Down, Face::Up];
        for face in faces {
            let mut face_constraints = Vec::new();
            for (constraint_tile, constraint_connector) in self.connectors.get(&face).unwrap().iter().enumerate() {
                let mut valid_tiles = HashSet::new();
                let inverse_face = match face {
                    Face::Left => Face::Right,
                    Face::Right => Face::Left,
                    Face::Front => Face::Back,
                    Face::Back => Face::Front,
                    Face::Down => Face::Up,
                    Face::Up => Face::Down,
                };
                for (tile, connector) in self.connectors.get(&inverse_face).unwrap().iter().enumerate() {
                    let id_fits = constraint_connector.id == connector.id;
                    let symmetry_fits = match constraint_connector.symmetry {
                        Symmetry::Normal => if face == Face::Down || face == Face::Up {
                                connector.symmetry == Symmetry::Normal && self.rotations[constraint_tile] == self.rotations[tile]
                            } else {
                                connector.symmetry == Symmetry::Inverse
                            },
                        Symmetry::Inverse => connector.symmetry == Symmetry::Normal,
                        Symmetry::Symmetrical => connector.symmetry == Symmetry::Symmetrical,
                    };
                    if id_fits && symmetry_fits {
                        valid_tiles.insert(tile);
                    }
                }
                face_constraints.push(valid_tiles);
            }
            constraints.insert(face, face_constraints);
        }
        // let faces = [Face::Left, Face::Right, Face::Front, Face::Back, Face::Down, Face::Up];
        let faces = [Face::Left];
        for (tile, path) in self.vox_paths.iter().enumerate() {
            println!("\n{:?} constraints", path);
            for face in faces.iter() {
                println!("{:?}", face);
                for tile in constraints[face][tile].iter() {
                    println!("{:?}", self.vox_paths[*tile]);
                }
            }
            println!("\n");
        }
        return constraints;
    }

    // TODO: Move this function to vox module
    pub fn render(&self, tiles: Vec<HashSet<usize>>, coordinates: Vec<(usize, usize, usize)>, output_file: &str) {
        // TODO: Store fq path as tile name so we don't have to take in sample dir
        let file = File::create(output_file).expect("Unable to create vox viewer file");
        let mut writer = BufWriter::new(file);
        writer.write("// Generated wfc output\n".as_bytes());
        let mv_import_size = 10 * self.size; // TODO: Remove this hard-coded value
        let header = format!("mv_import {mv_import_size}\n", mv_import_size=mv_import_size);
        writer.write(header.as_bytes());
        for ((x, y, z), tiles) in coordinates.iter().zip(tiles) {
            let x = x * self.size;
            let y = y * self.size;
            let z = z * self.size;
            for tile in tiles.iter() {
                let path = self.vox_paths[*tile].clone();
                let absolute_path = path.canonicalize().unwrap();
                let absolute_path_str = absolute_path.to_str().unwrap();
                let tile = format!("{x} {y} {z} {path}\n", x=x, y=y, z=z, path=absolute_path_str);
                writer.write(tile.as_bytes());
            }
        }
    }
}
