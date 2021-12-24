use std::{collections::HashMap, hash::Hash};
use std::path::Path;
use std::fs;
use serde::Deserialize;
use image::{DynamicImage, GenericImage};
use crate::wave::Wave;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Direction {
    Left = 0,
    Right = 1,
    Up = 2,
    Down = 3,
}
const DIRECTION_COUNT: usize = 4;

impl Direction {
    fn from_usize(value: usize) -> Direction {
        return match value {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            3 => Direction::Down,
            _ => panic!("Unknown Direction: {}", value),
        }
    }
}

pub type Connector = (usize, i8);
type ConnectorSpec = [Connector; DIRECTION_COUNT];

#[derive(Debug, Deserialize)]
pub struct Config {
    size: usize,
    extension: String,
    graphics: Vec<Graphic>,
}

#[derive(Debug, Deserialize)]
struct Graphic {
    name: String,
    rotation_count: usize,
    reflect_x: bool,
    reflect_y: bool,
    connector_spec: ConnectorSpec,
}

pub fn read_config(sample_dir: &str) -> Config {
    let config_path = Path::new(sample_dir).join("config.json");
    let json = fs::read_to_string(config_path).expect("Unable to read sample config");
    return serde_json::from_str::<Config>(&json).expect("Unable to parse sample config");
}

pub fn tiles(config: &Config, sample_dir: &str) -> Vec<DynamicImage> {
    let mut tiles = Vec::new();
    for graphic in &config.graphics {
        let path = Path::new(sample_dir).join(&graphic.name).with_extension(&config.extension);
        let image = image::open(path).unwrap();
        push_tiles(&mut tiles, &image, graphic.rotation_count);
        if graphic.reflect_x {
            let image = image.flipv();
            push_tiles(&mut tiles, &image, graphic.rotation_count);
        }
        if graphic.reflect_y {
            let image = image.fliph();
            push_tiles(&mut tiles, &image, graphic.rotation_count);
        }
    }
    return tiles;
}

pub fn push_tiles(tiles: &mut Vec<DynamicImage>, image: &DynamicImage, rotation_count: usize) {
    let mut image = image.clone();
    tiles.push(image.clone());
    for _ in 0..rotation_count {
        image = image.rotate90();
        tiles.push(image.clone());
    }
}

pub fn connector_map(config: &Config) -> HashMap<Direction, Vec<Connector>> {
    let mut connector_map = HashMap::new();
    for direction in 0..DIRECTION_COUNT {
        connector_map.insert(
            Direction::from_usize(direction),
            Vec::new(),
        );
    }
    for graphic in &config.graphics {
        println!("{:?}", graphic.name);
        let connector_spec = graphic.connector_spec;
        insert_connections(&mut connector_map, connector_spec, graphic.rotation_count);
        if graphic.reflect_x {
            let connector_spec = reflect_x(connector_spec);
            insert_connections(&mut connector_map, connector_spec, graphic.rotation_count);
        }
        if graphic.reflect_y {
            let connector_spec = reflect_y(connector_spec);
            insert_connections(&mut connector_map, connector_spec, graphic.rotation_count);
        }
    }
    return connector_map;
}

fn insert_connections(connector_map: &mut HashMap<Direction, Vec<Connector>>, connector_spec: ConnectorSpec, rotation_count: usize) {
    let mut connector_spec = connector_spec.clone();
    println!("{:?}", connector_spec);
    insert(connector_map, connector_spec);
    for _ in 0..rotation_count {
        connector_spec = rotate90(connector_spec);
        println!("{:?}", connector_spec);
        insert(connector_map, connector_spec);
    }
}

fn insert(connector_map: &mut HashMap<Direction, Vec<Connector>>, connector_spec: ConnectorSpec) {
    for (direction_id, connector) in connector_spec.iter().enumerate() {
        let direction = Direction::from_usize(direction_id);
        connector_map.get_mut(&direction).unwrap().push(*connector);
    }
}

fn rotate90(connector_spec: ConnectorSpec) -> ConnectorSpec {
    let mut output = [(0, 0); 4];
    output[Direction::Left as usize] = connector_spec[Direction::Down as usize];
    output[Direction::Right as usize] = connector_spec[Direction::Up as usize];
    output[Direction::Up as usize] = connector_spec[Direction::Left as usize];
    output[Direction::Down as usize] = connector_spec[Direction::Right as usize];
    for direction in 0..DIRECTION_COUNT {
        if output[direction].1 != -1 {
            output[direction].1 = (output[direction].1 + 1) % 4;
        }
    }
    return output;
}

fn reflect_x(connector_spec: ConnectorSpec) -> ConnectorSpec {
    let mut output = [(0, 0); 4];
    output[Direction::Up as usize] = connector_spec[Direction::Down as usize];
    output[Direction::Down as usize] = connector_spec[Direction::Up as usize];
    output[Direction::Left as usize] = connector_spec[Direction::Left as usize];
    output[Direction::Right as usize] = connector_spec[Direction::Right as usize];
    output[Direction::Left as usize].1 = (connector_spec[Direction::Left as usize].1 + 2) % 4;
    output[Direction::Right as usize].1 = (connector_spec[Direction::Right as usize].1 + 2) % 4;
    return output;
}

fn reflect_y(connector_spec: ConnectorSpec) -> ConnectorSpec {
    let mut output = [(0, 0); 4];
    output[Direction::Left as usize] = connector_spec[Direction::Right as usize];
    output[Direction::Right as usize] = connector_spec[Direction::Left as usize];
    output[Direction::Up as usize] = connector_spec[Direction::Up as usize];
    output[Direction::Down as usize] = connector_spec[Direction::Down as usize];
    output[Direction::Up as usize].1 = (connector_spec[Direction::Up as usize].1 + 2) % 4;
    output[Direction::Down as usize].1 = (connector_spec[Direction::Down as usize].1 + 2) % 4;
    return output;
}

pub fn render(wave: Wave, config: Config, tiles: Vec<DynamicImage>, output_path: &str) {
    let output_width = (wave.width() * config.size) as u32;
    let output_height = (wave.height() * config.size) as u32;
    let mut output = DynamicImage::new_rgba8(output_width, output_height);
    for (slot, state) in wave.states().iter().enumerate() {
        let x = ((slot % wave.width()) * config.size) as u32;
        let y = ((slot / wave.width()) * config.size) as u32;
        for (tile, state) in state.iter().enumerate() {
            if *state {
                let image = &tiles[tile];
                if let Err(err) = output.copy_from(image, x, y) {
                    println!("Error while copying image to output: {}", err);
                }
            }
        }
    }
    output.save(output_path).unwrap();
}