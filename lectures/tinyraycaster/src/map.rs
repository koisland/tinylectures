use eyre::bail;

use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{
    enemy::Enemy,
    tiles::{Tile, TileState, TileType},
};

pub struct Map {
    pub src: String,
    pub w: usize,
    pub h: usize,
    // tile position to id
    pub tile_pos_id_map: HashMap<(usize, usize), usize>,
    pub id_tile_map: BTreeMap<usize, Tile>,
    // enemy position to id
    pub enemy_pos_id_map: HashMap<(usize, usize), usize>,
    pub id_enemy_map: BTreeMap<usize, Enemy>,
}

impl Map {
    pub fn new(infile: &str) -> eyre::Result<Self> {
        let fh = BufReader::new(File::open(infile)?);
        let mut map = Map {
            src: String::new(),
            w: 0,
            h: 0,
            enemy_pos_id_map: HashMap::default(),
            id_enemy_map: BTreeMap::default(),
            tile_pos_id_map: HashMap::default(),
            id_tile_map: BTreeMap::default(),
        };

        let mut map_w: usize = 0;
        let mut map_h: usize = 0;
        for (h, line) in fh.lines().enumerate() {
            let line = line?;
            let line = line.trim();
            let w = line.len();

            // Add tiles as entities
            for (x, tile) in line.chars().enumerate() {
                let Ok(tile_typ) = TryInto::<TileType>::try_into(tile) else {
                    continue;
                };
                let tile = Tile {
                    state: TileState::Base,
                    typ: tile_typ,
                };

                let eid = map.id_tile_map.len();
                map.tile_pos_id_map.insert((x, h), eid);
                map.id_tile_map.insert(eid, tile);
            }

            map.src.push_str(line);
            // eprintln!("{line} ({w}, {h})");
            // Only check at end. Could also do here and give failing line
            map_w = w;
            // 0-index
            map_h = h + 1;
        }

        if map_w * map_h != map.src.len() {
            bail!("Map does not have uniform length and width");
        }
        map.w = map_w;
        map.h = map_h;
        Ok(map)
    }

    pub fn tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tile_pos_id_map
            .get(&(x, y))
            .and_then(|id| self.id_tile_map.get(id))
    }

    pub fn tiles(&self) -> impl Iterator<Item = (usize, usize, Option<&Tile>)> {
        (0..self.h).flat_map(move |y| (0..self.w).map(move |x| (x, y, self.tile(x, y))))
    }

    pub fn spawn_enemy(&mut self, enemy: Enemy) {
        let eid = self.id_enemy_map.len();
        // TODO: This should change
        self.enemy_pos_id_map
            .insert((enemy.x as usize, enemy.y as usize), eid);
        self.id_enemy_map.insert(eid, enemy);
    }

    // pub fn get_entity_by_id(&mut self, id: usize) -> Option<&mut Box<dyn Entity + 'static>> {
    //     self.entities.get_mut(&id)
    // }
}
