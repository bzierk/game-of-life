mod utils;

use fixedbitset::FixedBitSet;
use wasm_bindgen::prelude::*;
use web_sys::console;

extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

/// Macro to provide `println!(..)` style syntax for `console.log` logging
macro_rules! log {
    ( $( $t:tt)* ) => {
        console::log_1(&format!( $($t)* ).into());
    }
}

macro_rules! debug {
    ( $( $t:tt)* ) => {
        console::debug_1(&format!( $($t)* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        // console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    // fn drop(&mut self) {
    //     console::time_end_with_label(self.name);
    // }
    fn drop(&mut self) {}
}

// #[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

// #[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {
    /// Get the list of cells in a Universe
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a Universe by passing row and column of each cell as an array
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
}

// Separate impl for things we want exposed to JS
// #[wasm_bindgen]
impl Universe {
    /// Takes a row and column in the Universe grid and returns the cell's index in
    /// a linear array
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    /// Takes a row and column in the Universe grid and returns the number of `Alive` neighbors
    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        // `self.height - 1` is added and mod height is used to effectively subtract one. This is
        // to avoid integer underflow and the special handling required to deal with the edge of
        // the grid
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                // This is the cell in question, don't count as neighbor
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                // Modulo handles wrapping on edge of grid
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                // Add value of cell. This works since we use `Alive = 1` and `Dead = 0`
                count += self.cells[idx] as u8;
            }
        }

        count
    }
    /// Iterates over the Universe determining the state of each cell in the next tick
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        let mut next = {
            let _timer = Timer::new("Allocate next cells");
            self.cells.clone()
        };

        {
            let _timer = Timer::new("New generation");
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    next.set(idx, match (cell, live_neighbors) {
                        (true, x) if x < 2 => {
                            // debug!("cell[{}, {}] became false with {} neighbors", row, col, live_neighbors);
                            false
                        }
                        (true, 2) | (true, 3) => true,
                        (true, x) if x > 3 => {
                            // debug!("cell[{}, {}] became false with {} neighbors", row, col, live_neighbors);
                            false
                        }
                        (false, 3) => {
                            // debug!("cell[{}, {}] became true with {} neighbors", row, col, live_neighbors);
                            true
                        }
                        (otherwise, _) => otherwise
                    });
                }
            }
        }
        let _timer = Timer::new("Free old cells");
        self.cells = next;
    }

    /// Initializes a new Universe using a 64x64 grid
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 128;
        let height = 128;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 128;
        let height = 128;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, rand::random::<f32>() % 1.0 < 0.5);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn empty() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, false);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    /// Set width of the universe and reset all cells to `Dead`
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let size = (width * self.height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, false);
        }
        self.cells = cells;
    }

    /// Set height of the universe and reset all cells to `Dead`
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let size = (height * self.width) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, false);
        }
        self.cells = cells;
    }

    pub fn toggle_cell(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        self.cells.toggle(idx);
    }

    pub fn spawn_glider(&mut self, row: u32, col: u32) {
        let cells = [(row - 1, col + 1), (row, col - 1), (row, col + 1), (row + 1, col), (row + 1, col + 1)];
        self.set_cells(&cells);
    }

    pub fn spawn_pulsar(&mut self, row: u32, col: u32) {
        let cells = [
            (row - 6, col - 4), (row - 6, col - 3), (row - 6, col - 2), (row - 6, col + 2), (row - 6, col + 3), (row - 6, col + 4),
            (row - 4, col - 6), (row - 4, col - 1), (row - 4, col + 1), (row - 4, col + 6),
            (row - 3, col - 6), (row - 3, col - 1), (row - 3, col + 1), (row - 3, col + 6),
            (row - 2, col - 6), (row - 2, col - 1), (row - 2, col + 1), (row - 2, col + 6),
            (row - 1, col - 4), (row - 1, col - 3), (row - 1, col - 2), (row - 1, col + 2), (row - 1, col + 3), (row - 1, col + 4),
            (row + 1, col - 4), (row + 1, col - 3), (row + 1, col - 2), (row + 1, col + 2), (row + 1, col + 3), (row + 1, col + 4),
            (row + 2, col - 6), (row + 2, col - 1), (row + 2, col + 1), (row + 2, col + 6),
            (row + 3, col - 6), (row + 3, col - 1), (row + 3, col + 1), (row + 3, col + 6),
            (row + 4, col - 6), (row + 4, col - 1), (row + 4, col + 1), (row + 4, col + 6),
            (row + 6, col - 4), (row + 6, col - 3), (row + 6, col - 2), (row + 6, col + 2), (row + 6, col + 3), (row + 6, col + 4),
        ];
        self.set_cells(&cells);
    }
}