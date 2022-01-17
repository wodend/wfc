// use std::collections::{HashSet, HashMap};
// use std::hash::Hash;
// use std::fs;
// use std::fs::File;
// use std::path::Path;
// use std::io::BufWriter;
// use std::io::Write;
// use std::cmp::max;
// 
// use serde::{Serialize, Deserialize};
// 
// use super::vox::Vox;
// 
// // TODO: Make private and add methods to get these
// #[derive(Debug, Serialize, Deserialize)]
// pub struct Graphics {
//     pub size: usize,
//     pub tiles: Vec<Tile>,
// }
// 
// // TODO: Make private and add methods to get these
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Tile {
//     pub name: String,
//     pub connectors: Connectors,
// }
// 
// 
// // TODO: Make private and add methods to get these
// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
// pub struct Connectors {
//     pub left: Connector,
//     pub right: Connector,
//     pub front: Connector,
//     pub back: Connector,
//     pub up: Connector,
//     pub down: Connector,
// }
// 
// #[derive(Debug, Serialize, Deserialize, Clone, Hash)]
// pub struct Connector {
//     id: usize,
//     symmetry: SymmetryType,
//     rotation: Option<usize>,
// }
// 
// /// Z is the only rotation axis
// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
// enum SymmetryType {
//     Normal = 0,
//     Rotated = 1,
//     Symmetrical = 2,
// } 
// 
// impl PartialEq for Connector {
//     fn eq(&self, other: &Self) -> bool {
//         let id_eq = self.id == other.id;
//         let symmetry_eq = match self.symmetry {
//             SymmetryType::Normal => other.symmetry == SymmetryType::Rotated,
//             SymmetryType::Rotated => other.symmetry == SymmetryType::Normal,
//             SymmetryType::Symmetrical => other.symmetry == SymmetryType::Symmetrical,
//         };
//         let rotation_eq = self.rotation == other.rotation;
//         let eq = id_eq && symmetry_eq && rotation_eq;
//         return eq;
//     }
// }
// 
// impl Eq for Connector {}
// 
// impl Graphics {
//     // 
// // fn init_valid_tile_ids(tiles: Vec<Tile>) -> HashMap<Direction, Vec<Vec<usize>>> {
// //     let mut valid_tile_ids = HashMap::from([
// //         (Direction::Left, Vec::new()),
// //         (Direction::Right, Vec::new()),
// //         (Direction::Front, Vec::new()),
// //         (Direction::Back, Vec::new()),
// //         (Direction::Down, Vec::new()),
// //         (Direction::Up, Vec::new()),
// //     ]);
// //     // TODO: Refactor as a struct to avoid unwraps?
// //     // TODO: Refactor to be less verbose
// //     // TODO: Refactor to reduce duplicated logic by storing as hashsets
// //     for (tile_id, tile) in tiles.iter().enumerate() {
// //         let mut valid_left_tile_ids = Vec::new();
// //         let mut valid_right_tile_ids = Vec::new();
// //         let mut valid_front_tile_ids = Vec::new();
// //         let mut valid_back_tile_ids = Vec::new();
// //         let mut valid_down_tile_ids = Vec::new();
// //         let mut valid_up_tile_ids = Vec::new();
// //         for (other_tile_id, other_tile) in tiles.iter().enumerate() {
// //             if tile.connectors.left == other_tile.connectors.right {
// //                 valid_left_tile_ids.push(other_tile_id);
// //             }
// //         }
// //         for (other_tile_id, other_tile) in tiles.iter().enumerate() {
// //             if tile.connectors.right == other_tile.connectors.left {
// //                 valid_right_tile_ids.push(other_tile_id);
// //             }
// //         }
// //         for (other_tile_id, other_tile) in tiles.iter().enumerate() {
// //             if tile.connectors.front == other_tile.connectors.back {
// //                 valid_front_tile_ids.push(other_tile_id);
// //             }
// //         }
// //         for (other_tile_id, other_tile) in tiles.iter().enumerate() {
// //             if tile.connectors.back == other_tile.connectors.front {
// //                 valid_back_tile_ids.push(other_tile_id);
// //             }
// //         }
// //         for (other_tile_id, other_tile) in tiles.iter().enumerate() {
// //             if tile.connectors.down == other_tile.connectors.up {
// //                 valid_down_tile_ids.push(other_tile_id);
// //             }
// //         }
// //         for (other_tile_id, other_tile) in tiles.iter().enumerate() {
// //             if tile.connectors.up == other_tile.connectors.down {
// //                 valid_up_tile_ids.push(other_tile_id);
// //             }
// //         }
// //         valid_tile_ids.get_mut(&Direction::Left).unwrap().push(valid_left_tile_ids);
// //         valid_tile_ids.get_mut(&Direction::Right).unwrap().push(valid_right_tile_ids);
// //         valid_tile_ids.get_mut(&Direction::Front).unwrap().push(valid_front_tile_ids);
// //         valid_tile_ids.get_mut(&Direction::Back).unwrap().push(valid_back_tile_ids);
// //         valid_tile_ids.get_mut(&Direction::Down).unwrap().push(valid_down_tile_ids);
// //         valid_tile_ids.get_mut(&Direction::Up).unwrap().push(valid_up_tile_ids);
// //     }
// //     return valid_tile_ids;
// // }
//     /// Reads config, generating transformed versions of them if necessary
//     pub fn new(sample_dir: &str) -> Self {
//         // TODO: Read from generated tiles.json if it exists
//         let config_path = Path::new(sample_dir).join("config.json");
//         let json = fs::read_to_string(config_path).expect("Unable to read sample config");
//         let mut graphics = serde_json::from_str::<Self>(&json).expect("Unable to parse sample config");
//         // TODO: Serialize tiles.json with generated tiles
//         let tiles = generate_tiles(sample_dir, graphics.tiles);
//         graphics.tiles = tiles;
//         return graphics;
//     }
//     handle.join().unwrap();handle.join().unwrap();
// 
//     // pub fn write_mv_import_file(&self, wave: Wave, output_path: &str, sample_dir: &str) {
//     //     // TODO: Store fq path as tile name so we don't have to take in sample dir
//     //     let file = File::create(output_path).expect("Unable to create vox viewer file");
//     //     let mut writer = BufWriter::new(file);
//     //     writer.write("// Generated wfc output\n".as_bytes());
//     //     let mv_import_size = max(wave.width(), max(wave.depth(), wave.height()))  * self.size;
//     //     let header = format!("mv_import {mv_import_size}\n", mv_import_size=mv_import_size);
//     //     writer.write(header.as_bytes());
//     //     for (slot, state) in wave.states().iter().enumerate() {
//     //         let (mut x, mut y, mut z) = coordinates(slot, wave.width(), wave.depth());
//     //         x *= self.size;
//     //         y *= self.size;
//     //         z *= self.size;
//     //         for (tile, state) in state.iter().enumerate() {
//     //             if *state {
//     //                 let path = Path::new(sample_dir)
//     //                     .join(&self.tiles[tile].name)
//     //                     .with_extension("vox");
//     //                 let absolute_path = path.canonicalize().unwrap();
//     //                 let absolute_path_str = absolute_path.to_str().unwrap();
//     //                 let tile = format!("{x} {y} {z} {path}\n", x=x, y=y, z=z, path=absolute_path_str);
//     //                 writer.write(tile.as_bytes());
//     //             }
//     //         }
//     //     }
//     // }
// 
//     pub fn tiles(&self) -> Vec<Tile> {
//         return self.tiles.clone();
//     }
// }
// 
// /// Generates transformed tiles from the base list given in the config
// fn generate_tiles(sample_dir: &str, tiles: Vec<Tile>) -> Vec<Tile> {
//     let mut generated_tiles = Vec::new();
//     for tile in tiles {
//         generated_tiles.push(tile.clone());
//         let mut rotated_tiles = rotated_tiles(sample_dir, &tile);
//         generated_tiles.append(&mut rotated_tiles);
//         // TODO: Implement reflections
//         // if graphic.reflect_x {
//         //     let image = image.flipv();
//         //     push_tiles(&mut tiles, &image, graphic.rotation_count);
//         // }
//         // if graphic.reflect_y {
//         //     let image = image.fliph();
//         //     push_tiles(&mut tiles, &image, graphic.rotation_count);
//         // }
//     }
//     return generated_tiles;
// }
// 
// /// Generates unique rotations
// fn rotated_tiles(sample_dir: &str, tile: &Tile) -> Vec<Tile> {
//     let mut rotated_tiles = Vec::new();
//     let mut previous_connectors = HashSet::from([tile.connectors.clone()]);
//     let max_rotation_count = 3;
//     let extension = "vox";
//     let sample_dir = Path::new(sample_dir);
//     let path = sample_dir
//         .join(&tile.name)
//         .with_extension(extension);
//     let file = File::open(path).expect("Unable to open tile file");
//     let mut vox = Vox::from(file);
//     let mut rotated_tile = tile.clone();
//     for rotation in 0..max_rotation_count {
//         vox.rotate_90_z();
//         rotated_tile = rotate_90_z(&tile.name, &rotated_tile, rotation);
//         if !previous_connectors.contains(&rotated_tile.connectors) {
//             previous_connectors.insert(rotated_tile.connectors.clone());
//             rotated_tiles.push(rotated_tile.clone());
//             let path = sample_dir
//                 .join(&rotated_tile.name)
//                 .with_extension(extension);
//             let file = File::create(path).expect("Unable to create rotated file");
//             vox.write(file);
//         }
//     }
//     return rotated_tiles;
// }
// 
// /// Rotate 90 degrees about the tile's center-point along the z axis
// fn rotate_90_z(original_tile_name: &str, tile: &Tile, rotation: usize) -> Tile {
//     let rotation_name = match rotation {
//         0 => "90",
//         1 => "180",
//         2 => "270",
//         _ => panic!("Unknown rotation: {}", rotation),
//     };
//     let rotated_tile_name = format!(
//         "{tile_name}-r{rotation_name}",
//         tile_name=original_tile_name,
//         rotation_name=rotation_name,
//     );
//     let mut up = tile.connectors.up.clone();
//     up.rotation = Some(up.rotation.unwrap() + 1);
//     let mut down = tile.connectors.down.clone();
//     down.rotation = Some(down.rotation.unwrap() + 1);
//     let rotated_tile = Tile {
//         name: rotated_tile_name,
//         connectors: Connectors {
//             left: tile.connectors.back.clone(),
//             right: tile.connectors.front.clone(),
//             front: tile.connectors.left.clone(),
//             back: tile.connectors.right.clone(),
//             up: tile.connectors.up.clone(),
//             down: tile.connectors.down.clone(),
//         }
//     };
//     return rotated_tile;
// }
// 
// pub fn fits(a: &Connector, b: &Connector) -> bool {
//     let id_fits = a.id == b.id;
//     let symmetry_fits = match a.symmetry {
//         SymmetryType::Normal => b.symmetry == SymmetryType::Rotated,
//         SymmetryType::Rotated => b.symmetry == SymmetryType::Normal,
//         SymmetryType::Symmetrical => b.symmetry == SymmetryType::Symmetrical,
//     };
//     let rotation_fits = a.rotation == b.rotation;
//     let fits = id_fits && symmetry_fits && rotation_fits;
//     return fits;
// }
// 
// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     #[test]
//     fn test_graphics() {
//         let sample_dir = "tests/samples/concrete";
//         let tiles = Graphics::new(sample_dir);
//         assert!(true);
//     }
// }
// 