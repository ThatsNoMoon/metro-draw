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

use iced::{
	canvas::{
		event::Status, Cache, Cursor, Event, Frame, Geometry, LineCap,
		LineDash, LineJoin, Path, Stroke,
	},
	keyboard::{self, KeyCode, Modifiers},
	mouse,
	pure::widget::canvas::Program,
	Color, Point, Rectangle, Vector,
};
use ordered_float::NotNan;

use super::{
	Interpolation, InterpolationDirection, LineIndex, Map, Station,
	StationIndex,
};
use crate::{color::ColorExt, EditMode, Message};

pub(crate) struct MapView<'m> {
	map: &'m Map,
	edit_mode: EditMode,
	selected_line: LineIndex,
}

impl<'m> MapView<'m> {
	pub(super) fn new(
		map: &'m Map,
		edit_mode: EditMode,
		selected_line: LineIndex,
	) -> Self {
		Self {
			map,
			edit_mode,
			selected_line,
		}
	}
}

#[derive(Debug, Default)]
pub(crate) struct ViewState {
	cache: Cache,
	dragging: DragState,
	pan_offset: Vector,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DragState {
	None,
	Clicked(ClickStart),
	Dragging(ClickStart, Option<StationIndex>),
	Panning(Point, Vector),
}

impl Default for DragState {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ClickStart {
	Station(StationIndex),
	Empty(Point),
}

const STATION_INNER_SIZE: f32 = 15.0;
const STATION_OUTER_SIZE: f32 = 20.0;

const GRID_SIZE: f32 = 50.0;

const LINE_COLORS: &[u32] =
	&[0x33bbff, 0x3cbe3c, 0xff714d, 0xbf60bf, 0xff9600, 0xffd700];

fn line_color(line: LineIndex) -> Color {
	Color::from_rgb32(LINE_COLORS[line % LINE_COLORS.len()])
}

const DRAG_RANGE: f32 = 5.0;

impl Program<Message> for MapView<'_> {
	type State = ViewState;
	fn draw(
		&self,
		state: &ViewState,
		bounds: Rectangle,
		cursor: Cursor,
	) -> Vec<Geometry> {
		let geometry = state.cache.draw(bounds.size(), |frame| {
			let background = Path::rectangle(Point::ORIGIN, frame.size());
			frame.fill(&background, Color::from_rgb32(0x19191D));

			frame.translate(state.pan_offset);

			{
				let v_grid_lines =
					(bounds.width / GRID_SIZE + 2.0).ceil() as u32;
				let h_grid_lines =
					(bounds.height / GRID_SIZE + 2.0).ceil() as u32;

				let stroke = Stroke {
					color: Color::from_rgb32(0x3d3d4a),
					width: 2.0,
					line_cap: LineCap::Butt,
					line_join: LineJoin::Miter,
					line_dash: LineDash {
						segments: &[],
						offset: 0,
					},
				};

				let start_x =
					(-state.pan_offset.x / GRID_SIZE - 1.0).round() * GRID_SIZE;

				let top_y = -state.pan_offset.y - GRID_SIZE;
				let bottom_y = -state.pan_offset.y + bounds.height + GRID_SIZE;

				for i in 0..v_grid_lines {
					let x = start_x + GRID_SIZE * i as f32;

					let top = Point::new(x, top_y);
					let bottom = Point::new(x, bottom_y);

					let line = Path::line(top, bottom);
					frame.stroke(&line, stroke)
				}

				let start_y =
					(-state.pan_offset.y / GRID_SIZE - 1.0).round() * GRID_SIZE;

				let left_x = -state.pan_offset.x - GRID_SIZE;
				let right_x = -state.pan_offset.x + bounds.width + GRID_SIZE;

				for i in 0..h_grid_lines {
					let y = start_y + GRID_SIZE * i as f32;

					let left = Point::new(left_x, y);
					let right = Point::new(right_x, y);

					let line = Path::line(left, right);
					frame.stroke(&line, stroke)
				}
			}

			for (i, line) in self.map.lines.iter().enumerate() {
				for segment in &line.segments {
					let start = self.map.stations[segment.start].position;
					let end = self.map.stations[segment.end].position;
					self.draw_segment(
						start,
						end,
						segment.interpolation,
						line_color(i),
						frame,
					);
				}
			}

			if let (
				DragState::Dragging(ClickStart::Station(s), _),
				EditMode::Line,
				Some(p),
			) = (state.dragging, self.edit_mode, cursor.position())
			{
				self.draw_segment(
					self.map.stations[s].position,
					p - (bounds.position() - Point::ORIGIN) - state.pan_offset,
					Interpolation::Auto(InterpolationDirection::Auto),
					line_color(self.selected_line),
					frame,
				)
			}

			for station in &self.map.stations {
				frame.fill(
					&Path::circle(station.position, STATION_OUTER_SIZE),
					Color::from_rgb32(0xd8e0ef),
				);
				frame.fill(
					&Path::circle(station.position, STATION_INNER_SIZE),
					Color::from_rgb32(0x030405),
				);
			}
		});

		vec![geometry]
	}

	fn update(
		&self,
		state: &mut ViewState,
		event: Event,
		bounds: Rectangle,
		cursor: Cursor,
	) -> (Status, Option<Message>) {
		match event {
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
				match cursor.position() {
					Some(p) if bounds.contains(p) => {
						let panned = (p - (bounds.position() - Point::ORIGIN))
							- state.pan_offset;
						if self.edit_mode != EditMode::None {
							state.dragging = if let Some(station) =
								self.find_station_at(panned)
							{
								DragState::Clicked(ClickStart::Station(station))
							} else {
								DragState::Clicked(ClickStart::Empty(panned))
							};
						}
						return (Status::Captured, None);
					}
					_ => (),
				}
			}
			Event::Mouse(mouse::Event::ButtonPressed(
				mouse::Button::Middle,
			)) => {
				if let Some(p) = cursor.position() {
					let p = p - (bounds.position() - Point::ORIGIN);
					state.dragging = DragState::Panning(p, state.pan_offset);
					return (Status::Captured, None);
				}
			}
			Event::Mouse(mouse::Event::CursorMoved { position }) => {
				let p = position - (bounds.position() - Point::ORIGIN);
				let panned = p - state.pan_offset;
				match state.dragging {
					DragState::Clicked(start_pos) => {
						state.cache.clear();
						match start_pos {
							ClickStart::Station(s) => {
								let d = magnitude(
									self.map.stations[s].position - panned,
								);
								if d > DRAG_RANGE {
									let inside =
										(d < STATION_OUTER_SIZE).then(|| s);

									state.dragging =
										DragState::Dragging(start_pos, inside);
								}
							}
							ClickStart::Empty(start) => {
								let d = magnitude(start - panned);
								if d > DRAG_RANGE {
									state.dragging =
										DragState::Panning(p, state.pan_offset);
								}
							}
						}
					}
					DragState::Dragging(ClickStart::Station(start), inside)
						if self.edit_mode == EditMode::Line =>
					{
						state.cache.clear();

						match inside {
							None => {
								if let Some(now_inside) =
									self.find_station_at(panned)
								{
									if now_inside != start
										&& !self.map.lines[self.selected_line]
											.segments
											.iter()
											.any(|s| {
												s.contains(start, now_inside)
											}) {
										state.dragging = DragState::Dragging(
											ClickStart::Station(now_inside),
											Some(now_inside),
										);
										return (
											Status::Captured,
											Some(Message::AddSegment(
												start, now_inside,
											)),
										);
									}
								}
							}
							Some(s) => {
								if magnitude(
									self.map.stations[s].position - panned,
								) > STATION_OUTER_SIZE
								{
									state.dragging = DragState::Dragging(
										ClickStart::Station(start),
										None,
									);
								}
							}
						}
					}
					DragState::Panning(start, initial_offset) => {
						state.cache.clear();
						state.pan_offset = initial_offset + (p - start);

						let (min_x, max_x) = min_max(
							self.map.stations.iter().map(|s| s.position.x),
						);
						let (min_y, max_y) = min_max(
							self.map.stations.iter().map(|s| s.position.y),
						);

						state.pan_offset.x = state
							.pan_offset
							.x
							.min(max_x)
							.max(min_x - bounds.width);
						state.pan_offset.y = state
							.pan_offset
							.y
							.min(max_y)
							.max(min_y - bounds.height);
					}
					_ => (),
				}
			}
			Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
				let dragging = state.dragging;

				state.cache.clear();
				state.dragging = DragState::None;

				match (self.edit_mode, dragging) {
					(
						EditMode::Station,
						DragState::Clicked(ClickStart::Empty(p)),
					) => {
						let x = (p.x / GRID_SIZE).round() * GRID_SIZE;
						let y = (p.y / GRID_SIZE).round() * GRID_SIZE;
						let position = Point::new(x, y);
						return (
							Status::Captured,
							Some(Message::AddStation(Station { position })),
						);
					}
					(
						EditMode::Remove,
						DragState::Clicked(ClickStart::Station(s)),
					) => {
						return (
							Status::Captured,
							Some(Message::RemoveStation(s)),
						)
					}
					_ => (),
				}
			}
			Event::Mouse(mouse::Event::ButtonReleased(
				mouse::Button::Middle,
			)) => {
				state.cache.clear();
				state.dragging = DragState::None;
			}
			Event::Keyboard(keyboard::Event::KeyPressed {
				key_code,
				modifiers,
			}) => {
				if modifiers == Modifiers::CTRL {
					if key_code == KeyCode::Delete {
						state.cache.clear();
						return (Status::Captured, Some(Message::ClearMap));
					}
				} else if modifiers.is_empty()
					&& state.dragging == DragState::None
				{
					match key_code {
						KeyCode::R => {
							return (
								Status::Captured,
								Some(Message::SwitchEditMode(EditMode::Remove)),
							)
						}
						KeyCode::A => {
							return (
								Status::Captured,
								Some(Message::SwitchEditMode(
									EditMode::Station,
								)),
							)
						}
						KeyCode::D => {
							return (
								Status::Captured,
								Some(Message::SwitchEditMode(EditMode::Line)),
							)
						}
						KeyCode::S => {
							return (
								Status::Captured,
								Some(Message::SwitchEditMode(EditMode::None)),
							)
						}
						KeyCode::Key1 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(0)),
							)
						}
						KeyCode::Key2 if self.map.lines.len() >= 2 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(1)),
							)
						}
						KeyCode::Key3 if self.map.lines.len() >= 3 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(2)),
							)
						}
						KeyCode::Key4 if self.map.lines.len() >= 4 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(3)),
							)
						}
						KeyCode::Key5 if self.map.lines.len() >= 5 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(4)),
							)
						}
						KeyCode::Key6 if self.map.lines.len() >= 6 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(5)),
							)
						}
						KeyCode::Key7 if self.map.lines.len() >= 7 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(6)),
							)
						}
						KeyCode::Key8 if self.map.lines.len() >= 8 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(7)),
							)
						}
						KeyCode::Key9 if self.map.lines.len() >= 9 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(8)),
							)
						}
						KeyCode::Key0 if self.map.lines.len() >= 10 => {
							return (
								Status::Captured,
								Some(Message::SwitchLine(9)),
							)
						}
						_ => (),
					}
				}
			}
			_ => (),
		}

		(Status::Ignored, None)
	}
}

impl MapView<'_> {
	fn find_station_at(&self, p: Point) -> Option<StationIndex> {
		self.map
			.stations
			.iter()
			.enumerate()
			.rev()
			.map(|(i, s)| {
				let d = NotNan::new(magnitude(s.position - p)).unwrap();
				(i, d)
			})
			.min_by_key(|&(_, d)| d)
			.filter(|(_, d)| d.into_inner() < STATION_OUTER_SIZE)
			.map(|(i, _)| i)
	}

	fn draw_segment(
		&self,
		start: Point,
		end: Point,
		interpolation: Interpolation,
		color: Color,
		frame: &mut Frame,
	) {
		let line = Path::new(|b| match interpolation {
			Interpolation::Auto(d) => {
				b.move_to(start);

				let mid = interpolate_auto(start, end, d);

				b.line_to(mid);

				b.line_to(end);
			}
		});

		frame.stroke(
			&line,
			Stroke {
				color,
				width: 10.0,
				line_cap: LineCap::Round,
				line_join: LineJoin::Round,
				line_dash: LineDash {
					segments: &[],
					offset: 0,
				},
			},
		);
	}
}

fn magnitude(v: Vector) -> f32 {
	(v.x.powi(2) + v.y.powi(2)).sqrt()
}

fn min_max(values: impl IntoIterator<Item = f32>) -> (f32, f32) {
	let (min, max) = values.into_iter().fold(
		(f32::INFINITY, f32::NEG_INFINITY),
		|(mut min, mut max), x| {
			if x > max {
				max = x
			}
			if x < min {
				min = x
			}
			(min, max)
		},
	);

	let min = if min.is_finite() { min } else { 0.0 };

	let max = if max.is_finite() { max } else { 0.0 };

	(min, max)
}

fn interpolate_auto(
	start: Point,
	end: Point,
	direction: InterpolationDirection,
) -> Point {
	use InterpolationDirection::*;

	let dx = end.x - start.x;
	let dy = end.y - start.y;

	let horizontal = || Point::new(end.x - dy.abs() * dx.signum(), start.y);
	let vertical = || Point::new(start.x, end.y - dx.abs() * dy.signum());
	let diagonal = || {
		if dx.abs() > dy.abs() {
			Point::new(start.x + dy.abs() * dx.signum(), end.y)
		} else {
			Point::new(end.x, start.y + dx.abs() * dy.signum())
		}
	};

	match direction {
		Horizontal => horizontal(),
		Vertical => vertical(),
		Diagonal => diagonal(),
		Auto => {
			if dx.abs() > dy.abs() {
				horizontal()
			} else {
				vertical()
			}
		}
	}
}
