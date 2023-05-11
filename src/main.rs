use std::cmp::max;

fn main() {
	let w = 800;
	let h = 800;
	let mut image = image::ImageBuffer::new(w, h);

	for (px, py, pixel) in image.enumerate_pixels_mut() {
		let rx = px as f32 / w as f32;
		let ry = py as f32 / h as f32;
		let r = (rx * 255.0) as u8;
		let g = (ry * 255.0) as u8;
		let b = max(r, g);
		*pixel = image::Rgb([r, g, b]);
	}

	image.save("output.png").unwrap();
}
