use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
pub fn get_welcome_message() -> String {
    "Welcome to Palmietopia!".to_string()
}

/// Terrain types for map tiles
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Terrain {
    Grassland,
    Forest,
    Mountain,
    Water,
    Desert,
}

impl Terrain {
    /// Get a random terrain type
    fn random() -> Self {
        use getrandom::getrandom;
        let mut buf = [0u8; 1];
        getrandom(&mut buf).unwrap();
        match buf[0] % 5 {
            0 => Terrain::Grassland,
            1 => Terrain::Forest,
            2 => Terrain::Mountain,
            3 => Terrain::Water,
            _ => Terrain::Desert,
        }
    }
}

/// A single hex tile with axial coordinates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub q: i32,
    pub r: i32,
    pub terrain: Terrain,
}

/// The game map containing all tiles
#[derive(Serialize, Deserialize)]
pub struct GameMap {
    pub tiles: Vec<Tile>,
    pub radius: u32,
}

impl GameMap {
    /// Generate a hexagonal map with the given radius
    pub fn generate(radius: u32) -> Self {
        let mut tiles = Vec::new();
        let r = radius as i32;

        // Generate hexagonal map using axial coordinates
        for q in -r..=r {
            let r1 = (-r).max(-q - r);
            let r2 = r.min(-q + r);
            for r_coord in r1..=r2 {
                tiles.push(Tile {
                    q,
                    r: r_coord,
                    terrain: Terrain::random(),
                });
            }
        }

        GameMap { tiles, radius }
    }
}

/// Generate a tiny map (radius 2, 19 tiles)
#[wasm_bindgen]
pub fn generate_tiny_map() -> String {
    let map = GameMap::generate(2);
    serde_json::to_string(&map).unwrap()
}
