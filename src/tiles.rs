use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use image::{DynamicImage, GenericImage};

#[derive(Debug, PartialEq, Eq)]
pub struct Tiles {
    pub len: usize,
    pub path: String,
    pub size: usize,
    pub extension: String,
    pub connectors: HashMap<Direction, Vec<usize>>, // [tile]
    pub reverse: HashMap<Direction, Direction>,
    graphics: Vec<String>, // [tile]
    transformations: Vec<Transformation>, // [tile]
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
 enum Transformation {
    None,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[derive(Debug, Deserialize)]
struct Config {
    pub size: usize,
    pub extension: String,
    pub graphics: Vec<Graphic>,
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

impl Graphic {
    fn rotate90(&mut self) {
        let new_up = self.left;
        let new_down = self.right;
        self.left = self.down;
        self.right = self.up;
        self.up = new_up;
        self.down = new_down;
        self.rotation_count -= 1;
    }
}

impl Tiles {
    pub fn new(sample_dir: &str) -> Self {
        let config_path = Path::new(sample_dir).join("config.json");
        let json = fs::read_to_string(config_path).expect("Unable to read sample config");
        let mut config = serde_json::from_str::<Config>(&json).expect("Unable to parse sample config");
        let mut connectors = HashMap::from([
            (Direction::Left, Vec::new()),
            (Direction::Right, Vec::new()),
            (Direction::Up, Vec::new()),
            (Direction::Down, Vec::new()),
        ]);
        let mut graphics = Vec::new();
        let mut transformations = Vec::new();
        let reverse = HashMap::from([
            (Direction::Left, Direction::Right),
            (Direction::Right, Direction::Left),
            (Direction::Up, Direction::Down),
            (Direction::Down, Direction::Up),
        ]);
        for graphic in config.graphics.iter_mut() {
            graphics.push(graphic.name.clone());
            connectors.get_mut(&Direction::Left).unwrap().push(graphic.left);
            connectors.get_mut(&Direction::Right).unwrap().push(graphic.right);
            connectors.get_mut(&Direction::Up).unwrap().push(graphic.up);
            connectors.get_mut(&Direction::Down).unwrap().push(graphic.down);
            transformations.push(Transformation::None);
            let possible_transformations = [Transformation::Rotate90, Transformation::Rotate180, Transformation::Rotate270];
            let mut current_transformation = 0;
            while graphic.rotation_count > 0 {
                graphics.push(graphic.name.clone());
                graphic.rotate90();
                connectors.get_mut(&Direction::Left).unwrap().push(graphic.left);
                connectors.get_mut(&Direction::Right).unwrap().push(graphic.right);
                connectors.get_mut(&Direction::Up).unwrap().push(graphic.up);
                connectors.get_mut(&Direction::Down).unwrap().push(graphic.down);
                transformations.push(possible_transformations[current_transformation].clone());
                current_transformation += 1;
            }
        }
        return Self {
            len: transformations.len(),
            path: sample_dir.to_string(),
            size: config.size,
            extension: config.extension,
            connectors: connectors,
            reverse: reverse,
            graphics: graphics,
            transformations: transformations,
        };
    }

    pub fn render(&self, width: usize, height: usize, states: &Vec<Vec<bool>>, output_path: &str) {
        println!("{:?}", states);
        let output_width = (width * self.size) as u32;
        let output_height = (height * self.size) as u32;
        let mut output = DynamicImage::new_rgba8(output_width, output_height);
        for (slot, state) in states.iter().enumerate() {
            let x = ((slot % width) * self.size) as u32;
            let y = ((slot / width) * self.size) as u32;
            self.copy_graphics(&mut output, state, x, y);
        }
        output.save(output_path).unwrap();
    }

    pub fn copy_graphics(&self, output: &mut DynamicImage, state: &Vec<bool>, x: u32, y: u32) {
        for (tile, state) in state.iter().enumerate() {
            if *state {
                let graphic = &self.graphics[tile];
                let transformation = &self.transformations[tile];
                let image_path = Path::new(&self.path).join(graphic).with_extension(&self.extension);
                let mut image = image::open(image_path).unwrap();
                image = match transformation {
                    Transformation::None => image,
                    Transformation::Rotate90 => image.rotate90(),
                    Transformation::Rotate180 => image.rotate180(),
                    Transformation::Rotate270 => image.rotate270(),
                };
                if let Err(err) = output.copy_from(&image, x, y) {
                    println!("Error while copying image to output: {}", err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sample_dir= "src/unit_tests";
        let tiles = Tiles::new(sample_dir);
        let expected_tiles = Tiles {
            len: 5,
            path: sample_dir.to_string(),
            size: 10,
            extension: "png".to_string(),
            connectors: HashMap::from([
                (Direction::Left, vec![
                    0,
                    0,
                    1,
                    1,
                    0,
                ]),
                (Direction::Right, vec![
                    1,
                    1,
                    0,
                    0,
                    0,
                ]),
                (Direction::Up, vec![
                    1,
                    0,
                    0,
                    1,
                    0,
                ]),
                (Direction::Down, vec![
                    0,
                    1,
                    1,
                    0,
                    0,
                ]),
            ]),
            reverse: HashMap::from([
                (Direction::Left, Direction::Right),
                (Direction::Right, Direction::Left),
                (Direction::Up, Direction::Down),
                (Direction::Down, Direction::Up),
            ]),
            graphics: vec![
                "corner".to_string(),
                "corner".to_string(),
                "corner".to_string(),
                "corner".to_string(),
                "empty".to_string(),
            ],
            transformations: vec![
                Transformation::None,
                Transformation::Rotate90,
                Transformation::Rotate180,
                Transformation::Rotate270,
                Transformation::None,
            ],
        };
        assert_eq!(tiles, expected_tiles);
    }
}