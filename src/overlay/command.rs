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

use crate::platform::Clipboard;

pub enum Command {
	None,
	Exit,
	Move(Move),
	Scroll(Scroll),
	Select(Select),
	Copy(Clipboard),
	Paste(Clipboard),
	Hint(Hint),
}

pub enum Scroll {
	Up(u32),
	Down(u32),
	PageUp(u32),
	PageDown(u32),
	Begin,
	End,
	To(u32),
}

pub enum Move {
	Left(u32),
	Right(u32),
	Up(u32),
	Down(u32),
	Start,
	End,
	To(u32, u32),
	Next(u32, Next),
	Previous(u32, Previous),
}

pub enum Next {
	Word(Word),
	Match(Match),
}

pub enum Previous {
	Word(Word),
	Match(Match),
}

pub type Boundary = Box<dyn Fn(&str) -> bool>;

pub enum Word {
	Start(Boundary),
	End(Boundary),
}

pub enum Match {
	After(String),
	Before(String),
}

pub enum Select {
	Normal,
	Block,
	Line,
}

pub enum Hint {
	Start(u32),
	Pick(char),
	Open,
	Copy(Clipboard),
}
