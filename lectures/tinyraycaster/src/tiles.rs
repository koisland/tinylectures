use std::str::FromStr;

use eyre::bail;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TileType {
    Rock,
    GrayBrick,
    RedBrick,
    Door,
    GreenRock,
    GreenHex,
}

impl TryFrom<char> for TileType {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '0' => TileType::Rock,
            '1' => TileType::GrayBrick,
            '2' => TileType::RedBrick,
            '3' => TileType::Door,
            '4' => TileType::GreenRock,
            '5' => TileType::GreenHex,
            _ => bail!("Invalid tile type. {value}"),
        })
    }
}

impl From<TileType> for char {
    fn from(value: TileType) -> Self {
        match value {
            TileType::Rock => '0',
            TileType::GrayBrick => '1',
            TileType::RedBrick => '2',
            TileType::Door => '3',
            TileType::GreenRock => '4',
            TileType::GreenHex => '5',
        }
    }
}

impl FromStr for TileType {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "rock" => TileType::Rock,
            "gray_brick" => TileType::GrayBrick,
            "red_brick" => TileType::RedBrick,
            "door" => TileType::Door,
            "green_rock" => TileType::GreenRock,
            "green_hex" => TileType::GreenHex,
            _ => bail!("Invalid tile type {s}"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    /// State
    pub state: TileState,
    /// Type of tile
    pub typ: TileType,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TileState {
    Base,
}

impl FromStr for TileState {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "base" => TileState::Base,
            _ => bail!("Invalid tile state {s}"),
        })
    }
}
