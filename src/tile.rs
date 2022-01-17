use core::panic;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::path::Path;

use serde::{Serialize, Deserialize};

use super::vox::Vox;
use super::model::Model;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    size: usize,
    tiles: Vec<Tile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Tile {
    name: String,
    connectors: Connectors,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct Connectors {
    left: HorizontalConnector,
    right: HorizontalConnector,
    front: HorizontalConnector,
    back: HorizontalConnector,
    up: VerticalConnector,
    down: VerticalConnector,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
struct HorizontalConnector {
    id: usize,
    symmetry: SymmetryType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
enum SymmetryType {
    Normal = -1,
    Rotated = 0,
    Symmetrical = 1,
} 

impl PartialEq for HorizontalConnector {
    fn eq(&self, other: &Self) -> bool {
        let id_eq = self.id == other.id;
        let symmetry_eq = match self.symmetry {
            SymmetryType::Normal => other.symmetry == SymmetryType::Rotated,
            SymmetryType::Rotated => other.symmetry == SymmetryType::Normal,
            SymmetryType::Symmetrical => other.symmetry == SymmetryType::Symmetrical,
        };
        let eq = id_eq && symmetry_eq;
        return eq;
    }
}

impl Eq for HorizontalConnector {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct VerticalConnector {
    id: usize,
    rotation: usize,
}

pub struct Tiles {
    size: usize,
    values: Vec<Tile>,
}

impl Tile {
    pub fn horizontal_connector(&self, face: usize) -> &HorizontalConnector {
        let connector = match face {
            Model::LEFT => &self.connectors.left,
            Model::RIGHT => &self.connectors.right,
            Model::FRONT => &self.connectors.front,
            Model::BACK => &self.connectors.back,
            _ => unreachable!(),
        };
        return connector;
    }

    pub fn vertical_connector(&self, face: usize) -> &VerticalConnector {
        let connector = match face {
            Model::DOWN => &self.connectors.down,
            Model::UP => &self.connectors.up,
            _ => unreachable!(),
        };
        return connector;
    }
}

/// Generates transformed tiles from the base list given in the config
fn generate_tiles(sample_dir: &str, tiles: Vec<Tile>) -> Vec<Tile> {
    let mut generated_tiles = Vec::new();
    for tile in tiles {
        generated_tiles.push(tile.clone());
        let mut rotated_tiles = rotated_tiles(sample_dir, &tile);
        generated_tiles.append(&mut rotated_tiles);
        // TODO: Implement reflections
        // if graphic.reflect_x {
        //     let image = image.flipv();
        //     push_tiles(&mut tiles, &image, graphic.rotation_count);
        // }
        // if graphic.reflect_y {
        //     let image = image.fliph();
        //     push_tiles(&mut tiles, &image, graphic.rotation_count);
        // }
    }
    return generated_tiles;
}

/// Generates unique rotations
fn rotated_tiles(sample_dir: &str, tile: &Tile) -> Vec<Tile> {
    let mut rotated_tiles = Vec::new();
    let mut previous_connectors = HashSet::from([tile.connectors.clone()]);
    let max_rotation_count = 3;
    let extension = "vox";
    let sample_dir = Path::new(sample_dir);
    let path = sample_dir
        .join(&tile.name)
        .with_extension(extension);
    let file = File::open(path).unwrap();
    let mut vox = Vox::from(file);
    let mut rotated_tile = tile.clone();
    for rotation in 0..max_rotation_count {
        vox.rotate_90_z();
        rotated_tile = rotate_90_z(&tile.name, &rotated_tile, rotation);
        if !previous_connectors.contains(&rotated_tile.connectors) {
            previous_connectors.insert(rotated_tile.connectors.clone());
            rotated_tiles.push(rotated_tile.clone());
            let path = sample_dir
                .join(&rotated_tile.name)
                .with_extension(extension);
            let file = File::create(path).unwrap();
            vox.write(file);
        }
    }
    return rotated_tiles;
}

/// Rotate 90 degrees about the tile's center-point along the z axis
fn rotate_90_z(original_tile_name: &str, tile: &Tile, rotation: usize) -> Tile {
    let rotation_name = match rotation {
        0 => "90",
        1 => "180",
        2 => "270",
        _ => panic!("Unknown rotation: {}", rotation),
    };
    let rotated_tile_name = format!(
        "{tile_name}-r{rotation_name}",
        tile_name=original_tile_name,
        rotation_name=rotation_name,
    );
    let mut up = tile.connectors.up.clone();
    up.rotation += 1;
    let mut down = tile.connectors.down.clone();
    down.rotation += 1;
    let rotated_tile = Tile {
        name: rotated_tile_name,
        connectors: Connectors {
            left: tile.connectors.back.clone(),
            right: tile.connectors.front.clone(),
            front: tile.connectors.left.clone(),
            back: tile.connectors.right.clone(),
            up: up,
            down: down,
        }
    };
    return rotated_tile;
}

impl Tiles {
    pub fn new(sample_dir: &str) -> Self {
        // TODO: Generate tiles.json and constraints.csv and read from them if they exist
        let config_path = Path::new(sample_dir).join("config.json");
        let json = fs::read_to_string(config_path).unwrap();
        let config = serde_json::from_str::<Config>(&json).unwrap();
        let size = config.size;
        let values = generate_tiles(sample_dir, config.tiles);
        let tiles = Self {
            size: size,
            values: values,
        };
        return tiles;
    }

    pub fn constraints(&self) -> Vec<Vec<HashSet<usize>>> {
        let mut constraints = Vec::new();
        for face in Model::HORIZONTAL_FACES {
            let mut face_constraints = Vec::new();
            for constraint_tile in self.values.iter() {
                let mut valid_tile_ids = HashSet::new();
                for (tile_id, tile) in self.values.iter().enumerate() {
                    let inverse_face = match face {
                        Model::LEFT => Model::RIGHT,
                        Model::RIGHT => Model::LEFT,
                        Model::FRONT => Model::BACK,
                        Model::BACK => Model::FRONT,
                        Model::DOWN => Model::UP,
                        Model::UP => Model::DOWN,
                        _ => panic!("Unknown face"),
                    };
                    if constraint_tile.horizontal_connector(face) == tile.horizontal_connector(inverse_face) {
                        valid_tile_ids.insert(tile_id);
                    }
                }
                face_constraints.push(valid_tile_ids);
            }
            constraints.push(face_constraints);
        }
        // TODO: Reduce this duplication
        for face in Model::VERTICAL_FACES {
            let mut face_constraints = Vec::new();
            for constraint_tile in self.values.iter() {
                let mut valid_tile_ids = HashSet::new();
                for (tile_id, tile) in self.values.iter().enumerate() {
                    let inverse_face = match face {
                        Model::LEFT => Model::RIGHT,
                        Model::RIGHT => Model::LEFT,
                        Model::FRONT => Model::BACK,
                        Model::BACK => Model::FRONT,
                        Model::DOWN => Model::UP,
                        Model::UP => Model::DOWN,
                        _ => panic!("Unknown face"),
                    };
                    if constraint_tile.vertical_connector(face) == tile.vertical_connector(inverse_face) {
                        valid_tile_ids.insert(tile_id);
                    }
                }
                face_constraints.push(valid_tile_ids);
            }
            constraints.push(face_constraints);
        }
        return constraints;
    }

    pub fn len(&self) -> usize {
        return self.values.len();
    }

    pub fn size(&self) -> usize {
        return self.size;
    }

    pub fn name(&self, tile: usize) -> &str {
        return &self.values[tile].name;
    }
}
