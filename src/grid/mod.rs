use coord::Coord;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub enum CellState {
    Alive,
    Dead,
}

#[derive(Clone)]
pub struct Grid {
    cell_states: HashMap<Coord<isize>, CellState>,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cell_states: HashMap::new(),
        }
    }

    fn neighbors(&self, coord: Coord<isize>) -> Vec<(Coord<isize>, CellState)> {
        coord
            .neighbors()
            .iter()
            .map(|&coord| {
                (
                    coord,
                    *self.cell_states.get(&coord).unwrap_or(&CellState::Dead),
                )
            })
            .collect()
    }

    pub fn cells(&self) -> impl Iterator<Item = (&Coord<isize>, &CellState)> {
        self.cell_states.iter()
    }

    pub fn living_neighbors(&self, coord: Coord<isize>) -> Vec<(Coord<isize>, CellState)> {
        self.neighbors(coord)
            .into_iter()
            .filter(|(coord, _state)| match self.cell_states.get(&coord) {
                Some(CellState::Alive) => true,
                _ => false,
            })
            .collect()
    }

    pub fn get(&self, coord: Coord<isize>) -> Option<&CellState> {
        self.cell_states.get(&coord)
    }

    pub fn set_alive(&mut self, cell: Coord<isize>) {
        self.cell_states.insert(cell, CellState::Alive);

        // add surrounding cells to grid to trigger updates on them
        self.neighbors(cell)
            .into_iter()
            .for_each(|(coord, _state)| match self.cell_states.entry(coord) {
                Entry::Occupied(_) => (),
                Entry::Vacant(neighbor) => {
                    neighbor.insert(CellState::Dead);
                }
            });
    }

    pub fn set_dead(&mut self, cell: Coord<isize>) {
        self.cell_states.remove(&cell);
    }

    pub fn from_file(file_name: &str) -> Self {
        let file = Path::new(file_name);
        match file.extension().and_then(OsStr::to_str) {
            Some("txt") => Self::from_txt(file).unwrap_or_default(),
            Some("rle") => Self::from_rle(file).unwrap_or_default(),
            invalid => panic!("invalid file extension {:?}", invalid),
        }
    }

    fn from_txt(file_name: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut grid = Grid::new();
        let f = File::open(file_name)?;
        let f = BufReader::new(f);
        f.lines()
            .filter_map(|l| l.ok())
            .enumerate()
            .for_each(|(row, line)| {
                line.chars().enumerate().for_each(|(col, c)| {
                    if let 'O' = c {
                        grid.set_alive(Coord {
                            x: col as isize,
                            y: row as isize,
                        })
                    }
                })
            });
        Ok(grid)
    }

    fn from_rle(file_name: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        use regex::Regex;

        let mut grid = Grid::new();
        let f = File::open(file_name)?;
        let f = BufReader::new(f);

        let mut lines = f
            .lines()
            .filter_map(|l| l.ok())
            .skip_while(|line| line.starts_with('#'));

        let size_line = lines.next().ok_or("invalid file format")?;
        let (_x, _y): (usize, usize) = {
            let re = Regex::new(r"x\s*=\s*(?P<x>\d+)\s*,\s*y\s*=\s*(?P<y>\d+)")?;
            let captures = re.captures(&size_line).ok_or("could not capture")?;
            (
                captures
                    .get(1)
                    .ok_or("did not capture x")?
                    .as_str()
                    .parse()?,
                captures
                    .get(2)
                    .ok_or("did not capture y")?
                    .as_str()
                    .parse()?,
            )
        };

        // Concatenate remaining lines into one long string
        let content = lines.fold("".into(), |prev, line| format!("{}{}", prev, line));

        let content_lines = content.split('$').collect::<Vec<_>>();

        let tag_regex = Regex::new(r"(\d*)([b|o])")?;
        for (row_idx, content_line) in content_lines.iter().enumerate() {
            let mut col = 0;
            for capture in tag_regex.captures_iter(content_line) {
                let (repetitions, cell_state) = {
                    (
                        capture[1].parse::<usize>().unwrap_or(1),
                        match capture[2].into() {
                            "b" => CellState::Dead,
                            "o" => CellState::Alive,
                            _ => {
                                return Err(regex::Error::Syntax(
                                    "could not capture cell state".into(),
                                )
                                .into())
                            }
                        },
                    )
                };
                if let CellState::Alive = cell_state {
                    for col_idx in col..col + repetitions {
                        grid.set_alive(Coord {
                            x: col_idx as isize,
                            y: row_idx as isize,
                        });
                    }
                }
                col += repetitions;
            }
        }
        Ok(grid)
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (left, right, up, down) = self.cell_states.keys().fold(
            (
                isize::max_value(),
                isize::min_value(),
                isize::max_value(),
                isize::min_value(),
            ),
            |(min_x, max_x, min_y, max_y), &cell| {
                let (x, y) = (cell.x, cell.y);
                (
                    if x < min_x { x } else { min_x },
                    if x > max_x { x } else { max_x },
                    if y < min_y { y } else { min_y },
                    if y > max_y { y } else { max_y },
                )
            },
        );

        for row in up..=down {
            writeln!(
                f,
                "{} asd",
                (left..=right).fold("".to_string(), |prev, col| {
                    match self.cell_states.get(&Coord { x: col, y: row }) {
                        Some(CellState::Alive) => format!("{}{}", prev, "X"),
                        _ => format!("{}{}", prev, "O"),
                    }
                })
            )?;
        }
        Ok(())
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cell_states: HashMap::new(),
        }
    }
}
