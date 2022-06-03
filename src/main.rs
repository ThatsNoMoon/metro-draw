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

#![allow(dead_code)]

mod color;
mod map;

use iced::{
	executor,
	pure::{Application, Element},
	Command, Length, Settings,
};

use crate::map::{LineIndex, Map, Station, StationIndex};

struct State {
	map: Map,
	edit_mode: EditMode,
	selected_line: LineIndex,
}

#[derive(Debug, Clone)]
enum Message {
	AddStation(Station),
	AddSegment(StationIndex, StationIndex),
	RemoveStation(StationIndex),
	SwitchLine(LineIndex),
	ClearMap,
	SwitchEditMode(EditMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditMode {
	None,
	Station,
	Line,
	Remove,
}

impl Application for State {
	type Executor = executor::Default;
	type Message = Message;
	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(
			State {
				map: Map::default(),
				edit_mode: EditMode::Station,
				selected_line: 0,
			},
			Command::none(),
		)
	}

	fn title(&self) -> String {
		"MetroDraw".to_owned()
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		match message {
			Message::AddStation(station) => {
				self.map.add_station(station);
			}
			Message::AddSegment(start, end) => {
				self.map.add_segment(self.selected_line, start, end);
			}
			Message::RemoveStation(station) => {
				self.map.remove_station(station);
			}
			Message::SwitchLine(line) => {
				self.selected_line = line;
			}
			Message::ClearMap => {
				self.map.clear();
			}
			Message::SwitchEditMode(mode) => {
				self.edit_mode = mode;
			}
		}

		Command::none()
	}

	fn view(&self) -> Element<'_, Message> {
		self.map
			.view(self.edit_mode, self.selected_line)
			.width(Length::Fill)
			.height(Length::Fill)
			.into()
	}
}

fn main() -> iced::Result {
	State::run(Settings {
		antialiasing: true,
		..Settings::default()
	})
}
