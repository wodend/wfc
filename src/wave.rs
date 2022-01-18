use rand;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use std::{collections::{HashSet,HashMap}, hint::unreachable_unchecked};

use super::model::Face;

#[derive(Debug)]
/// A container to hold the state of `wfc` waves.
pub struct Waves {
    collapsed_count: usize,
    rng: ThreadRng,
    entropies: Vec<f32>,
    tiles: Vec<HashSet<usize>>,
}

#[derive(Debug)]
/// A `Waves` error
pub enum WavesError {
    Contradiction,
}

impl Waves {
    /// Constructs an uncollapsed `Waves`.
    pub fn new(wave_count: usize, tile_count: usize) -> Self {
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

    /// Propogates `constraints` over `graph` starting from `wave`.
    pub fn propogate(
        &mut self,
        constraints: &HashMap<Face, Vec<HashSet<usize>>>,
        graph: &Vec<Vec<(usize, Face)>>,
        wave: usize,
    ) -> Result<(), WavesError> {
        let mut stack = vec![wave];
        let mut visited = HashSet::new();
        while let Some(wave) = stack.pop() {
            visited.insert(wave);
            // TODO: Spawn a new thread for each face
            let mut constrain_results = Vec::new();
            for (edge_wave, edge_face) in graph[wave].iter() {
                if !visited.contains(edge_wave) && self.entropies[*edge_wave] > 0.0 {
                    constrain_results.push(self.constrain(
                        *edge_wave,
                        constraints,
                        wave,
                        edge_face,
                    ));
                }
            }
            for result in constrain_results {
                let (edge_wave, removed_tile_counts) = result?;
                if removed_tile_counts > 0 {
                    stack.push(edge_wave);
                }
            }
        }
        return Ok(());
    }

    /// Constrains `edge_wave` with `constraints` of `wave` on `edge_face`.
    fn constrain(
        &mut self,
        edge_wave: usize,
        constraints: &HashMap<Face, Vec<HashSet<usize>>>,
        wave: usize,
        edge_face: &Face,
    ) -> Result<(usize, usize), WavesError> {
        let constraints = &constraints[edge_face];
        let mut valid_tiles = HashSet::new();
        for tile in self.tiles[wave].iter() {
            for valid_tile in constraints[*tile].iter() {
                valid_tiles.insert(*valid_tile);
            }
        }
        let mut removed_tile_count = 0;
        let tiles = self.tiles[edge_wave].clone();
        for tile in tiles.iter() {
            if !valid_tiles.contains(tile) {
                self.entropies[edge_wave] -= 1.0;
                self.tiles[edge_wave].remove(tile);
                removed_tile_count += 1;
            }
        }
        if self.tiles[edge_wave].is_empty() {
            return Err(WavesError::Contradiction);
        }
        //if self.tiles[edge_wave].len() == 1 {
        //    //println!("Constrain collapse {:?}", edge_wave);
        //    self.collapse(edge_wave);
        //}
        return Ok((edge_wave, removed_tile_count));
    }

    /// Returns true if all waves are collapsed
    pub fn are_collapsed(&self) -> bool {
        return self.collapsed_count == self.entropies.len();
    }

    /// Returns the current `tiles` state of all waves
    pub fn tiles(&self) -> Vec<HashSet<usize>> {
        return self.tiles.clone();
    }
}
