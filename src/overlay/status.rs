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

use std::rc::Rc;
use std::ops::Deref;
use unicode_segmentation::UnicodeSegmentation;

use crate::style::Style;
use crate::terminal::Cell;

/// Status bar.
#[derive(Debug)]
pub struct Status {
	cols:  u32,
	style: Rc<Style>,

	inner:    Vec<Cell>,
	mode:     String,
	position: String,
}

impl Status {
	/// Create a new status bar with the given style and columns.
	pub fn new(style: Style, cols: u32) -> Self {
		let style = Rc::new(style);

		Status {
			cols:  cols,
			style: style.clone(),

			inner:    vec![Cell::empty(style.clone()); cols as usize],
			mode:     "".into(),
			position: "".into(),
		}
	}

	/// Change the mode shown in the status bar.
	pub fn mode<T: Into<String>>(&mut self, string: T) {
		let string = string.into();

		for (_, cell) in self.mode.graphemes(true).zip(self.inner.iter_mut()) {
			cell.make_empty(self.style.clone());
		}

		for (ch, cell) in string.graphemes(true).zip(self.inner.iter_mut()) {
			cell.make_occupied(ch, self.style.clone());
		}

		self.mode = string;
	}

	/// Change the cursor position shown in the status bar.
	pub fn position(&mut self, (x, y): (u32, u32)) {
		let format = format!("{}:{}", y, x);

		for (_, cell) in self.position.graphemes(true).rev().zip(self.inner.iter_mut().rev()) {
			cell.make_empty(self.style.clone());
		}

		for (ch, cell) in format.graphemes(true).rev().zip(self.inner.iter_mut().rev()) {
			cell.make_occupied(ch, self.style.clone());
		}

		self.position = format;
	}
}

impl Deref for Status {
	type Target = Vec<Cell>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
