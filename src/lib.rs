mod utils;

use fixedbitset::FixedBitSet;
use js_sys::Reflect::has;
use wasm_bindgen::prelude::*;

extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

/// Macro to provide `println!(..)` style syntax for `console.log` logging
macro_rules! log {
    ( $( $t:tt)* ) => {
        web_sys::console::log_1(&format!( $($t)* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
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
#[wasm_bindgen]
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
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(idx, match (cell, live_neighbors) {
                    (true, x) if x < 2 => {log!("cell[{}, {}] became false with {} neighbors", row, col, live_neighbors); false},
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => {log!("cell[{}, {}] became false with {} neighbors", row, col, live_neighbors); false},
                    (false, 3) => {log!("cell[{}, {}] became true with {} neighbors", row, col, live_neighbors); true},
                    (otherwise, _) => otherwise
                });
            }
        }
        self.cells = next;
    }

    /// Initializes a new Universe using a 64x64 grid
    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

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
}