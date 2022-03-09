use std::collections::{HashMap, HashSet};

use rand;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use super::model::Face;

#[derive(Debug)]
/// A container to hold the state of `wfc` waves.
pub struct Waves<'a> {
    graph: &'a Vec<Vec<(usize, Face)>>,
    constraints: &'a HashMap<Face, Vec<HashSet<usize>>>,
    collapsed_count: usize,
    rng: ThreadRng,
    entropies: Vec<f32>,
    tiles: Vec<HashSet<usize>>,
}

#[derive(Debug)]
/// A wfc contradiction at the specified wave
pub struct Contradiction {
    pub wave: usize,
    pub tiles: HashSet<usize>,
    pub face: Face,
}

impl<'a> Waves<'a> {
    /// Constructs an uncollapsed `Waves`.
    pub fn new(
        graph: &'a Vec<Vec<(usize, Face)>>,
        constraints: &'a HashMap<Face, Vec<HashSet<usize>>>,
    ) -> Self {
        let wave_count = graph.len();
        let tile_count = constraints[&Face::Left].len();
        let collapsed_count = 0;
        let mut rng = rand::thread_rng();
        let mut entropies = vec![tile_count as f32; wave_count];
        for entropy in entropies.iter_mut() {
            *entropy += rng.gen::<f32>(); // Add noise to break min entropy ties
        }
        let mut all_tiles = HashSet::new();
        for tile in 0..tile_count {
            all_tiles.insert(tile);
        }
        let tiles = vec![all_tiles; wave_count];
        return Self {
            graph: &graph,
            constraints: &constraints,
            collapsed_count: collapsed_count,
            entropies: entropies,
            tiles: tiles,
            rng: rng,
        };
    }

    /// Returns the minimum entropy wave.
    pub fn min_entropy_wave(&self) -> usize {
        let mut min_entropy_wave = 0;
        let mut min_entropy = f32::MAX;
        for (wave, entropy) in self.entropies.iter().enumerate() {
            if *entropy > 0.0 && *entropy < min_entropy {
                // println!("found min entropy {} {}", wave, *entropy);
                min_entropy_wave = wave;
                min_entropy = *entropy;
            }
        }
        return min_entropy_wave;
    }

    /// Picks a tile at random from the tiles of `wave`.
    pub fn observe(&mut self, wave: usize) {
        let tiles = Vec::from_iter(self.tiles[wave].clone());
        let observed_tile = tiles.choose(&mut self.rng).unwrap();
        self.tiles[wave] = HashSet::from([*observed_tile]);
        self.collapse(wave);
    }

    /// Marks a wave as collapsed.
    fn collapse(&mut self, wave: usize) {
        self.entropies[wave] = 0.0;
        self.collapsed_count += 1;
    }

    /// Propogates constraints over graph starting from `wave`.
    pub fn propogate(&mut self, wave: usize) -> Result<(), Contradiction> {
        let mut stack = vec![wave];
        let mut visited = HashSet::new();
        // let mut observed_waves = HashSet::new();
        while let Some(wave) = stack.pop() {
            visited.insert(wave);
            // TODO: Spawn a new thread for each face
            // TODO: Benchmark to determine which of the following optimizations is better:
            // 1. Observe tiles which only have 1 possible tile after propogation
            // 2. Stop propogating when we reach an unchanged tile
            // Due to bugs in approach 1, we are going with approach 2 for now
            // let mut initial_tile_counts = Vec::new();
            for (edge_wave, edge_face) in self.graph[wave].iter() {
                if !visited.contains(edge_wave) && self.entropies[*edge_wave] > 0.0 {
                    //initial_tile_counts.push((*edge_wave, self.tiles[*edge_wave].len()));
                    let initial_tile_count = self.tiles[*edge_wave].len();
                    self.constrain(*edge_wave, self.constraints, wave, edge_face);
                    if self.tiles[*edge_wave].is_empty() {
                        return Err(
                            Contradiction {
                                wave: wave,
                                tiles: self.tiles[wave].clone(),
                                face: edge_face.clone(),
                            }
                        );
                    }
                    if self.tiles[*edge_wave].len() != initial_tile_count {
                        stack.push(*edge_wave);
                    }
                }
                // For approach 1, we must always push unvisited edges
                // if !visited.contains(edge_wave) {
                //     stack.push(*edge_wave);
                // }
            }
            // for (edge_wave, initial_tile_count) in initial_tile_counts {
            //     if self.tiles[edge_wave].is_empty() {
            //         return Err(
            //             Contradiction {
            //                 wave: wave,
            //                 tiles: self.tiles[wave],

            //             }
            //         );
            //     }
            //     // if self.tiles[edge_wave].len() == 1 {
            //     //     if self.entropies[edge_wave] < 1.0 {
            //     //         println!("unreachable");
            //     //     }
            //     //     observed_waves.insert(edge_wave);
            //     // }
            //     if self.tiles[edge_wave].len() != initial_tile_count {
            //         stack.push(edge_wave);
            //     }
            // }
        }
        // for wave in observed_waves.iter() {
        //     self.collapse(*wave);
        // }
        return Ok(());
    }

    /// Constrains `edge_wave` with `constraints` of `wave` on `edge_face`.
    fn constrain(
        &mut self,
        edge_wave: usize,
        constraints: &HashMap<Face, Vec<HashSet<usize>>>,
        wave: usize,
        edge_face: &Face,
    ) {
        let constraints = &constraints[edge_face];
        let mut valid_tiles = HashSet::new();
        for tile in self.tiles[wave].iter() {
            for valid_tile in constraints[*tile].iter() {
                valid_tiles.insert(*valid_tile);
            }
        }
        let tiles = self.tiles[edge_wave].clone();
        println!("{:?}", tiles);
        for tile in tiles.iter() {
            if !valid_tiles.contains(tile) {
                self.entropies[edge_wave] -= 1.0;
                self.tiles[edge_wave].remove(tile);
            }
        }
    }

    /// Returns true if all waves are collapsed
    pub fn are_collapsed(&self) -> bool {
        return self.collapsed_count == self.entropies.len();
    }

    /// Returns the current `tiles` state of all waves
    pub fn tiles(&self) -> &Vec<HashSet<usize>> {
        return &self.tiles;
    }
}
