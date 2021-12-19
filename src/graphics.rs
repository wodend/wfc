use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::Deserialize;
use image::{DynamicImage, GenericImage};
use crate::wave::Wave;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

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
    left: usize,
    right: usize,
    up: usize,
    down: usize,
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
        let mut image = image::open(path).unwrap();
        tiles.push(image.clone());
        for _ in 0..graphic.rotation_count {
            image = image.rotate90();
            tiles.push(image.clone());
        }
    }
    return tiles;
}

pub fn connectors(config: &Config) -> HashMap<Direction, Vec<usize>> {
    // TODO: Serialize connectors and read from CSV?
    let mut connectors = HashMap::from([
        (Direction::Left, Vec::new()),
        (Direction::Right, Vec::new()),
        (Direction::Up, Vec::new()),
        (Direction::Down, Vec::new()),
    ]);
    for graphic in &config.graphics {
        let mut left = graphic.left;
        let mut right = graphic.right;
        let mut up = graphic.up;
        let mut down = graphic.down;
        connectors.get_mut(&Direction::Left).unwrap().push(left);
        connectors.get_mut(&Direction::Right).unwrap().push(right);
        connectors.get_mut(&Direction::Up).unwrap().push(up);
        connectors.get_mut(&Direction::Down).unwrap().push(down);
        for _ in 0..graphic.rotation_count {
            // Rotate 90 degrees clockwise
            let tmp = left;
            left = down;
            down = right;
            right = up;
            up = tmp;
            connectors.get_mut(&Direction::Left).unwrap().push(left);
            connectors.get_mut(&Direction::Right).unwrap().push(right);
            connectors.get_mut(&Direction::Up).unwrap().push(up);
            connectors.get_mut(&Direction::Down).unwrap().push(down);
        }
    }
    return connectors;
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
                let image= &tiles[tile];
                if let Err(err) = output.copy_from(image, x, y) {
                    println!("Error while copying image to output: {}", err);
                }
            }
        }
    }
    output.save(output_path).unwrap();
}