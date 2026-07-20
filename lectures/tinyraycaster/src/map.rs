use eyre::{bail, ContextCompat};
use itertools::Itertools;
use rand::prelude::*;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    marker::PhantomData,
    path::PathBuf,
};

use crate::color::Color;

#[derive(Debug, Clone)]
pub enum Texture {
    #[allow(dead_code)]
    Color(Color),
    #[allow(dead_code)]
    Sprite(PathBuf),
}

#[derive(Debug)]
pub struct Tile<'src> {
    pub(crate) icon: char,
    pub(crate) texture: &'src Texture,
}

#[derive(Debug, Clone)]
pub struct Map<S: MapState> {
    pub(crate) src: String,
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) tilemap: HashMap<char, Texture>,
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
            tilemap: HashMap::new(),
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

    pub fn with_textures(mut self, infile: &str) -> eyre::Result<Self> {
        let fh = BufReader::new(File::open(infile)?);
        self.tilemap.clear();

        for line in fh.lines() {
            let line = line?;

            let Some((tile, src)) = line.trim().split('\t').collect_tuple() else {
                bail!("Invalid line format for {line}. Expects two columns, tilechar and path/rgb")
            };
            let tile = tile
                .chars()
                .next()
                .with_context(|| format!("No character in line, {line}"))?;

            // Check if path or texture
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
                Texture::Sprite(path)
            };

            if let Some(tile_src) = self.tilemap.get_mut(&tile) {
                *tile_src = texture
            } else {
                self.tilemap.insert(tile.to_owned(), texture);
            }
        }

        Ok(self)
    }

    fn get_random_tilecolors(&self, seed: Option<u64>) -> HashMap<char, Texture> {
        let mut rng = seed
            .map(StdRng::seed_from_u64)
            .unwrap_or_else(|| rand::make_rng());

        self.src
            .chars()
            .unique()
            .map(|v| {
                let r: u8 = rng.random();
                let g: u8 = rng.random();
                let b: u8 = rng.random();
                (v, Texture::Color(Color::new(r, g, b, None)))
            })
            .collect()
    }

    pub fn finish(self) -> eyre::Result<Map<Init>> {
        if self.src.is_empty() {
            bail!("Map not initialized")
        }
        let tilemap = if self.tilemap.is_empty() {
            self.get_random_tilecolors(None)
        } else {
            self.tilemap
        };
        Ok(Map {
            src: self.src,
            w: self.w,
            h: self.h,
            tilemap,
            _state: PhantomData,
        })
    }
}

impl Map<Init> {
    #[inline]
    pub fn tile<'src>(&'src self, x: usize, y: usize) -> Option<Tile<'src>> {
        let idx = x + y * self.w;
        self.src
            .get(idx..idx + 1)
            .and_then(|tile| tile.chars().next())
            .and_then(|tile| {
                self.tilemap.get(&tile).map(|texture| Tile {
                    icon: tile,
                    texture,
                })
            })
    }

    pub fn tiles<'src>(&'src self) -> impl Iterator<Item = (usize, usize, Option<Tile<'src>>)> {
        (0..self.h).flat_map(move |y| (0..self.w).map(move |x| (x, y, self.tile(x, y))))
    }
}
