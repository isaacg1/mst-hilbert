// Make a minimum spanning tree, from lattice edges with minimum edge weights
// Starting from a random point, depth first search assigning colors via Hilbert tree.

use std::collections::HashSet;
use std::env;

use disjoint::DisjointSet;
use hilbert::point::Point; // Very outdated, upgrade if it becomes deprecated
use image::{ImageBuffer, RgbImage};
use num::{BigUint, One, Zero};
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Edge {
    row: usize,
    col: usize,
    is_right: bool, // If not right, down.
}
struct Location {
    row: usize,
    col: usize,
}
#[derive(PartialEq)]
enum Dir {
    Right,
    Left,
    Up,
    Down,
}
type Color = [u8; 3];
type ColorBase = [u8; 3];
fn color_base_to_color(cb: ColorBase, color_size: u8) -> Color {
    cb.map(|cbc| (cbc as u64 * 255 / (color_size - 1) as u64) as u8)
}
fn make_image(scale: usize, seed: u64) -> RgbImage {
    assert!(scale > 0);
    let size = scale.pow(3);
    let color_size = scale.pow(2) as u8;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut edges: Vec<Edge> = (0..size)
        .flat_map(|row| {
            (0..size).flat_map(move |col| {
                (0..2).map(move |b| Edge {
                    row,
                    col,
                    is_right: b == 0,
                })
            })
        })
        .collect();
    edges.shuffle(&mut rng);
    let mut tree: HashSet<Edge> = HashSet::new();
    let mut vertices = DisjointSet::with_len(scale.pow(6));
    for edge in edges {
        let first = Location {
            row: edge.row,
            col: edge.col,
        };
        let second = if edge.is_right {
            Location {
                row: edge.row,
                col: (edge.col + 1) % size,
            }
        } else {
            Location {
                row: (edge.row + 1) % size,
                col: edge.col,
            }
        };
        let were_distinct =
            vertices.join(first.row * size + first.col, second.row * size + second.col);
        if were_distinct {
            tree.insert(edge);
        }
    }
    let start_vert = Location {
        row: rng.gen_range(0..size),
        col: rng.gen_range(0..size),
    };
    // Trees have no cycles, so no seen list is needed.
    let mut stack = vec![(start_vert, None)];
    let mut img: RgbImage = ImageBuffer::new(size as u32, size as u32);
    let mut index = BigUint::zero();
    while let Some((current_vert, dir)) = stack.pop() {
        // Draw hilbert color
        let bits = color_size.next_power_of_two().checked_ilog2().unwrap_or(8);
        let point = Point::new_from_hilbert_index(0, &index, bits as usize, 3);
        let coord = point.get_coordinates();
        img.put_pixel(
            current_vert.row as u32,
            current_vert.col as u32,
            image::Rgb(color_base_to_color(
                [coord[0] as u8, coord[1] as u8, coord[2] as u8],
                color_size,
            )),
        );
        index += BigUint::one();
        // New verts
        let right_edge = Edge {
            row: current_vert.row,
            col: current_vert.col,
            is_right: true,
        };
        if tree.contains(&right_edge) && dir != Some(Dir::Left) {
            let right_vert = Location {
                row: current_vert.row,
                col: (current_vert.col + 1) % size,
            };
            stack.push((right_vert, Some(Dir::Right)));
        }
        let down_edge = Edge {
            row: current_vert.row,
            col: current_vert.col,
            is_right: false,
        };
        if tree.contains(&down_edge) && dir != Some(Dir::Up) {
            let down_vert = Location {
                row: (current_vert.row + 1) % size,
                col: current_vert.col,
            };
            stack.push((down_vert, Some(Dir::Down)));
        }
        let left_edge = Edge {
            row: current_vert.row,
            col: (current_vert.col + size - 1) % size,
            is_right: true,
        };
        if tree.contains(&left_edge) && dir != Some(Dir::Right) {
            let left_vert = Location {
                row: left_edge.row,
                col: left_edge.col,
            };
            stack.push((left_vert, Some(Dir::Left)));
        }
        let up_edge = Edge {
            row: (current_vert.row + size - 1) % size,
            col: current_vert.col,
            is_right: false,
        };
        if tree.contains(&up_edge) && dir != Some(Dir::Down) {
            let up_vert = Location {
                row: up_edge.row,
                col: up_edge.col,
            };
            stack.push((up_vert, Some(Dir::Up)));
        }
    }
    img
}
fn main() {
    let scale: usize = env::args()
        .nth(1)
        .expect("scale provided")
        .parse()
        .expect("scale is num");
    let seed: u64 = env::args()
        .nth(2)
        .expect("seed provided")
        .parse()
        .expect("seed is num");
    let filename = format!("img-{scale}-{seed}.png");
    println!("Start {filename}");
    let img = make_image(scale, seed);
    img.save(&filename).unwrap();
}
