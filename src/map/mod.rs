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

mod view;

use std::collections::HashMap;

use iced::{
	pure::widget::{canvas::Program, Canvas},
	Color, Point,
};
use petgraph::{graph::NodeIndex, visit::EdgeRef, Undirected};

use self::view::MapView;
use crate::{color::ColorExt, EditMode, Message};

type Index = u16;

pub(crate) type StationIndex = NodeIndex<Index>;
pub(crate) type LineIndex = Index;

type Graph = petgraph::Graph<Station, Segment, Undirected, Index>;

#[derive(Debug, Clone)]
pub(crate) struct Map {
	graph: Graph,
	lines: Vec<Line>,
}

impl Default for Map {
	fn default() -> Self {
		Self {
			graph: Graph::with_capacity(0, 0),
			lines: [0x33bbff, 0x3cbe3c, 0xff714d, 0xbf60bf, 0xff9600, 0xffd700]
				.into_iter()
				.map(|c| Line {
					color: Color::from_rgb32(c),
				})
				.collect(),
		}
	}
}

impl Map {
	pub(crate) fn add_station(&mut self, station: Station) {
		self.graph.add_node(station);
	}

	pub(crate) fn add_segment(
		&mut self,
		line: LineIndex,
		start: StationIndex,
		end: StationIndex,
	) {
		self.graph.add_edge(
			start,
			end,
			Segment {
				line,
				interpolation: Interpolation::Auto(
					InterpolationDirection::Auto,
				),
			},
		);
	}

	pub(crate) fn remove_station(&mut self, index: StationIndex) {
		let mut to_rejoin = HashMap::new();

		for edge in self.graph.edges(index) {
			let line = edge.weight().line;
			let interpolation = edge.weight().interpolation;
			let endpoint = if edge.source() == index {
				edge.target()
			} else {
				edge.source()
			};

			match to_rejoin.get(&line).copied() {
				None => {
					to_rejoin.insert(line, Ok((endpoint, None, interpolation)));
				}
				Some(Ok((a, None, i))) => {
					to_rejoin.insert(line, Ok((a, Some(endpoint), i)));
				}
				Some(Ok((_, Some(_), _))) => {
					to_rejoin.insert(line, Err(()));
				}
				Some(Err(())) => (),
			}
		}

		self.graph.remove_node(index);

		for (line, rejoin) in to_rejoin {
			let (a, b, interpolation) = match rejoin {
				Ok((a, Some(b), i)) => (a, b, i),
				_ => continue,
			};

			self.graph.add_edge(
				a,
				b,
				Segment {
					line,
					interpolation,
				},
			);
		}
	}

	pub(crate) fn clear(&mut self) {
		self.graph.clear();
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Station {
	position: Point,
}

#[derive(Debug, Clone)]
struct Line {
	color: Color,
}

#[derive(Debug, Clone)]
struct Segment {
	line: LineIndex,
	interpolation: Interpolation,
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
	) -> Canvas<Message, impl Program<Message> + '_> {
		Canvas::new(MapView::new(self, edit_mode, selected_line))
	}
}
