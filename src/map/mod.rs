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

pub(crate) mod view;

use iced::{pure::widget::Canvas, Point};

use self::view::MapView;
use crate::{EditMode, Message};

pub(crate) type StationIndex = usize;
pub(crate) type LineIndex = usize;

#[derive(Debug, Clone)]
pub(crate) struct Map {
	stations: Vec<Station>,
	lines: Vec<Line>,
}

impl Default for Map {
	fn default() -> Self {
		Self {
			stations: vec![],
			lines: vec![Line { segments: vec![] }; 6],
		}
	}
}

impl Map {
	pub(crate) fn add_station(&mut self, station: Station) {
		self.stations.push(station);
	}

	pub(crate) fn add_segment(
		&mut self,
		line: LineIndex,
		start: StationIndex,
		end: StationIndex,
	) {
		self.lines[line].segments.push(Segment {
			start,
			interpolation: Interpolation::Auto(InterpolationDirection::Auto),
			end,
		})
	}

	pub(crate) fn remove_station(&mut self, index: StationIndex) {
		self.stations.remove(index);

		for line in &mut self.lines {
			let mut including_segments = line
				.segments
				.iter()
				.enumerate()
				.filter(|(_, seg)| seg.start == index || seg.end == index);

			let (start_seg, start_station, interpolation) =
				match including_segments.next() {
					Some((
						i,
						&Segment {
							start,
							end,
							interpolation,
						},
					)) => {
						if end == index {
							(i, start, interpolation)
						} else {
							(i, end, interpolation)
						}
					}
					None => continue,
				};

			let (end_seg, end_station) = match including_segments.next() {
				Some((i, &Segment { start, end, .. })) => {
					if end == index {
						(i, start)
					} else {
						(i, end)
					}
				}
				None => {
					line.segments.remove(start_seg);
					continue;
				}
			};

			if including_segments.next().is_some() {
				line.segments
					.retain(|seg| seg.start != index && seg.end != index);
			} else {
				line.segments.remove(end_seg);
				line.segments.remove(start_seg);
				line.segments.push(Segment {
					start: start_station,
					interpolation,
					end: end_station,
				})
			}
		}

		for line in &mut self.lines {
			for segment in &mut line.segments {
				if segment.start > index {
					segment.start -= 1;
				}
				if segment.end > index {
					segment.end -= 1;
				}
			}
		}
	}

	pub(crate) fn clear(&mut self) {
		*self = Self::default();
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Station {
	position: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct Line {
	segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
struct Segment {
	start: StationIndex,
	interpolation: Interpolation,
	end: StationIndex,
}

impl Segment {
	fn contains(&self, a: StationIndex, b: StationIndex) -> bool {
		self.start == a && self.end == b || self.start == b && self.end == a
	}
}

#[derive(Debug, Clone, Copy)]
enum Interpolation {
	Auto(InterpolationDirection),
}

#[derive(Debug, Clone, Copy)]
enum InterpolationDirection {
	Auto,
	Horizontal,
	Vertical,
	Diagonal,
}

impl Map {
	pub(crate) fn view(
		&self,
		edit_mode: EditMode,
		selected_line: LineIndex,
	) -> Canvas<Message, MapView<'_>> {
		Canvas::new(MapView::new(self, edit_mode, selected_line))
	}
}
