use std::{collections::VecDeque, error, fmt, fs::write, ops::Range};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

const BLANK: i32 = 0;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

type Tile = i32;
type Coordinate = (usize, usize);
type Trace = (Coordinate, Coordinate);

/// The type of game board data contains:
///     1.the size of the board
///     2.the two-dimensional array of the tiles value,
///       the real value = 0 << (the stored value)
///     3.the scores of current situation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    size: usize,
    tiles: Vec<Vec<Tile>>,
    score: u32,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::from("Situation:\n");
        for x in 0..self.size {
            for y in 0..self.size {
                s.push_str(&format!("{:5}", self.tiles[x][y]));
            }
            s.push_str("\n");
        }
        write!(f, "{}", s)
    }
}

impl Board {
    pub fn new(size: usize, tiles: Option<Vec<Tile>>, score: u32) -> Self {
        Board {
            size,
            tiles: match tiles {
                Some(data) => data.chunks(size).map(|x| x.to_vec()).collect(),
                None => vec![vec![BLANK; size]; size],
            },
            score,
        }
    }

    pub fn get(&self, pos: &Coordinate) -> Option<&Tile> {
        self.tiles.get(pos.0)?.get(pos.1)
    }

    pub fn set(&mut self, pos: &Coordinate, value: Tile) {
        assert!(value >= 0, "The tile value must be greater then 0.");
        self.tiles[pos.0][pos.1] = value;
    }

    /// Generates multiple values randomly in the given range.
    pub fn generate(&mut self, times: u32, scope: Range<i32>) {
        let mut rng = thread_rng();
        for _ in 0..times {
            loop {
                let x = rng.gen_range(0..self.size);
                let y = rng.gen_range(0..self.size);
                if self.tiles[x][y] == 0 {
                    self.tiles[x][y] = rng.gen_range(scope.clone());
                    break;
                }
            }
        }
    }

    fn next(&self, pos: &Coordinate, direction: &Direction) -> Option<Coordinate> {
        let cell = match direction {
            Direction::Up if pos.0 >= 1 => (pos.0 - 1, pos.1),
            Direction::Down if pos.0 + 1 < self.size => (pos.0 + 1, pos.1),
            Direction::Left if pos.1 >= 1 => (pos.0, pos.1 - 1),
            Direction::Right if pos.1 + 1 < self.size => (pos.0, pos.1 + 1),
            _ => return None,
        };
        Some(cell)
    }

    /// Saves the board data formatted as json to the given path.
    pub fn save(&self, path: &str) -> Result<(), Box<dyn error::Error>> {
        let json = serde_json::to_string(&self)?;
        write(path, json)?;
        log::debug!("Saved to file: {}", path);
        Ok(())
    }
}


pub struct Core;

impl Core {
    pub fn new() -> Self {
        Core {}
    }

    pub fn is_game_over(&self, board: &Board) -> bool {
        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_back((0, 0));
        while !queue.is_empty() {
            let current = queue.pop_front().unwrap();
            if let Some(&BLANK) = board.get(&current) {
                return false;
            }
            for d in [Direction::Right, Direction::Down].iter() {
                if let Some(next) = board.next(&current, d) {
                    if board.get(&current) == board.get(&next) {
                        return false;
                    }
                    queue.push_back(next);
                }
            }
        }
        true
    }

    /// Moves by some direction and returns the traces of all moved tiles.
    pub fn shift(&self, board: &mut Board, direction: &Direction) -> Vec<Trace> {
        let mut traces: Vec<Trace> = Vec::new();

        match direction {
            Direction::Right => {
                for x in 0..board.size {
                    for y in (0..board.size).rev() {
                        let mut t = self.do_shift(board, &(x, y), direction);
                        traces.append(&mut t);
                    }
                }
            }
            Direction::Down => {
                for y in 0..board.size {
                    for x in (0..board.size).rev() {
                        let mut t = self.do_shift(board, &(x, y), direction);
                        traces.append(&mut t);
                    }
                }
            }
            Direction::Up => {
                for y in 0..board.size {
                    for x in 0..board.size {
                        let mut t = self.do_shift(board, &(x, y), direction);
                        traces.append(&mut t);
                    }
                }
            }
            Direction::Left => {
                for x in 0..board.size {
                    for y in 0..board.size {
                        let mut t = self.do_shift(board, &(x, y), direction);
                        traces.append(&mut t);
                    }
                }
            }
        }
        traces
    }

    fn do_shift(&self, board: &mut Board, tile: &Coordinate, direction: &Direction) -> Vec<Trace> {
        let mut tile = tile.clone();
        let mut no_swapped = false;
        let mut traces: Vec<Trace> = Vec::new();
        loop {
            if let Some(next_tile) = board.next(&tile, &direction) {
                // debug!("next: {:?} {:?}", next_tile, self.get(&next_tile));

                let tile_val = *board.get(&tile).unwrap();
                let next_tile_val = *board.get(&next_tile).unwrap();

                if tile_val != BLANK {
                    if next_tile_val == BLANK {
                        board.set(&next_tile, tile_val);
                        board.set(&tile, BLANK);
                        if no_swapped {
                            let last_tile = traces.pop().unwrap().0;
                            traces.push((last_tile, next_tile));
                        } else {
                            traces.push((tile, next_tile))
                        }
                        no_swapped = true;
                    } else if tile_val == next_tile_val {
                        board.set(&next_tile, tile_val + 1);
                        board.set(&tile, BLANK);
                        board.score += 1 << (tile_val + 1);
                        traces.push((tile, next_tile));
                        no_swapped = false;
                    }
                }

                tile = next_tile;
            } else {
                return traces;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    #![cfg_attr(rustfmt, rustfmt_skip)]

    use super::*;

    #[test]
    fn test_shift() {
        let core = Core::new();
        let mut board = Board::new(
            4,
            Some(
                vec![
                    1, 0, 0, 0,
                    1, 0, 1, 1,
                    1, 0, 1, 2,
                    0, 0, 0, 0,
                ]
            ),
            0
        );

        core.shift(&mut board, &Direction::Right);
        assert_eq!(
            board.tiles.concat(),
            [
                [0, 0, 0, 1],
                [0, 0, 1, 2],
                [0, 0, 0, 3],
                [0, 0, 0, 0],
            ].concat()
        );

        core.shift(&mut board, &Direction::Down);
        assert_eq!(
            board.tiles.concat(),
            [
                [0, 0, 0, 0],
                [0, 0, 0, 1],
                [0, 0, 0, 2],
                [0, 0, 1, 3],
            ].concat()
        );
    }

    #[test]
    fn test_game_over() {
        let core = Core::new();
        let board = Board::new(
            4,
            Some(
                vec![
                    1, 2, 3, 4,
                    4, 3, 2, 1,
                    1, 2, 3, 4,
                    4, 3, 2, 1,
                ]
            ),
            0
        );
        assert_eq!(core.is_game_over(&board), true);

        let board = Board::new(
            4,
            Some(
                vec![
                    1, 2, 3, 4,
                    4, 3, 2, 1,
                    1, 2, 3, 4,
                    4, 3, 2, 4,
                ]
            ),
            0
        );
        assert_eq!(core.is_game_over(&board), false);
    }

}
