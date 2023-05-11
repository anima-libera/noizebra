use std::f32::consts::TAU;

fn positive_fract(x: f32) -> f32 {
	x - f32::floor(x)
}

fn smoothstep(x: f32) -> f32 {
	if x < 0.0 {
		0.0
	} else if 1.0 < x {
		1.0
	} else {
		x * x * (3.0 - 2.0 * x)
	}
}

fn smoothcos(x: f32) -> f32 {
	(f32::cos((1.0 - x) * TAU / 2.0) + 1.0) / 2.0
}

fn interpolate(
	smoothing: &dyn Fn(f32) -> f32,
	x: f32,
	x_inf: f32,
	x_sup: f32,
	dst_inf: f32,
	dst_sup: f32,
) -> f32 {
	let ratio = (x - x_inf) / (x_sup - x_inf);
	let smooth_ratio = smoothing(ratio);
	dst_inf + smooth_ratio * (dst_sup - dst_inf)
}

fn raw_noise_a_node(xs: &[i32]) -> f32 {
	let mut a = 0;
	let mut b = 0;
	for (i, x) in xs.iter().copied().enumerate() {
		a ^= x;
		b ^= 17 * (i as i32 + 11) + x;
		std::mem::swap(&mut a, &mut b);
		a ^= a << ((i + 7) % (((b % 11) as usize).saturating_add(5)));
	}
	positive_fract(f32::cos(a as f32 + b as f32))
}

fn raw_noise_a(xs: &[f32], channels: &[i32]) -> f32 {
	if xs.is_empty() {
		raw_noise_a_node(channels)
	} else {
		// For every continuous coordinate, we interpolate between
		// the two closest discreet node values on that axis.
		// In one dimension (with N <= x < N+1), it looks like this:
		// ... --|------#----|--> ...
		//       N      x   N+1
		//      inf         sup
		// And we can do that by calling this recursively
		// with N and N+1 as additional channel parameters.
		let mut channels_inf = Vec::from(channels);
		let mut channels_sup = Vec::from(channels);
		channels_inf.push(f32::floor(xs[0]) as i32);
		channels_sup.push(f32::floor(xs[0]) as i32 + 1);
		let sub_noise_inf = raw_noise_a(&xs[1..], &channels_inf);
		let sub_noise_sup = raw_noise_a(&xs[1..], &channels_sup);
		let x_fract = positive_fract(xs[0]);
		interpolate(&smoothcos, x_fract, 0.0, 1.0, sub_noise_inf, sub_noise_sup)
	}
}

fn octaves_noise_a(octave_count: i32, xs: &[f32], channels: &[i32]) -> f32 {
	let mut xs = Vec::from(xs);
	let mut value_sum = 0.0;
	let mut coef_sum = 0.0;
	let mut scale = 1.0;
	for _i in 0..octave_count {
		xs.iter_mut().for_each(|x| *x *= 2.0);
		let coef = 1.0 / (scale as f32);
		value_sum += coef * raw_noise_a(&xs, channels);
		coef_sum += coef;
		scale *= 2.0;
	}
	value_sum / coef_sum
}

fn image_generator_test_00(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 200.0;
	let nosie_value =
		raw_noise_a_node(&[f32::floor(rx * scale) as i32, f32::floor(ry * scale) as i32]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_01(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 40.0;
	let nosie_value = octaves_noise_a(6, &[rx * scale, ry * scale], &[]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_02(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 40.0;
	let nosie_value = octaves_noise_a(6, &[rx * scale, ry * scale], &[]);
	let gray = if nosie_value < 0.5 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_03(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_r = octaves_noise_a(6, &[rx * scale, ry * scale], &[]);
	let nosie_value_g = octaves_noise_a(3, &[rx * scale, ry * scale], &[]);
	let nosie_value_b = octaves_noise_a(1, &[rx * scale, ry * scale], &[]);
	image::Rgb([
		if nosie_value_r < 0.5 { 0u8 } else { 255u8 },
		if nosie_value_g < 0.5 { 0u8 } else { 255u8 },
		if nosie_value_b < 0.5 { 0u8 } else { 255u8 },
	])
}

fn image_generator_test_04(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_r = octaves_noise_a(1, &[rx * scale, ry * scale], &[1]);
	let nosie_value_g = octaves_noise_a(1, &[rx * scale, ry * scale], &[2]);
	let nosie_value_b = octaves_noise_a(1, &[rx * scale, ry * scale], &[3]);
	image::Rgb([
		if nosie_value_r < 0.5 { 0u8 } else { 255u8 },
		if nosie_value_g < 0.5 { 0u8 } else { 255u8 },
		if nosie_value_b < 0.5 { 0u8 } else { 255u8 },
	])
}

fn image_generator_test_05(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 5.0;
	let scale_b = 5.0;
	let nosie_value_x = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_y = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[2]);
	let power = 1.0;
	let x = rx + (nosie_value_x * 2.0 - 1.0) * power;
	let y = ry + (nosie_value_y * 2.0 - 1.0) * power;
	let nosie_value = octaves_noise_a(6, &[x * scale_b, y * scale_b], &[3]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_06(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 5.0;
	let scale_b = 5.0;
	let nosie_value_x = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_y = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[2]);
	let power = 1.0;
	let x = rx + (nosie_value_x * 2.0 - 1.0) * power;
	let y = ry + (nosie_value_y * 2.0 - 1.0) * power;
	let nosie_value = octaves_noise_a(6, &[x * scale_b, y * scale_b], &[3]);
	let gray = if nosie_value < 0.5 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_07(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 5.0;
	let scale_p = 2.0;
	let scale_b = 5.0;
	let nosie_value_x = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_y = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[2]);
	let power = octaves_noise_a(4, &[rx * scale_p, ry * scale_p], &[4]);
	let x = rx + (nosie_value_x * 2.0 - 1.0) * power;
	let y = ry + (nosie_value_y * 2.0 - 1.0) * power;
	let nosie_value = octaves_noise_a(6, &[x * scale_b, y * scale_b], &[3]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_08(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 5.0;
	let scale_b = 5.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let value = intersection * intersection * intersection * intersection * intersection;
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_09(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 5.0;
	let scale_b = 5.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let value = intersection * intersection * intersection * intersection * intersection;
	let gray = if value < 0.9 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn main() {
	let w = 800;
	let h = 800;
	let mut image = image::ImageBuffer::new(w, h);

	for (px, py, pixel) in image.enumerate_pixels_mut() {
		let rx = px as f32 / w as f32;
		let ry = py as f32 / h as f32;
		*pixel = image_generator_test_09(rx, ry);
	}

	image.save("output.png").unwrap();
}
