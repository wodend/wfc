mod wave;
mod graphics;
use crate::wave::Wave;

pub fn run(sample_dir: &str, width: usize, height: usize, output_path: &str) -> bool {
    let config = graphics::read_config(sample_dir);
    let tiles = graphics::tiles(&config, sample_dir);
    let connectors = graphics::connectors(&config);
    let mut wave = Wave::new(connectors, width, height);
    loop {
        let slot = wave.lowest_entropy_slot();
        wave.observe(slot);
        let is_contradiction = wave.propogate(slot);
        if is_contradiction {
            graphics::render(wave, config, tiles, output_path);
            return false;
        }
        if wave.is_collapsed() {
            graphics::render(wave, config, tiles, output_path);
            return true;
        }
    }
}