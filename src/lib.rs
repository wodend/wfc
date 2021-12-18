mod wave;
mod tiles;

use crate::wave::Wave;

pub fn run(sample_dir: &str, width: usize, height: usize, output_path: &str) -> bool {
    let mut wave = Wave::new(sample_dir, width, height);
    let mut i = 0;
    loop {
        let slot = wave.lowest_entropy_slot();
        wave.observe(slot);
        let is_contradiction = wave.propogate(slot);
        if is_contradiction {
            println!("contradiction! {} {:?}", slot, wave.states);
            wave.render(output_path);
            return false;
        }
        if wave.is_collapsed() {
            wave.render(output_path);
            return true;
        }
        println!("{} slot: {}", i, slot);
        println!("{} states: {:?}", i, wave.states);
        println!("{} entropies: {:?}", i, wave.entropies);
        i += 1;
        //println!("slot: {} end: {:?}", slot, wave.states);
    }
}