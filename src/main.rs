use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use image::{GenericImageView, GenericImage, Rgba};
use crate::voronoi::Voronoi;

mod voronoi;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long)]
	preserve_above: Option<u8>,
	#[arg(long, value_enum, default_value_t = voronoi::EdgeMode::Zero)]
	edge_mode: voronoi::EdgeMode,
	#[arg(long, value_enum, default_value_t = OutputAs::Bleed)]
	output_as: OutputAs,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputAs {
	Bleed,
	Coverage,
	UV,
	Distance
}

fn main() {
    let args = Cli::parse();

	println!("Loading image from {}...", &args.input.display());
	let reader = image::io::Reader::open(&args.input).unwrap().with_guessed_format().unwrap();
	let format = reader.format().unwrap_or(image::ImageFormat::Png);
	let mut image = reader.decode().unwrap();
	let (width, height) = (image.width(), image.height());
	println!("Image size is {} by {}", width, height);

	let mut search_radius = (width.max(height) as f64).log2().ceil().exp2() as i64 / 2;
	println!("Will use a search radius of {}", search_radius);

	let preserve_above = args.preserve_above.unwrap_or(0);
	println!("Will preserve alpha above {}", preserve_above);

	println!("Will use {} edge behaviour", args.edge_mode);

	let mut voronoi1 = Voronoi::new(width, height);
	let mut voronoi2 = Voronoi::new(width, height);
	let mut voronoi_read = &mut voronoi1;
	let mut voronoi_write = &mut voronoi2;

	println!("Preparing Voronoi graph for filling...");
	for y in 0..height {
		for x in 0..width {
			let alpha = image.get_pixel(x, y)[3];
			if alpha > preserve_above {
				voronoi_read.set_closest((x, y), Some((x, y)));
			}
		}
	}

	println!("Filling Voronoi graph...");
	loop {
		for y in 0..height as i64 {
			for x in 0..width as i64 {
				const CARDINAL_MULT: f64 = 1.0;
				const DIAGONAL_MULT: f64 = 1.41421356;
				let maybe_closest = [
					(voronoi_read.get_closest((x, y), args.edge_mode), 1.0),
					(voronoi_read.get_closest((x, y - search_radius), args.edge_mode), CARDINAL_MULT),
					(voronoi_read.get_closest((x, y + search_radius), args.edge_mode), CARDINAL_MULT),
					(voronoi_read.get_closest((x - search_radius, y), args.edge_mode), CARDINAL_MULT),
					(voronoi_read.get_closest((x + search_radius, y), args.edge_mode), CARDINAL_MULT),
					(voronoi_read.get_closest((x - search_radius, y - search_radius), args.edge_mode), DIAGONAL_MULT),
					(voronoi_read.get_closest((x - search_radius, y + search_radius), args.edge_mode), DIAGONAL_MULT),
					(voronoi_read.get_closest((x + search_radius, y - search_radius), args.edge_mode), DIAGONAL_MULT),
					(voronoi_read.get_closest((x + search_radius, y + search_radius), args.edge_mode), DIAGONAL_MULT)
				]
				.into_iter()
				.filter_map(|(maybe_position, multiplier)| {
					if let Some(position) = maybe_position {
						let distance_x = position.0 as f64 - x as f64;
						let distance_y = position.1 as f64 - y as f64;
						let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
						Some((position, distance * multiplier))
					} else {
						None
					}
					
				}).
				reduce(|best, candidate| if candidate.1 < best.1 { candidate } else { best });
	
				if let Some(closest) = maybe_closest {
					voronoi_write.set_closest((x as u32, y as u32), Some(closest.0));
				}
			}
		}

		let old_voronoi_read = voronoi_read;
		voronoi_read = voronoi_write;
		voronoi_write = old_voronoi_read;

		if search_radius <= 1 {
			break
		} else {
			search_radius /= 2;
		}
	}

	match args.output_as {
		OutputAs::Bleed => {
			println!("Bleeding pixels...");
			for y in 0..height {
				for x in 0..width {
					if let Some(position) = voronoi_read.get_closest((x as i64, y as i64), args.edge_mode) {
						let colour = image.get_pixel(position.0, position.1);
						image.put_pixel(x, y, Rgba([colour[0], colour[1], colour[2], 255]));
					} else {
						image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
					}
				}
			}
		},
		OutputAs::UV => {
			println!("Plotting closest UVs...");
			for y in 0..height {
				for x in 0..width {
					if let Some(position) = voronoi_read.get_closest((x as i64, y as i64), args.edge_mode) {
						image.put_pixel(x, y, Rgba([
							(position.0 as f64 * 255.0 / width as f64).round() as u8, 
							(position.1 as f64 * 255.0 / height as f64).round() as u8,
							0, 
							255
						]));
					} else {
						image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
					}
				}
			}
		},
		OutputAs::Coverage => {
			println!("Plotting Voronoi coverage...");
			for y in 0..height {
				for x in 0..width {
					if let Some(_) = voronoi_read.get_closest((x as i64, y as i64), args.edge_mode) {
						image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
					} else {
						image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
					}
				}
			}
		},
		OutputAs::Distance => {
			println!("Plotting distance field...");
			for y in 0..height {
				for x in 0..width {
					if let Some(position) = voronoi_read.get_closest((x as i64, y as i64), args.edge_mode) {
						let distance_x = position.0 as f64 - x as f64;
						let distance_y = position.1 as f64 - y as f64;
						let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
						let value = 255 - distance.clamp(0.0, 255.0).round() as u8;
						image.put_pixel(x, y, Rgba([value, value, value, 255]));
					} else {
						image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
					}
				}
			}
		},
	}

	let output = args.output.unwrap_or(args.input);
	println!("Saving image to {}...", output.display());
	image.save_with_format(output, format).unwrap();
	println!("Completed!");
}
