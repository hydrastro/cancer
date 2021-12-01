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
use std::ptr;

use libc::c_int;
use crate::ffi::pango::*;
use crate::sys::glib;
use crate::sys::pango;
use crate::style;
use crate::error::{self, Error};

/// The font to use for rendering.
#[derive(Debug)]
pub struct Font {
	map:     pango::Map,
	context: pango::Context,
	set:     pango::Set,
	metrics: pango::Metrics,
}

unsafe impl Send for Font { }
unsafe impl Sync for Font { }

impl Font {
	/// Load the font from the given configuration.
	pub fn load<T: AsRef<str>>(name: T) -> error::Result<Self> {
		let map     = pango::Map::new();
		let context = pango::Context::new(&map);
		let set     = context.fonts(&pango::Description::from(name))
			.ok_or_else(|| Error::Message("missing font".into()))?;

		let metrics = set.metrics();
		if metrics.width() == 0 || metrics.height() == 0 {
			return Err(Error::Message("wrong font dimensions".into()));
		}

		Ok(Font {
			map:     map,
			context: context,
			set:     set,
			metrics: metrics,
		})
	}

	/// Shape the string.
	pub fn shape<T: AsRef<str>>(&self, text: T, style: style::Attributes) -> pango::GlyphItem {
		let text = text.as_ref();

		unsafe {
			let attrs = pango::Attributes::new();
			let attrs = if style.contains(style::BOLD) {
				attrs.weight(pango::Weight::Bold)
			}
			else if style.contains(style::FAINT) {
				attrs.weight(pango::Weight::Light)
			}
			else {
				attrs.weight(pango::Weight::Normal)
			};

			let attrs = if style.contains(style::ITALIC) {
				attrs.style(pango::Style::Italic)
			}
			else {
				attrs.style(pango::Style::Normal)
			};

			let     list   = glib::List(pango_itemize(self.context.0, text.as_ptr() as *const _, 0, text.len() as c_int, attrs.0, ptr::null()));
			let mut result = Vec::new();
			let mut list   = list.0;

			while !list.is_null() {
				let item   = (*list).data as *mut PangoItem;
				let glyphs = pango::GlyphString::new();
				pango_shape(text.as_ptr() as *const _, text.len() as c_int, &(*item).analysis, glyphs.0);

				result.push(pango::GlyphItem::new(pango::Item(item), glyphs));
				list = (*list).next;
			}

			result.into_iter().next().unwrap()
		}
	}
}

impl AsRef<pango::Context> for Font {
	fn as_ref(&self) -> &pango::Context {
		&self.context
	}
}

impl Deref for Font {
	type Target = pango::Metrics;

	fn deref(&self) -> &Self::Target {
		&self.metrics
	}
}
