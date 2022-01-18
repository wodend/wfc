use std::fs::File;
use std::io::BufRead;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Write};
use std::path::Path;

use super::tile::Rotation;

const TAG_SIZE: usize = 4;
const VOX_TAG: &[u8; TAG_SIZE] = b"VOX ";
const INT_SIZE: usize = 4;
const VERSION: i32 = 150;

const MAIN_TAG: &[u8; TAG_SIZE] = b"MAIN";
const CHUNK_BYTE_COUNTS_SIZE: usize = 8;

const SIZE_TAG: &[u8; TAG_SIZE] = b"SIZE";

const XYZI_TAG: &[u8; TAG_SIZE] = b"XYZI";
const XYZI_SIZE: usize = 4;

const RGBA_TAG: &[u8; TAG_SIZE] = b"RGBA";
const PALETTE_RGBA_COUNT: usize = 256;
const RGBA_SIZE: usize = 4;

// TODO: Propogate these errors up the stack
fn read_tag(reader: &mut BufReader<File>) -> [u8; TAG_SIZE] {
    let mut tag = [0; TAG_SIZE];
    reader.read(&mut tag).unwrap();
    return tag;
}

fn read_int(reader: &mut BufReader<File>) -> i32 {
    let mut int_bytes = [0; INT_SIZE];
    reader.read(&mut int_bytes).unwrap();
    let i = i32::from_le_bytes(int_bytes);
    return i;
}

fn read_xyzi(reader: &mut BufReader<File>) -> [u8; XYZI_SIZE] {
    let mut xyzi = [0; XYZI_SIZE];
    reader.read(&mut xyzi).unwrap();
    return xyzi;
}

fn read_rgba(reader: &mut BufReader<File>) -> [u8; RGBA_SIZE] {
    let mut rgba = [0; RGBA_SIZE];
    reader.read(&mut rgba).unwrap();
    return rgba;
}

fn write_tag(tag: &[u8; TAG_SIZE], writer: &mut BufWriter<File>) {
    writer.write(tag).unwrap();
}

fn write_int(i: i32, writer: &mut BufWriter<File>) {
    let int_bytes = i32::to_le_bytes(i);
    writer.write(&int_bytes).unwrap();
}

fn write_xyzi(xyzi: &[u8; XYZI_SIZE], writer: &mut BufWriter<File>) {
    writer.write(xyzi).unwrap();
}

fn write_rgba(rgba: &[u8; RGBA_SIZE], writer: &mut BufWriter<File>) {
    writer.write(rgba).unwrap();
}

/// A MagicaVoxel object
pub struct Vox {
    version: i32,
    x_size: i32,
    y_size: i32,
    z_size: i32,
    voxel_count: i32,
    xyzis: Vec<[u8; XYZI_SIZE]>,
    palette: [[u8; RGBA_SIZE]; PALETTE_RGBA_COUNT],
}

impl Vox {
    /// Read object data from a file
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let vox_tag = read_tag(&mut reader);
        if vox_tag != *VOX_TAG {
            panic!("Unsupported file format");
        }
        let version = read_int(&mut reader);
        if version != VERSION {
            panic!("Unsupported version, expected {:?}", VERSION);
        }

        let main_tag = read_tag(&mut reader);
        if main_tag != *MAIN_TAG {
            panic!("Bad file format, expected {:?} tag", main_tag);
        }
        reader.consume(CHUNK_BYTE_COUNTS_SIZE);

        let size_tag = read_tag(&mut reader);
        if size_tag != *SIZE_TAG {
            panic!("Bad file format, expected {:?} tag", size_tag);
        }
        reader.consume(CHUNK_BYTE_COUNTS_SIZE);
        let x_size = read_int(&mut reader);
        let y_size = read_int(&mut reader);
        let z_size = read_int(&mut reader);

        let xyzi_tag = read_tag(&mut reader);
        if xyzi_tag != *XYZI_TAG {
            panic!("Bad file format, expected {:?} tag", xyzi_tag);
        }
        reader.consume(CHUNK_BYTE_COUNTS_SIZE);
        let voxel_count = read_int(&mut reader);
        let mut xyzis = vec![[0; 4]; voxel_count as usize];
        for xyzi in xyzis.iter_mut() {
            *xyzi = read_xyzi(&mut reader);
        }

        let rgba_tag = read_tag(&mut reader);
        if rgba_tag != *RGBA_TAG {
            panic!("Bad file format, expected {:?} tag", rgba_tag);
        }
        reader.consume(CHUNK_BYTE_COUNTS_SIZE);
        let mut palette = [[0; 4]; PALETTE_RGBA_COUNT];
        for i in 1..PALETTE_RGBA_COUNT {
            palette[i] = read_rgba(&mut reader);
        }
        let vox = Self {
            version: version,
            x_size: x_size,
            y_size: y_size,
            z_size: z_size,
            voxel_count: voxel_count,
            xyzis: xyzis,
            palette: palette,
        };
        return Ok(vox);
    }

    /// Write object data to a file
    pub fn write<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        write_tag(VOX_TAG, &mut writer);
        write_int(self.version, &mut writer);

        write_tag(MAIN_TAG, &mut writer);
        let size_chunk_size = INT_SIZE * 3; // x, y, z
        let xyzi_chunk_size = INT_SIZE // voxel_count
        + self.voxel_count as usize * XYZI_SIZE;
        let rgba_chunk_size = PALETTE_RGBA_COUNT * RGBA_SIZE;
        let main_child_chunks_size = ((TAG_SIZE + CHUNK_BYTE_COUNTS_SIZE) * 3)
            + size_chunk_size
            + xyzi_chunk_size
            + rgba_chunk_size;
        write_int(0, &mut writer); // MAIN has no content
        write_int(main_child_chunks_size as i32, &mut writer);

        write_tag(SIZE_TAG, &mut writer);
        write_int(size_chunk_size as i32, &mut writer);
        write_int(0, &mut writer); // SIZE has no children
        write_int(self.x_size, &mut writer);
        write_int(self.y_size, &mut writer);
        write_int(self.z_size, &mut writer);

        write_tag(XYZI_TAG, &mut writer);
        write_int(xyzi_chunk_size as i32, &mut writer);
        write_int(0, &mut writer); // XYZI has no children
        write_int(self.voxel_count as i32, &mut writer);
        for i in 0..self.voxel_count as usize {
            write_xyzi(&self.xyzis[i], &mut writer);
        }

        write_tag(RGBA_TAG, &mut writer);
        write_int(rgba_chunk_size as i32, &mut writer);
        write_int(0, &mut writer); // RGBA has no children
        for i in 1..PALETTE_RGBA_COUNT {
            write_rgba(&self.palette[i], &mut writer);
        }
        return Ok(());
    }

    /// Return a new `Vox` rotated `rotation` degrees about the z axis
    pub fn rotated(&self, rotation: &Rotation) -> Self {
        let r = match rotation {
            Rotation::R0 => [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
            Rotation::R90 => [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
            Rotation::R180 => [[-1, 0, 0], [0, -1, 0], [0, 0, 1]],
            Rotation::R270 => [[0, 1, 0], [-1, 0, 0], [0, 0, 1]],
        };
        let mut rotated_xyzis = Vec::new();
        let x_offset = self.x_size / 2;
        let y_offset = self.y_size / 2;
        let z_offset = self.z_size / 2;
        let correction = match rotation {
            Rotation::R0 => (0, 0),
            Rotation::R90 => (1, 0),
            Rotation::R180 => (1, 1),
            Rotation::R270 => (0, 1),
        };
        for xyzi in self.xyzis.iter() {
            let x = xyzi[0] as i32 - x_offset;
            let y = xyzi[1] as i32 - y_offset;
            let z = xyzi[2] as i32 - z_offset;
            let rx = r[0][0] * x + r[0][1] * y + r[0][2] * z + x_offset - correction.0; // TODO: Figure out why this correction is needed
            let ry = r[1][0] * x + r[1][1] * y + r[1][2] * z + y_offset - correction.1;
            let rz = r[2][0] * x + r[2][1] * y + r[2][2] * z + z_offset;
            rotated_xyzis.push([rx as u8, ry as u8, rz as u8, xyzi[3]]);
        }
        return Self {
            version: self.version,
            x_size: self.x_size, // Assumes tile is square
            y_size: self.y_size,
            z_size: self.z_size,
            voxel_count: self.voxel_count,
            xyzis: rotated_xyzis,
            palette: self.palette.clone(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_vox() {
        let vox = Vox::open("tests/samples/concrete/config-1-road_turn_low.vox").unwrap();
        vox.write("tests/samples/concrete/vox_test-0-road_turn_low.vox")
            .unwrap();
        assert!(true);
    }

    #[test]
    fn test_rotate_90_z() {
        let vox = Vox::open("tests/samples/concrete/config-1-road_turn_low.vox").unwrap();
        let rotated_vox = vox.rotated(&Rotation::R90);
        rotated_vox.write("tests/samples/concrete/vox_test-1-road_turn_low_r90.vox")
            .unwrap();
        assert!(true);
    }
}
