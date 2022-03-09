use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::model::Face;
use super::vox::Vox;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    tile_size: usize,
    tile_configs: Vec<TileConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TileConfig {
    name: String,
    connectors: Connectors,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct Connectors {
    left: Connector,
    right: Connector,
    front: Connector,
    back: Connector,
    down: Connector,
    up: Connector,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct Connector {
    id: i64,
    symmetry: Symmetry,
}

impl Connector {
    fn inverse(&self) -> Self {
        let id = self.id;
        let symmetry = match self.symmetry{
            Symmetry::Symmetrical => Symmetry::Symmetrical,
            Symmetry::Normal => Symmetry::Inverse,
            Symmetry::Inverse => Symmetry::Normal,
        };
        return Self {
            id: id,
            symmetry: symmetry,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
enum Symmetry {
    Normal,
    Inverse,
    Symmetrical,
}

impl Connectors {
    /// Returns a new `Connectors` rotated `rotation` degrees about the z axis
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

    /// Returns a new `Connectors` rotated `rotation` degrees about the z axis
    fn reflected(&self, axis: &Axis) -> Self {
        return match axis {
            Axis::X => Self {
                left: self.right.clone().inverse(),
                right: self.left.clone().inverse(),
                front: self.front.clone().inverse(),
                back: self.back.clone().inverse(),
                down: self.down.clone(),
                up: self.up.clone(),
            },
            Axis::Y => Self {
                left: self.left.clone().inverse(),
                right: self.right.clone().inverse(),
                front: self.back.clone().inverse(),
                back: self.front.clone().inverse(),
                down: self.down.clone(),
                up: self.up.clone(),
            },
        };
    }

    /// Returns a reference to the `Connector` for `face`.
    fn get(&self, face: &Face) -> &Connector {
        return match face {
            Face::Left => &self.left,
            Face::Right => &self.right,
            Face::Front => &self.front,
            Face::Back => &self.back,
            Face::Down => &self.down,
            Face::Up => &self.up,
        };
    }
}

#[derive(Debug)]
/// A container for tile data provided in a sample directory.
pub struct Tiles {
    size: usize,
    vox_paths: Vec<PathBuf>,
    rotations: Vec<Rotation>,
    connectors: Vec<Connectors>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A 3D z axis rotation following the right-hand rule.
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A 3D axis for specifying a reflection.
pub enum Axis {
    X,
    Y,
}

impl Tiles {
    /// Returns a new `Tiles` based on the config in `sample_dir`.
    pub fn from(sample_dir: &str) -> std::io::Result<Self> {
        let sample_dir = Path::new(sample_dir).to_path_buf();
        let config_path = sample_dir.join("config.json");
        let config_json = fs::read_to_string(config_path).unwrap();
        let config = serde_json::from_str::<Config>(&config_json).unwrap();
        let mut vox_paths = Vec::new();
        let mut rotations = Vec::new();
        let mut connectors = Vec::new();
        for tile_config in config.tile_configs {
            vox_paths.push(sample_dir.join(tile_config.name).with_extension("vox"));
            rotations.push(Rotation::R0);
            connectors.push(tile_config.connectors);
        }
        let tiles = Self {
            size: config.tile_size,
            vox_paths: vox_paths,
            rotations: rotations,
            connectors: connectors,
        };
        return Ok(tiles);
    }

    /// Generates transformed tiles and vox objects for each config tile.
    pub fn generate_transformed_tiles(&mut self) {
        let mut generated_count = 0;
        let mut generated = Vec::new();
        for (vox_path, connectors) in self.vox_paths.iter().zip(&self.connectors) {
            let vox = Vox::open(vox_path).unwrap();
            let tile_name = vox_path // TODO: Generate tile name if this fails
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .split("-")
                .collect::<Vec<&str>>()[2];
            let vox_extension = vox_path.extension().unwrap();
            let mut visited = HashSet::new();
            visited.insert(connectors.clone());
            let rotations = [Rotation::R90, Rotation::R180, Rotation::R270];
            for generated_rotation in rotations {
                let generated_connectors = connectors.rotated(&generated_rotation);
                if !visited.contains(&generated_connectors) {
                    visited.insert(generated_connectors.clone());
                    let generated_tile_name = format!(
                        "generated-{generated_count}-{tile_name}_{rotation:?}",
                        generated_count = generated_count,
                        tile_name = tile_name,
                        rotation = generated_rotation,
                    );
                    let generated_vox_path = vox_path
                        .with_file_name(generated_tile_name)
                        .with_extension(vox_extension);
                    let generated_vox = vox.rotated(&generated_rotation);
                    generated_vox.write(&generated_vox_path).unwrap();
                    generated_count += 1;
                    generated.push((generated_vox_path, generated_rotation, generated_connectors));
                }
            }
            let axes = [Axis::X, Axis::Y];
            for generated_axis in axes {
                let generated_connectors = connectors.reflected(&generated_axis);
                if !visited.contains(&generated_connectors) {
                    visited.insert(generated_connectors.clone());
                    let generated_tile_name = format!(
                        "generated-{generated_count}-{tile_name}_f{axis:?}",
                        generated_count = generated_count,
                        tile_name = tile_name,
                        axis = generated_axis,
                    );
                    let generated_vox_path = vox_path
                        .with_file_name(generated_tile_name)
                        .with_extension(vox_extension);
                    let generated_vox = vox.reflected(&generated_axis);
                    generated_vox.write(&generated_vox_path).unwrap();
                    generated_count += 1;
                    generated.push((generated_vox_path, Rotation::R0, generated_connectors));
                }
            }
        }

        for (vox_path, rotation, connectors) in generated {
            self.vox_paths.push(vox_path);
            self.rotations.push(rotation);
            self.connectors.push(connectors);
        }
    }

    /// Returns valid tiles for each tile on each face to constrain `wfc`.
    pub fn constraints(&self) -> HashMap<Face, Vec<HashSet<usize>>> {
        let mut constraints = HashMap::new();
        let faces = [
            Face::Left,
            Face::Right,
            Face::Front,
            Face::Back,
            Face::Down,
            Face::Up,
        ];
        for face in faces {
            let mut face_constraints = Vec::new();
            let inverse_face = match face {
                Face::Left => Face::Right,
                Face::Right => Face::Left,
                Face::Front => Face::Back,
                Face::Back => Face::Front,
                Face::Down => Face::Up,
                Face::Up => Face::Down,
            };
            for (constraint_rotation, constraint_connectors) in
                self.rotations.iter().zip(&self.connectors)
            {
                let constraint_connector = constraint_connectors.get(&face);
                let mut valid_tiles = HashSet::new();
                for (tile, (rotation, connectors)) in
                    self.rotations.iter().zip(&self.connectors).enumerate()
                {
                    let connector = connectors.get(&inverse_face);
                    let id_fits = constraint_connector.id == connector.id;
                    let symmetry_fits = match constraint_connector.symmetry {
                        Symmetry::Normal => {
                            if face == Face::Down || face == Face::Up {
                                connector.symmetry == Symmetry::Normal
                                    && constraint_rotation == rotation
                            } else {
                                connector.symmetry == Symmetry::Inverse
                            }
                        }
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
        let cs = constraints.get(&Face::Down).unwrap();
        for (tile, valid_tiles) in cs.iter().enumerate() {
            println!("---\n{:?}:", self.vox_paths[tile]);
            for tile in valid_tiles.iter() {
                println!("{:?}", self.vox_paths[*tile]);
            }
        }
        return constraints;
    }

    pub fn size(&self) -> usize {
        return self.size;
    }

    pub fn vox_paths(&self) -> &Vec<PathBuf> {
        return &self.vox_paths;
    }
}
