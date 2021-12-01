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
use std::mem;
use std::rc::Rc;
use unicode_width::UnicodeWidthStr;
use tendril::StrTendril;

use crate::style::Style;
use crate::sys::cairo;

#[derive(PartialEq, Clone, Debug)]
pub enum Cell {
	Empty {
		style: Rc<Style>,
	},

	Image {
		style:  Rc<Style>,
		buffer: Box<cairo::Image>,
	},

	Occupied {
		style: Rc<Style>,
		value: StrTendril,
	},

	Reference(u8),
}

impl Default for Cell {
	fn default() -> Self {
		Cell::Empty {
			style: Rc::new(Style::default()),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Position<'a> {
	x: u32,
	y: u32,

	inner: &'a Cell,
}

impl Cell {
	/// Create an empty cell.
	pub fn empty(style: Rc<Style>) -> Self {
		Cell::Empty {
			style: style
		}
	}

	/// Create an occupied cell.
	pub fn occupied(value: String, style: Rc<Style>) -> Self {
		Cell::Occupied {
			value: value.into(),
			style: style,
		}
	}

	/// Create a referencing cell.
	pub fn reference(offset: u8) -> Self {
		Cell::Reference(offset)
	}

	/// Check if the cell is in its default state.
	pub fn is_default(&self) -> bool {
		if let Cell::Empty { ref style, .. } = *self {
			style.foreground.is_none() &&
			style.background.is_none() &&
			style.attributes.is_empty()
		}
		else {
			false
		}
	}

	/// Check if the cell is empty.
	pub fn is_empty(&self) -> bool {
		if let Cell::Empty { .. } = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is occupied.
	pub fn is_occupied(&self) -> bool {
		if let Cell::Occupied { .. } = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is a reference.
	pub fn is_reference(&self) -> bool {
		if let Cell::Reference(..) = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is an image.
	pub fn is_image(&self) -> bool {
		if let Cell::Image { .. } = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is wide.
	pub fn is_wide(&self) -> bool {
		match *self {
			Cell::Empty { .. } |
			Cell::Image { .. } =>
				false,

			Cell::Occupied { ref value, .. } =>
				value.as_ref().width() > 1,

			Cell::Reference(..) =>
				unreachable!()
		}
	}

	/// Make the cell empty.
	pub fn make_empty(&mut self, style: Rc<Style>) {
		mem::replace(self, Cell::Empty {
			style: style,
		});
	}

	/// Make the cell occupied.
	pub fn make_occupied<T: Into<String>>(&mut self, value: T, style: Rc<Style>) {
		mem::replace(self, Cell::Occupied {
			value: value.into().into(),
			style: style,
		});
	}

	/// Make the cell into a reference.
	pub fn make_reference(&mut self, offset: u8) {
		mem::replace(self, Cell::Reference(offset));
	}

	/// Make the cell into an image.
	pub fn make_image(&mut self, buffer: cairo::Image, style: Rc<Style>) {
		if let Cell::Image { ref buffer, .. } = *self {
			if buffer.as_ref() == buffer.as_ref() {
				return;
			}
		}

		mem::replace(self, Cell::Image {
			buffer: Box::new(buffer),
			style:  style,
		});
	}

	/// Change the style in place.
	pub fn set_style(&mut self, value: Rc<Style>) {
		match *self {
			Cell::Empty { ref mut style, .. } |
			Cell::Image { ref mut style, .. } |
			Cell::Occupied { ref mut style, .. } =>
				*style = value,

			Cell::Reference(..) =>
				()
		}
	}

	/// Get the cell style.
	pub fn style(&self) -> &Rc<Style> {
		match *self {
			Cell::Empty { ref style, .. } |
			Cell::Image { ref style, .. } |
			Cell::Occupied { ref style, .. } =>
				style,

			Cell::Reference(..) =>
				unreachable!(),
		}
	}

	/// Get the value if any.
	pub fn value(&self) -> &str {
		match *self {
			Cell::Empty { .. } =>
				" ",

			Cell::Occupied { ref value, .. } =>
				value.as_ref(),

			Cell::Reference(..) |
			Cell::Image { .. } =>
				"",
		}
	}

	/// Get the cell width.
	pub fn width(&self) -> u32 {
		match *self {
			Cell::Empty { .. } |
			Cell::Image { .. } =>
				1,

			Cell::Occupied { ref value, .. } =>
				value.as_ref().width() as u32,

			Cell::Reference(..) =>
				unreachable!(),
		}
	}

	/// Get the reference offset.
	pub fn offset(&self) -> u32 {
		match *self {
			Cell::Reference(offset) =>
				offset as u32,

			Cell::Empty { .. } |
			Cell::Occupied { .. } |
			Cell::Image { .. } =>
				unreachable!()
		}
	}

	/// Get the image buffer.
	pub fn image(&self) -> &cairo::Image {
		match *self {
			Cell::Image { ref buffer, .. } =>
				buffer,

			Cell::Empty { .. } |
			Cell::Occupied { .. } |
			Cell::Reference(..) =>
				unreachable!()
		}
	}
}

impl<'a> Position<'a> {
	pub fn new(x: u32, y: u32, inner: &Cell) -> Position {
		Position {
			x: x,
			y: y,

			inner: inner
		}
	}

	/// Get the X.
	pub fn x(&self) -> u32 {
		self.x
	}

	/// Get the Y.
	pub fn y(&self) -> u32 {
		self.y
	}
}

impl<'a> Deref for Position<'a> {
	type Target = Cell;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}
