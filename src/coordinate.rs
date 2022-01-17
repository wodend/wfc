use super::model::Model;

pub struct Coordinates {
    width: usize,
    depth: usize,
    height: usize,
    xyzs: Vec<(usize, usize, usize)>,
    graph: Vec<Vec<(usize, usize)>>,
}

fn edges(width: usize, depth: usize, height: usize, x: usize, y: usize, z: usize) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    if x > 0 {
        let edge_wave = edge_wave(width, depth, x-1, y, z);
        edges.push((edge_wave, Model::LEFT));
    }
    if x < width - 1 {
        let edge_wave = edge_wave(width, depth, x+1, y, z);
        edges.push((edge_wave, Model::RIGHT));
    }
    if y > 0 {
        let edge_wave = edge_wave(width, depth, x, y-1, z);
        edges.push((edge_wave, Model::FRONT));
    }
    if y < depth - 1 {
        let edge_wave = edge_wave(width, depth, x, y+1, z);
        edges.push((edge_wave, Model::BACK));
    }
    if z > 0 {
        let edge_wave = edge_wave(width, depth, x, y, z-1);
        edges.push((edge_wave, Model::DOWN));
    }
    if z < height - 1 {
        let edge_wave = edge_wave(width, depth, x, y, z+1);
        edges.push((edge_wave, Model::UP));
    }
    return edges;
}

fn edge_wave(width: usize, depth: usize, x: usize, y: usize, z: usize) -> usize {
    return (z * width * depth) + (y * depth) + x;
}

impl Coordinates {
    pub fn new(width: usize, depth: usize, height: usize) -> Self {
        let mut xyzs = Vec::new();
        let mut graph = Vec::new();
        for z in 0..height {
            for y in 0..depth {
                for x in 0..width {
                    xyzs.push((x, y, z));
                    graph.push(edges(width, depth, height, x, y, z));
                }
            }
        }
        let coordinates = Self {
            width: width,
            height: height,
            depth: depth,
            xyzs: xyzs,
            graph: graph,
        };
        return coordinates;
    }

    pub fn max_dimension_size(&self) -> usize {
        return std::cmp::max(self.width, std::cmp::max(self.depth, self.height));
    }

    pub fn graph(&self) -> &Vec<Vec<(usize, usize)>> {
        return &self.graph;
    }

    pub fn xyzs(&self) -> &Vec<(usize, usize, usize)> {
        return &self.xyzs;
    }
}