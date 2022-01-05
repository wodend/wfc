use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::collections::HashMap;

use super::graphics::{Direction, Connector};

#[derive(Debug)]
pub struct Wave {
    len: usize,
    width: usize,
    height: usize,
    observed_count: usize,
    rng: ThreadRng,
    connector_map: HashMap<Direction, Vec<Connector>>,
    neighbors: Vec<Vec<(Direction, usize)>>, // [slot]
    states: Vec<Vec<bool>>, // [slot][tile]
    entropies: Vec<f32>, // [slot] TODO: Store entropies as a min heap?
}

fn valid_neighbors(width: usize, height: usize) -> Vec<Vec<(Direction, usize)>> {
    let slot_count = width * height;
    let mut valid_neighbors = vec![Vec::new(); slot_count];
    for (slot, neighbors) in valid_neighbors.iter_mut().enumerate() {
        let x = slot % width;
        let y = slot / width;
        if x > 0 {
            neighbors.push((Direction::Left, slot-1));
        }
        if x < width-1 {
            neighbors.push((Direction::Right, slot+1));
        }
        if y > 0 {
            neighbors.push((Direction::Up, slot-width));
        }
        if y < height-1 {
            neighbors.push((Direction::Down, slot+width));
        }
    }
    return valid_neighbors;
}

impl Wave {
    pub fn new(connector_map: HashMap<Direction, Vec<Connector>>, width: usize, height: usize) -> Self {
        let tile_count = connector_map[&Direction::Left].len();
        let slot_count = width * height;
        let states = vec![vec![true; tile_count]; slot_count];
        let mut rng = thread_rng();
        let mut entropies = vec![0.0; slot_count];
        let neighbors = valid_neighbors(width, height);
        for entropy in entropies.iter_mut() {
            *entropy = tile_count as f32 + rng.gen::<f32>();
        }
        return Self {
            len: slot_count,
            width: width,
            height: height,
            observed_count: 0,
            rng: rng,
            connector_map: connector_map,
            neighbors: neighbors,
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
        let observed_tile = possible_tiles.choose(&mut self.rng).unwrap();
        self.states[slot][*observed_tile] = true;
        self.entropies[slot] = 0.0;
        self.observed_count += 1;
    }

    pub fn propogate(&mut self, slot: usize) -> bool {
        let mut stack = vec![slot];
        let mut visited = HashSet::new();
        while !stack.is_empty() {
            let current_slot = stack.pop().unwrap();
            if visited.contains(&current_slot) {
                stack.pop();
            } else {
                visited.insert(current_slot);
                let current_neighbors = &self.neighbors[current_slot];
                for (direction, neighbor_slot) in current_neighbors.iter() {
                    let mut possible_connectors = HashSet::new();
                    for (tile, is_possible) in self.states[current_slot].iter().enumerate() {
                        if *is_possible {
                            let connector = self.connector_map[&direction][tile];
                            possible_connectors.insert(connector);
                        }
                    }
                    let reverse = match direction {
                        Direction::Left => Direction::Right,
                        Direction::Right => Direction::Left,
                        Direction::Up => Direction::Down,
                        Direction::Down => Direction::Up,
                    };
                    for (tile, is_possible) in self.states[*neighbor_slot].iter_mut().enumerate() {
                        let connector = self.connector_map[&reverse][tile];
                        if *is_possible && !possible_connectors.contains(&connector) {
                            *is_possible = false;
                            self.entropies[*neighbor_slot] -= 1.0;
                            if self.entropies[*neighbor_slot] < 1.0 { // No possible tiles
                                return true;
                            }
                            // TODO: Observe neighbor_slot if entropy < 2.0
                            // if self.entropies[*neighbor_slot] < 2.0 { // Only one possible tile
                            // }
                        }
                    }
                    stack.push(*neighbor_slot);
                }
            }
        }
        return false;
    }

    pub fn is_collapsed(&self) -> bool {
        return self.observed_count == self.len;
    }

    pub fn width(&self) -> usize {
        return self.width;
    }

    pub fn height(&self) -> usize {
        return self.height;
    }

    pub fn states(&self) -> &Vec<Vec<bool>> {
        return &self.states;
    }
}