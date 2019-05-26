#[derive(Clone)]
enum Cell {
    Alive,
    Dead,
}

#[derive(Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            width,
            height,
            cells: vec![vec![Cell::Dead; height]; width],
        }
    }
}
