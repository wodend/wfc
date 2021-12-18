//extern crate rand;

use crate::tiles::Tiles;
use crate::tiles::Direction;
use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::thread::current;

#[derive(Debug)]
pub struct Wave {
    len: usize,
    width: usize,
    height: usize,
    observed_count: usize,
    rng: ThreadRng,
    tiles: Tiles,
    pub states: Vec<Vec<bool>>, // [slot][tile] TODO: make private
    pub entropies: Vec<f32>, // [slot]
}

impl Wave {
    pub fn new(sample_dir: &str, width: usize, height: usize) -> Self {
        let tiles = Tiles::new(sample_dir);
        let tile_count = tiles.len;
        let slot_count = width * height;
        let mut states = vec![vec![true; tile_count]; slot_count];
        let mut rng = thread_rng();
        let mut entropies = vec![0.0; slot_count];
        for entropy in entropies.iter_mut() {
            *entropy = tile_count as f32 + rng.gen::<f32>();
        }
        println!("initial states: {:?}", states);
        println!("initial entropies: {:?}", entropies);
        return Self {
            len: slot_count,
            width: width,
            height: height,
            observed_count: 0,
            rng: rng,
            tiles: tiles,
            states: states,
            entropies: entropies,
        };
    }

    pub fn lowest_entropy_slot(&self) -> usize {
        let mut min_slot = 0;
        let mut min_entropy= self.len as f32 + 2.0;
        for (slot, entropy) in self.entropies.iter().enumerate() {
            if *entropy > 0.0 && *entropy < min_entropy {
                min_slot = slot;
                min_entropy = *entropy;
            }
        }
        return min_slot;
    }

    pub fn observe(&mut self, slot: usize) {
        let mut possible_tiles = Vec::new();
        for (tile, is_possible) in self.states[slot].iter_mut().enumerate() {
            if *is_possible {
                possible_tiles.push(tile);
            }
            *is_possible = false;
        }
        //println!("slot: {} possible: {:?}", slot, possible_tiles);
        let observed_tile = possible_tiles.choose(&mut self.rng).unwrap();
        self.states[slot][*observed_tile] = true;
        self.entropies[slot] = 0.0;
        self.observed_count += 1;
    }

    pub fn propogate(&mut self, slot: usize) -> bool {
        let mut stack = vec![slot];
        let mut visited = HashSet::new();
        let mut i = 0;
        while !stack.is_empty() {
            //println!("{} stack: {:?}", i, stack);
            //println!("{} visited: {:?}", i, visited);
            let current_slot = stack.pop().unwrap();
            //println!("{} current: {:?}", i, current_slot);
            if visited.contains(&current_slot) {
                stack.pop();
            } else {
                visited.insert(current_slot);
                for (direction, neighbor_slot) in self.neighbors(current_slot) {
                    //println!("{} neighbor: {:?}", i, neighbor_slot);
                    let mut possible_connectors = HashSet::new();
                    for (tile, is_possible) in self.states[current_slot].iter().enumerate() {
                        if *is_possible {
                            let connector = self.tiles.connectors[&direction][tile];
                            possible_connectors.insert(connector);
                        }
                    }
                    let reverse = &self.tiles.reverse[&direction];
                    for (tile, is_possible) in self.states[neighbor_slot].iter_mut().enumerate() {
                        let connector = self.tiles.connectors[&reverse][tile];
                        if *is_possible && !possible_connectors.contains(&connector) {
                            *is_possible = false;
                            self.entropies[neighbor_slot] -= 1.0;
                            // TODO: Observe neighbor_slot if entropy < 2.0
                            // TODO: Return false if entropy < 1.0
                        }
                    }
                    stack.push(neighbor_slot);
                }
            }
            i += 1;
        }
        return false;
    }

    fn neighbors(&self, slot: usize) -> Vec<(Direction, usize)> {
        let mut neighbors = Vec::new();
        let x = slot % self.width;
        let y = slot / self.width;
        if x > 0 {
            neighbors.push((Direction::Left, slot-1));
        }
        if x < self.width-1 {
            neighbors.push((Direction::Right, slot+1));
        }
        if y > 0 {
            neighbors.push((Direction::Up, slot-self.width));
        }
        if y < self.height-1 {
            neighbors.push((Direction::Down, slot+self.width));
        }
        return neighbors;
    }

    pub fn is_collapsed(&self) -> bool {
        return self.observed_count == self.len;
    }

    pub fn render(&self, output_path: &str) {
        self.tiles.render(self.width, self.height, &self.states, output_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbors_corner() {
        let sample_dir= "src/unit_tests";
        let wave = Wave::new(sample_dir, 3, 3);
        let slot = 0;
        let neighbors = wave.neighbors(slot);
        let expected_neighbors = vec![
            (Direction::Right, 1),
            (Direction::Down, 3),
        ];
        assert_eq!(neighbors, expected_neighbors);
    }

    #[test]
    fn test_neighbors_edge() {
        let sample_dir= "src/unit_tests";
        let wave = Wave::new(sample_dir, 3, 3);
        let slot= 1;
        let neighbors = wave.neighbors(slot);
        let expected_neighbors = vec![
            (Direction::Left, 0),
            (Direction::Right, 2),
            (Direction::Down, 4),
        ];
        assert_eq!(neighbors, expected_neighbors);
    }

    #[test]
    fn test_neighbors_middle() {
        let sample_dir= "src/unit_tests";
        let wave = Wave::new(sample_dir, 3, 3);
        let slot= 4;
        let neighbors = wave.neighbors(slot);
        let expected_neighbors = vec![
            (Direction::Left, 3),
            (Direction::Right, 5),
            (Direction::Up, 1),
            (Direction::Down, 7),
        ];
        assert_eq!(neighbors, expected_neighbors);
    }

    #[test]
    fn test_wave() {
        let sample_dir= "src/unit_tests";
        let mut wave = Wave::new(sample_dir, 2, 1);
        let slot = wave.lowest_entropy_slot();
        wave.observe(slot);
        wave.propogate(slot);

        println!("{:?}", wave.observed_count);
        println!("{:?}", wave.states);
        println!("{:?}", wave.entropies);
        wave.observe(slot);
        wave.propogate(slot);

        println!("{:?}", wave.observed_count);
        println!("{:?}", wave.states);
        println!("{:?}", wave.entropies);
        assert!(true);
    }
}