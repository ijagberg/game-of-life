use core::fmt;
use ggez::event;
use ggez::{Context, GameResult};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub enum Cell {
    Alive,
    Dead,
}

#[derive(Clone)]
pub struct Grid {
    pub cells: HashMap<(isize, isize), Cell>,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cells: HashMap::new(),
        }
    }

    fn neighbors(&self, (x, y): (isize, isize)) -> Vec<(isize, isize)> {
        vec![(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
    }

    pub fn living_neighbors(&self, (x, y): (isize, isize)) -> usize {
        self.neighbors((x, y))
            .iter()
            .filter(|(x, y)| match self.cells.get(&(*x, *y)) {
                Some(Cell::Alive) => true,
                _ => false,
            })
            .count()
    }

    pub fn set_alive(&mut self, (x, y): (isize, isize)) {
        self.cells.insert((x, y), Cell::Alive);
        self.neighbors((x, y))
            .iter()
            .for_each(|&(x1, y1)| match self.cells.entry((x1, y1)) {
                Entry::Occupied(_) => (),
                Entry::Vacant(neighbor) => {
                    neighbor.insert(Cell::Dead);
                }
            });
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (left, right, up, down) = self.cells.keys().fold(
            (
                isize::max_value(),
                isize::min_value(),
                isize::max_value(),
                isize::min_value(),
            ),
            |(min_x, max_x, min_y, max_y), &(x, y)| {
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
                    match self.cells.get(&(col, row)) {
                        Some(Cell::Alive) => format!("{}{}", prev, "X"),
                        _ => format!("{}{}", prev, "O"),
                    }
                })
            )?;
        }
        Ok(())
    }
}

impl event::EventHandler for Grid {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let mut next_cells = Grid::new();
        for (&(x, y), cell) in &self.cells {
            match cell {
                Cell::Alive => {
                    // Check if this cell will stay alive in the next state
                    let living_neighbors = self.living_neighbors((x, y));

                    // Set the cell to alive in the next state, and add all its neighbors
                    if living_neighbors == 2 || living_neighbors == 3 {
                        next_cells.set_alive((x, y));
                    }
                }
                Cell::Dead => {
                    // Check if this cell is born in the next state
                    let living_neighbors = self.living_neighbors((x, y));
                    if living_neighbors == 3 {
                        next_cells.set_alive((x, y));
                    }
                }
            }
        }
        self.cells = next_cells.cells;
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
}
