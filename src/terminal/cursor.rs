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

use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::convert::TryFrom;
use std::i32;

use picto::color::Rgba;
use crate::style::Style;
use crate::config::Config;
use crate::config::style::Shape;
use crate::control::DEC;
use crate::terminal::cell;

#[derive(PartialEq, Clone, Debug)]
pub struct Cursor {
	x: u32,
	y: u32,

	width:  u32,
	height: u32,

	pub state:  State,
	pub scroll: (u32, u32),
	pub style:  Rc<Style>,
	pub bright: Option<u8>,

	pub charsets: [DEC::Charset; 4],
	pub charset:  u8,

	pub foreground: Rgba<f64>,
	pub background: Rgba<f64>,
	pub shape:      Shape,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Travel {
	Position(Option<u32>, Option<u32>),

	Up(u32),
	Down(u32),
	Left(u32),
	Right(u32),
}

bitflags! {
	pub struct State: u8 {
		const BLINK   = 1 << 0;
		const VISIBLE = 1 << 1;
		const WRAP    = 1 << 2;
		const ORIGIN  = 1 << 3;
	}
}

impl Default for State {
	fn default() -> Self {
		VISIBLE
	}
}

pub use self::Travel::*;

#[allow(dead_code)]
impl Cursor {
	/// Create a new cursor.
	pub fn new(config: Arc<Config>, width: u32, height: u32) -> Self {
		let mut state = State::default();
		if config.style().cursor().blink() {
			state.insert(BLINK);
		}

		Cursor {
			x: 0,
			y: 0,

			width:  width,
			height: height,

			state:  state,
			scroll: (0, height - 1),
			style:  Default::default(),
			bright: None,

			charsets: [DEC::charset::ISO::Latin2.into(); 4],
			charset:  0,

			foreground: *config.style().cursor().foreground(),
			background: *config.style().cursor().background(),
			shape:      config.style().cursor().shape(),
		}
	}

	/// Adapt the cursor to a new size.
	pub fn resize(&mut self, width: u32, height: u32) {
		if self.scroll == (0, self.height - 1) {
			self.scroll = (0, height - 1);
		}

		if self.x >= width {
			self.x = width - 1;
		}

		if self.y >= height {
			self.y = height - 1;
		}

		self.width  = width;
		self.height = height;
	}

	/// Get the position.
	pub fn position(&self) -> (u32, u32) {
		(self.x, self.y)
	}

	/// Get the X.
	pub fn x(&self) -> u32 {
		self.x
	}

	/// Get the Y.
	pub fn y(&self) -> u32 {
		self.y
	}

	/// Get the current style.
	pub fn style(&self) -> &Rc<Style> {
		&self.style
	}

	/// Get the cursor foreground color.
	pub fn foreground(&self) -> &Rgba<f64> {
		&self.foreground
	}

	/// Get the cursor background color.
	pub fn background(&self) -> &Rgba<f64> {
		&self.background
	}

	/// Get the cursor shape.
	pub fn shape(&self) -> Shape {
		self.shape
	}

	/// Get whether the cursor should blink or not.
	pub fn blink(&self) -> bool {
		self.state.contains(BLINK)
	}

	/// Check if the cursor is visible.
	pub fn is_visible(&self) -> bool {
		self.state.contains(VISIBLE)
	}

	/// Check if the next insertion should wrap.
	pub fn wrap(&self) -> bool {
		self.state.contains(WRAP)
	}

	/// Get the scrolling region.
	pub fn scroll(&self) -> (u32, u32) {
		self.scroll
	}

	/// Update the current style if needed.
	pub fn update(&mut self, style: Style) {
		if &*self.style != &style {
			self.style = Rc::new(style);
		}
	}

	/// Move the cursor, returns the amount of cells it overflows if any.
	pub fn travel(&mut self, value: Travel) -> Option<i32> {
		if self.state.contains(WRAP) {
			self.state.remove(WRAP);
		}

		let mut overflow = None;

		match value {
			Position(x, y) => {
				if let Some(x) = x {
					if x >= self.width {
						self.x = self.width - 1;
					}
					else {
						self.x = x;
					}
				}

				if let Some(mut y) = y {
					if self.state.contains(ORIGIN) {
						y += self.scroll.0;
					}

					if y >= self.height {
						self.y = self.height - 1;
					}
					else {
						self.y = y;
					}
				}
			}

			Up(n) => {
				let new = (self.y as i32).saturating_sub(i32::try_from(n).unwrap_or(i32::MAX));

				if new < self.scroll.0 as i32 {
					self.y = self.scroll.0;
					overflow = Some(new - self.scroll.0 as i32);
				}
				else {
					self.y = new as u32;
				}
			}

			Down(n) => {
				let new = (self.y as i32).saturating_add(i32::try_from(n).unwrap_or(i32::MAX));

				if new > self.scroll.1 as i32 {
					self.y = self.scroll.1;
					overflow = Some(new - self.scroll.0 as i32);
				}
				else {
					self.y = new as u32;
				}
			}

			Left(n) => {
				let new = (self.x as i32).saturating_sub(i32::try_from(n).unwrap_or(i32::MAX));

				if new < 0 {
					self.x = 0;
					overflow = Some(new.abs());
				}
				else {
					self.x = new as u32;
				}
			}

			Right(n) => {
				let new = (self.x as i32).saturating_add(i32::try_from(n).unwrap_or(i32::MAX));

				if new >= self.width as i32 {
					self.x = self.width - 1;
					overflow = Some(new - (self.width as i32 - 1));
				}
				else {
					self.x = new as u32;
				}
			}
		}

		overflow
	}
}

/// A wrapper for a cursor and the cell it's on.
#[derive(Debug)]
pub struct Cell<'a> {
	cursor: &'a Cursor,
	cell:   cell::Position<'a>,
}

impl<'a> Cell<'a> {
	pub fn new(cursor: &'a Cursor, cell: cell::Position<'a>) -> Cell<'a> {
		Cell {
			cursor: cursor,
			cell:   cell,
		}
	}

	pub fn cell(&self) -> cell::Position {
		self.cell
	}
}

impl<'a> Deref for Cell<'a> {
	type Target = Cursor;

	fn deref(&self) -> &Self::Target {
		self.cursor
	}
}
