// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of cancer.
//
// cancer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cancer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cancer.  If not, see <http://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
use std::f32;

use picto::color::{Rgba, Hsl, RgbHue};
use crate::control::DEC::SIXEL;
use crate::sys::cairo;

#[derive(Debug)]
pub struct Sixel {
	raster: SIXEL::Header,
	origin: (u32, u32),

	grid:     Vec<Vec<cairo::Image>>,
	cell:     (u32, u32),
	limit:    (u32, u32),
	position: (u32, u32),

	colors:     HashMap<u32, (u8, u8, u8, u8), BuildHasherDefault<FnvHasher>>,
	color:      (u8, u8, u8, u8),
	background: (u8, u8, u8, u8),
}

impl Sixel {
	pub fn new(origin: (u32, u32), header: SIXEL::Header, background: &Rgba<f64>, cell: (u32, u32), limit: (u32, u32)) -> Self {
		Sixel {
			raster: header,
			origin: origin,

			grid:     Default::default(),
			cell:     cell,
			limit:    limit,
			position: (0, 0),

			colors:     Default::default(),
			color:      (0, 0, 0, 255),
			background: (
				(background.red   * 255.0) as u8,
				(background.green * 255.0) as u8,
				(background.blue  * 255.0) as u8,
				(background.alpha * 255.0) as u8),
		}
	}

	pub fn origin(&self) -> (u32, u32) {
		self.origin
	}

	pub fn rows(&self) -> usize {
		self.grid.len()
	}

	pub fn into_inner(self) -> Vec<Vec<cairo::Image>> {
		self.grid
	}

	pub fn aspect(&mut self, aspect: (u32, u32)) {
		self.raster.aspect = aspect;
	}

	pub fn enable(&mut self, id: u32) {
		self.color = self.colors.get(&id).unwrap_or(&self.background).clone();
	}

	pub fn define(&mut self, id: u32, color: SIXEL::Color) {
		let color = match color {
			SIXEL::Color::Hsl(h, s, l) =>
				Rgba::from(Hsl::new(RgbHue::from_radians(h as f32 * f32::consts::PI / 180.0),
					s as f32 / 100.0, l as f32 / 100.0)).to_pixel(),

			SIXEL::Color::Rgb(r, g, b) =>
				(r, g, b, 255),

			SIXEL::Color::Rgba(r, g, b, a) =>
				(r, g, b, a),
		};

		self.colors.insert(id, color);
	}

	pub fn start(&mut self) {
		self.position.0 = 0;
	}

	pub fn next(&mut self) {
		self.position.0  = 0;
		self.position.1 += 6 * self.raster.aspect.0;
	}

	pub fn draw(&mut self, times: u32, value: SIXEL::Map) {
		// If the value is empty and the background should not be set, it's just a
		// shift.
		if !self.raster.background && value.is_empty() {
			self.position.0 += times;
			return;
		}

		for _ in 0 .. times {
			// The X within the local grid.
			let x = (self.position.0 / self.cell.0) as usize;

			// Bail out early if the cell is beyond the terminal limit.
			if x as u32 + self.limit.0 >= self.limit.1 {
				break;
			}

			// The X within the image buffer.
			let xo = self.position.0 % self.cell.0;

			for (i, y) in (self.position.1 .. self.position.1 + (6 * self.raster.aspect.0)).enumerate() {
				// The bit index within the sixel map.
				let bit = (i as u32 / self.raster.aspect.0) as u8;

				// The Y within the image buffer.
				let yo = y as u32 % self.cell.1;

				// The Y within the grid.
				let y = (y / self.cell.1) as usize;

				// If the grid doesn't have enough rows, extend it.
				while y >= self.grid.len() {
					self.grid.push(Vec::new());
				}

				// If the grid doesn't have enough columns, extend it.
				while x >= self.grid[y].len() {
					self.grid[y].push(cairo::Image::new(self.cell.0, self.cell.1));
				}

				// If the bit is enabled, set it.
				if value.get(bit) {
					self.grid[y][x].set(xo, yo, &self.color);
				}
				// If disabled bits should set the background color, do so.
				else if self.raster.background {
					self.grid[y][x].set(xo, yo, &self.background);
				}
			}

			self.position.0 += 1;
		}
	}

	pub fn handle(&mut self, item: &SIXEL::T) {
		match *item {
			SIXEL::Raster { aspect, .. } => {
				self.aspect(aspect);
			}

			SIXEL::Enable(id) => {
				self.enable(id);
			}

			SIXEL::Define(id, color) => {
				self.define(id, color);
			}

			SIXEL::Value(value) => {
				self.draw(1, value);
			}

			SIXEL::Repeat(times, value) => {
				self.draw(times, value);
			}

			SIXEL::CarriageReturn => {
				self.start();
			}

			SIXEL::LineFeed => {
				self.next();
			}
		}
	}
}
