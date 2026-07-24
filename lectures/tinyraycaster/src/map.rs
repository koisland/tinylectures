use eyre::{bail, ContextCompat};
use image::{DynamicImage, GenericImageView};
use itertools::Itertools;

use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader},
    marker::PhantomData,
    str::FromStr,
};

use crate::{
    color::Color,
    enemy::{Enemy, EnemyState, EnemyType},
    tiles::{Tile, TileState, TileType},
};

#[derive(Debug, Clone)]
pub enum Texture {
    /// Single color
    Color(Color),
    /// Sprite
    Sprite(DynamicImage),
}

#[derive(Debug, Clone)]
pub struct Textures {
    // Size of texture square length and width.
    pub size: usize,
    // Textures mapped to map tiles and state
    pub tiles: HashMap<TileType, HashMap<TileState, Texture>>,
    // Textures mapped to map entities and state
    pub entities: HashMap<EnemyType, HashMap<EnemyState, Texture>>,
}

impl Textures {
    fn clear(&mut self) {
        self.size = 0;
        self.tiles.clear();
        self.entities.clear();
    }
}

pub struct Map<S: MapState> {
    pub src: String,
    pub w: usize,
    pub h: usize,
    // tile position to id
    pub tile_pos_id_map: HashMap<(usize, usize), usize>,
    pub id_tile_map: BTreeMap<usize, Tile>,
    // enemy position to id
    pub enemy_pos_id_map: HashMap<(usize, usize), usize>,
    pub id_enemy_map: BTreeMap<usize, Enemy>,
    pub textures: Textures,
    _state: PhantomData<S>,
}

// https://zerotomastery.io/blog/rust-typestate-patterns/
#[derive(Debug, Clone, Copy)]
pub struct Init;

#[derive(Debug, Clone, Copy)]
pub struct Uninit;

pub trait MapState {}

impl MapState for Init {}

impl MapState for Uninit {}

impl Map<Uninit> {
    pub fn new() -> Self {
        Map {
            src: String::new(),
            w: 0,
            h: 0,
            textures: Textures {
                size: 0,
                tiles: HashMap::new(),
                entities: HashMap::new(),
            },
            enemy_pos_id_map: HashMap::default(),
            id_enemy_map: BTreeMap::default(),
            tile_pos_id_map: HashMap::default(),
            id_tile_map: BTreeMap::default(),
            _state: PhantomData,
        }
    }

    pub fn with_map(mut self, infile: &str) -> eyre::Result<Self> {
        let fh = BufReader::new(File::open(infile)?);
        self.src.clear();

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

                let eid = self.id_tile_map.len();
                self.tile_pos_id_map.insert((x, h), eid);
                self.id_tile_map.insert(eid, tile);
            }

            self.src.push_str(line);
            // eprintln!("{line} ({w}, {h})");
            // Only check at end. Could also do here and give failing line
            map_w = w;
            // 0-index
            map_h = h + 1;
        }

        if map_w * map_h != self.src.len() {
            bail!("Map does not have uniform length and width");
        }
        self.w = map_w;
        self.h = map_h;
        Ok(self)
    }

    pub fn with_textures(mut self, infile: &str, size: usize) -> eyre::Result<Self> {
        let fh = BufReader::new(File::open(infile)?);
        self.textures.clear();

        let mut texture_cache = HashMap::new();

        for line in fh.lines() {
            let line = line?;
            // Skip comments or header
            if line.starts_with('#') {
                continue;
            }

            let Some((typ, lbl, state, src, idx)) = line.trim().split('\t').collect_tuple() else {
                bail!("Invalid line format for {line}. Expects two columns, tilechar and path/rgb")
            };

            // Check if RGB color or image texture
            let texture = if src.contains(',') {
                let (r, g, b) = src
                    .split(',')
                    .map(|v| v.parse::<u8>())
                    .collect_tuple()
                    .with_context(|| format!("Invalid rgb format for {src}"))?;
                Texture::Color(Color::new(r?, g?, b?, None))
            } else {
                let path = std::path::PathBuf::from(src);
                if !path.exists() {
                    bail!("Invalid src ({src}) for {line}.");
                }

                let img = if let Some(img) = texture_cache.get(src) {
                    img
                } else {
                    texture_cache.insert(src.to_owned(), image::open(src)?);
                    // Already inserted.
                    texture_cache.get(src).unwrap()
                };

                let (w, _) = img.dimensions();
                let w = w as usize;
                let idx: usize = idx.parse()?;

                // Assuming square.
                let ncols = w / size;
                // Convert idx to pixel coordinates
                let div = idx / ncols;
                let rem = idx % ncols;
                let x = (rem * size) as u32;
                let y = (div * size) as u32;
                // Then slice image out of sprite sheet
                let img_slice = img.view(x, y, size as u32, size as u32).to_image();
                Texture::Sprite(DynamicImage::ImageRgba8(img_slice))
            };

            match typ {
                "tile" => {
                    let tiletype: TileType = TileType::from_str(lbl)?;
                    let texture_state = TileState::from_str(state)?;

                    if let Some(tile_src) = self.textures.tiles.get_mut(&tiletype) {
                        tile_src.insert(texture_state, texture);
                    } else {
                        self.textures
                            .tiles
                            .insert(tiletype, HashMap::from_iter([(texture_state, texture)]));
                    }
                }
                "enemy" => {
                    let enemy: EnemyType = EnemyType::from_str(lbl)?;
                    let texture_state = EnemyState::from_str(state)?;
                    if let Some(ety_src) = self.textures.entities.get_mut(&enemy) {
                        ety_src.insert(texture_state, texture);
                    } else {
                        self.textures
                            .entities
                            .insert(enemy, HashMap::from_iter([(texture_state, texture)]));
                    }
                }
                _ => bail!("Invalid type. {typ}"),
            };
        }

        Ok(self)
    }

    pub fn finish(self) -> eyre::Result<Map<Init>> {
        if self.src.is_empty() {
            bail!("Map not initialized")
        }
        if self.textures.entities.is_empty() || self.textures.tiles.is_empty() {
            bail!("Textures (entities or tiles) not initialized")
        }
        Ok(Map {
            src: self.src,
            w: self.w,
            h: self.h,
            textures: self.textures,
            tile_pos_id_map: self.tile_pos_id_map,
            id_tile_map: self.id_tile_map,
            enemy_pos_id_map: self.enemy_pos_id_map,
            id_enemy_map: self.id_enemy_map,
            _state: PhantomData,
        })
    }
}

impl Map<Init> {
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
