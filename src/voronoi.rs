use std::fmt::Display;
use clap::ValueEnum;
use enum_display_derive::Display;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Display)]
pub enum EdgeMode {
	Clamp,
	Repeat,
	Zero
}

#[derive(Clone)]
pub struct Voronoi {
	width: u32,
	height: u32,
	pub closest: Vec<Option<(u32, u32)>>,
}

impl Voronoi {
	pub fn new(
		width: u32,
		height: u32
	) -> Self {
		let mut closest = Vec::with_capacity((width * height) as usize);
		for _ in 0..(width * height) {
			closest.push(None)
		}
		Self {
			width,
			height,
			closest
		}
	}

	pub fn get_closest(
		&self,
		position: (i64, i64),
		edge_mode: EdgeMode
	) -> Option<(u32, u32)> {
		let Some(position) = (
			match edge_mode {
				EdgeMode::Clamp => Some((
					position.0.clamp(0, self.width as i64 - 1) as u32,
					position.1.clamp(0, self.height as i64 - 1) as u32
				)),
				EdgeMode::Repeat => Some((
					position.0.rem_euclid(self.width as i64) as u32,
					position.1.rem_euclid(self.height as i64) as u32
				)),
				EdgeMode::Zero => {
					let in_bounds = 
						position.0 >= 0 && position.0 < self.width as i64 && 
						position.1 >= 0 && position.1 < self.height as i64;
					if in_bounds {
						Some((position.0 as u32, position.1 as u32))
					} else {
						None
					}
				}
			}
		) else {
			return None;
		};

		self.closest[(position.1 * self.width + position.0) as usize]
	}

	pub fn set_closest(
		&mut self,
		position: (u32, u32),
		new_closest: Option<(u32, u32)>
	) {
		self.closest[(position.1 * self.width + position.0) as usize] = new_closest;
	}
}