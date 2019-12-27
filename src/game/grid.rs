use super::state::CameraPosition;
use core::fmt;
use ggez::graphics;
use ggez::graphics::Rect;
use ggez::{Context, GameResult};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Debug)]
pub enum Cell {
    Alive,
    Dead,
}

#[derive(Clone, Default)]
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
        vec![
            (x + 1, y),     // Right
            (x - 1, y),     // Left
            (x, y + 1),     // Down
            (x, y - 1),     // Up
            (x + 1, y + 1), // Down Right
            (x + 1, y - 1), // Up Right
            (x - 1, y + 1), // Down Left
            (x - 1, y - 1), // Up Left
        ]
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

    pub fn set_dead(&mut self, (x, y): (isize, isize)) {
        self.cells.remove(&(x, y));
    }

    pub fn update(&mut self, _ctx: &mut Context) -> GameResult {
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

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        zoom_level: f32,
        camera_pos: &CameraPosition,
    ) -> GameResult {
        for ((x, y), cell) in &self.cells {
            match cell {
                Cell::Alive => {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Rect {
                            x: zoom_level * (*x * 30) as f32 - camera_pos.x,
                            y: zoom_level * (*y * 30) as f32 - camera_pos.y,
                            w: 30.0 * zoom_level,
                            h: 30.0 * zoom_level,
                        },
                        [0.3, 0.3, 0.0, 1.0].into(),
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0., y: 0. },))?;
                }
                Cell::Dead => (),
            }
        }

        Ok(())
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
