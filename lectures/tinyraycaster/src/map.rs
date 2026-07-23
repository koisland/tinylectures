use eyre::{bail, ContextCompat};
use image::{DynamicImage, GenericImageView};
use itertools::Itertools;
use rand::prelude::*;

use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader},
    marker::PhantomData,
};

use crate::{color::Color, entity::Entity};

#[derive(Debug, Clone)]
pub enum Texture {
    /// Single color
    Color(Color),
    /// Sprite
    Sprite(DynamicImage),
}

#[derive(Debug)]
pub struct Tile<'src> {
    pub(crate) icon: char,
    pub(crate) texture: &'src Texture,
}

#[derive(Debug, Clone)]
pub struct Textures {
    // Size of texture square length and width.
    size: usize,
    // Textures mapped to map character
    tilemap: HashMap<char, Texture>,
}

impl Textures {
    fn clear(&mut self) {
        self.size = 0;
        self.tilemap.clear();
    }
}

pub struct Map<S: MapState> {
    pub(crate) src: String,
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) entities: BTreeMap<usize, Box<dyn Entity>>,
    pub(crate) textures: Textures,
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
                tilemap: HashMap::new(),
            },
            entities: BTreeMap::default(),
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

            let Some((tile, src, idx)) = line.trim().split('\t').collect_tuple() else {
                bail!("Invalid line format for {line}. Expects two columns, tilechar and path/rgb")
            };
            let tile = tile
                .chars()
                .next()
                .with_context(|| format!("No character in line, {line}"))?;

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
                    bail!("Invalid src ({src}) for tile, {tile}.");
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

            if let Some(tile_src) = self.textures.tilemap.get_mut(&tile) {
                *tile_src = texture
            } else {
                self.textures.tilemap.insert(tile.to_owned(), texture);
            }
        }

        Ok(self)
    }

    fn get_random_tilecolors(&self, seed: Option<u64>) -> Textures {
        let mut rng = seed
            .map(StdRng::seed_from_u64)
            .unwrap_or_else(|| rand::make_rng());
        Textures {
            size: 0,
            tilemap: self
                .src
                .chars()
                .unique()
                .map(|v| {
                    let r: u8 = rng.random();
                    let g: u8 = rng.random();
                    let b: u8 = rng.random();
                    (v, Texture::Color(Color::new(r, g, b, None)))
                })
                .collect(),
        }
    }

    pub fn finish(self) -> eyre::Result<Map<Init>> {
        if self.src.is_empty() {
            bail!("Map not initialized")
        }
        let textures = if self.textures.tilemap.is_empty() {
            self.get_random_tilecolors(None)
        } else {
            self.textures
        };
        Ok(Map {
            src: self.src,
            w: self.w,
            h: self.h,
            textures,
            entities: BTreeMap::new(),
            _state: PhantomData,
        })
    }
}

impl Map<Init> {
    pub fn tile<'src>(&'src self, x: usize, y: usize) -> Option<Tile<'src>> {
        let idx = x + y * self.w;
        self.src
            .get(idx..idx + 1)
            .and_then(|tile| tile.chars().next())
            .and_then(|tile| {
                self.textures.tilemap.get(&tile).map(|texture| Tile {
                    icon: tile,
                    texture,
                })
            })
    }

    pub fn tiles<'src>(&'src self) -> impl Iterator<Item = (usize, usize, Option<Tile<'src>>)> {
        (0..self.h).flat_map(move |y| (0..self.w).map(move |x| (x, y, self.tile(x, y))))
    }

    pub fn spawn_entity(&mut self, entity: impl Entity + 'static) {
        let eid = self.entities.len();
        self.entities.insert(eid, Box::new(entity));
    }

    // pub fn get_entity_by_id(&mut self, id: usize) -> Option<&mut Box<dyn Entity + 'static>> {
    //     self.entities.get_mut(&id)
    // }
}
