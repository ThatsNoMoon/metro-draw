// Copyright (C) 2022  ThatsNoMoon
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of version 3 of the GNU Affero General Public License as published
// by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use iced::Color;

pub(crate) trait ColorExt {
	fn from_rgb32(rgb: u32) -> Self;
}

impl ColorExt for Color {
	fn from_rgb32(rgb: u32) -> Self {
		let (r, g, b) = (
			((rgb >> 16) & 0xFF) as u8,
			((rgb >> 8) & 0xFF) as u8,
			(rgb & 0xFF) as u8,
		);

		Self::from_rgb8(r, g, b)
	}
}
