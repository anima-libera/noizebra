use std::{
	f32::consts::TAU,
	hash::{Hash, Hasher},
};

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
	if x < 0.0 {
		0.0
	} else if 1.0 < x {
		1.0
	} else {
		(f32::cos((1.0 - x) * TAU / 2.0) + 1.0) / 2.0
	}
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
	let mut coef = 1.0;
	for _i in 0..octave_count {
		value_sum += coef * raw_noise_a(&xs, channels);
		coef_sum += coef;
		coef /= 2.0;
		xs.iter_mut().for_each(|x| *x *= 2.0);
	}
	value_sum / coef_sum
}

fn image_generator_test_00(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 400.0;
	let nosie_value =
		raw_noise_a_node(&[f32::floor(rx * scale) as i32, f32::floor(ry * scale) as i32]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_01(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 80.0;
	let nosie_value = octaves_noise_a(6, &[rx * scale, ry * scale], &[]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_02(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 80.0;
	let nosie_value = octaves_noise_a(6, &[rx * scale, ry * scale], &[]);
	let gray = if nosie_value < 0.5 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_03(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 20.0;
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
	let scale = 20.0;
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
	let scale_a = 10.0;
	let scale_b = 10.0;
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
	let scale_a = 10.0;
	let scale_b = 10.0;
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
	let scale_a = 10.0;
	let scale_p = 4.0;
	let scale_b = 10.0;
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
	let scale_a = 10.0;
	let scale_b = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let value = intersection * intersection * intersection * intersection * intersection;
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_09(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let value = intersection * intersection * intersection * intersection * intersection;
	let gray = if value < 0.9 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_10(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_p = 4.0;
	let scale_b = 10.0;
	let nosie_value_x = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_y = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[2]);
	let power = octaves_noise_a(4, &[rx * scale_p, ry * scale_p], &[4]);
	let power = power * power * power * power;
	let x = rx + (nosie_value_x * 2.0 - 1.0) * power;
	let y = ry + (nosie_value_y * 2.0 - 1.0) * power;
	let nosie_value = octaves_noise_a(6, &[x * scale_b, y * scale_b], &[3]);
	let gray = (nosie_value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_11(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_p = 4.0;
	let scale_b = 10.0;
	let nosie_value_x = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_y = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[2]);
	let power_noise = octaves_noise_a(4, &[rx * scale_p, ry * scale_p], &[4]);
	let power = power_noise * power_noise * power_noise * power_noise;
	let x = rx + (nosie_value_x * 2.0 - 1.0) * power;
	let y = ry + (nosie_value_y * 2.0 - 1.0) * power;
	let nosie_value = octaves_noise_a(6, &[x * scale_b, y * scale_b], &[3]);
	let red_value = f32::cos(power_noise * 2.0);
	let red_value = if red_value < 0.0 {
		0.0
	} else if 1.0 < red_value {
		1.0
	} else {
		red_value
	};
	image::Rgb([
		(red_value * 255.0) as u8,
		(nosie_value * 255.0) as u8,
		((1.0 - nosie_value_x * nosie_value_y) * 255.0) as u8,
	])
}

fn image_generator_test_12(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection_u = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let intersection_v = 1.0 - f32::abs(nosie_value_a - (1.0 - nosie_value_b)) / 2.0;
	let u = intersection_u * intersection_u * intersection_u * intersection_u * intersection_u;
	let v = intersection_v * intersection_v * intersection_v * intersection_v * intersection_v;
	image::Rgb([
		if u < 0.9 { 0u8 } else { 255u8 },
		if v < 0.9 { 0u8 } else { 255u8 },
		0,
	])
}

fn image_generator_test_13(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection_u = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let intersection_v = 1.0 - f32::abs(nosie_value_a - (1.0 - nosie_value_b)) / 2.0;
	let u = intersection_u.powi(20);
	let v = intersection_v.powi(20);
	image::Rgb([(u * 255.0) as u8, (v * 255.0) as u8, 0])
}

fn image_generator_test_14(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_s = 4.0;
	let scale_a = 10.0;
	let scale_b = 6.0 * octaves_noise_a(4, &[rx * scale_s, ry * scale_s], &[3]);
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let intersection_u = 1.0 - f32::abs(nosie_value_a - nosie_value_b) / 2.0;
	let intersection_v = 1.0 - f32::abs(nosie_value_a - (1.0 - nosie_value_b)) / 2.0;
	let u = intersection_u.powi(20);
	let v = intersection_v.powi(20);
	image::Rgb([(u * 255.0) as u8, (v * 255.0) as u8, 0])
}

fn image_generator_test_15(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 20.0;
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let distance = 0.5;
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let scale_b = 3.0;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale_b, ry * scale_b], &[2]);
	let gray = (nosie_value_b * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_16(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let distance = 0.03;
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let scale_b = 10.0;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale_b, ry * scale_b], &[2]);
	let gray = (nosie_value_b * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_17(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 40.0;
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let distance = 0.005;
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let scale_b = 10.0;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale_b, ry * scale_b], &[2]);
	let gray = (nosie_value_b * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_18(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_m = 10.0;
	let scale_a = 40.0 * octaves_noise_a(4, &[rx * scale_m, ry * scale_m], &[4]);
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let scale_d = 10.0;
	let distance = 0.04 * octaves_noise_a(4, &[rx * scale_d, ry * scale_d], &[3]);
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let scale_b = 10.0;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale_b, ry * scale_b], &[2]);
	let gray = (nosie_value_b * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_19(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_m = 10.0;
	let scale_a = 40.0 * octaves_noise_a(4, &[rx * scale_m, ry * scale_m], &[4]);
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let scale_d = 10.0;
	let distance = 0.04 * octaves_noise_a(4, &[rx * scale_d, ry * scale_d], &[3]);
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let scale_b = 10.0;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale_b, ry * scale_b], &[2]);
	image::Rgb([
		(nosie_value_a * nosie_value_b * 255.0) as u8,
		(nosie_value_b * 255.0) as u8,
		(nosie_value_b * 255.0) as u8,
	])
}

fn image_generator_test_20(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_m = 4.0;
	let scale_a = 10.0 * octaves_noise_a(4, &[rx * scale_m, ry * scale_m], &[4]);
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let scale_d = 10.0;
	let distance = 0.2 * octaves_noise_a(4, &[rx * scale_d, ry * scale_d], &[3]);
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	image::Rgb([
		interpolate(&smoothcos, rx, 0.0, 1.0, 0.0, 255.0) as u8,
		interpolate(&smoothcos, ry, 0.0, 1.0, 0.0, 255.0) as u8,
		interpolate(&smoothcos, 1.0 - rx * ry, 0.0, 1.0, 0.0, 255.0) as u8,
	])
}

fn image_generator_test_21(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_m = 4.0;
	let scale_a = 10.0 * octaves_noise_a(4, &[rx * scale_m, ry * scale_m], &[4]);
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let scale_d = 10.0;
	let distance = 0.3 * octaves_noise_a(4, &[rx * scale_d, ry * scale_d], &[3]);
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let value = f32::hypot(rx - 0.5, ry - 0.5);
	let value = interpolate(&smoothcos, value, 0.0, 0.3, 1.0, 0.0);
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_22(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_m = 4.0;
	let scale_a = 5.0 * octaves_noise_a(4, &[rx * scale_m, ry * scale_m], &[4]);
	let nosie_value_a = octaves_noise_a(15, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let scale_d = 3.0;
	let distance = 0.4 * octaves_noise_a(4, &[rx * scale_d, ry * scale_d], &[3]);
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let gray = if rx < 0.5 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_23(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let gray = ((f32::cos(angle) * 0.5 + 0.5) * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_24(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 8.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value_a = f32::cos(angle * 3.5) * 0.5 + 0.5;
	let value_b = f32::sin(angle) * 0.5 + 0.5;
	let value_c = f32::cos(angle * 5.0) * 0.5 + 0.5;
	image::Rgb([
		(value_a * 255.0) as u8,
		0, //(value_b * 255.0) as u8,
		(value_c * 255.0) as u8,
	])
}

fn image_generator_test_25(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 8.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle * 20.0) * 0.5 + 0.5;
	image::Rgb([
		(value * 255.0) as u8,
		(value * 255.0) as u8,
		(value * 255.0) as u8,
	])
}

fn image_generator_test_26(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 8.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale_b, ry * scale_b], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let distance = f32::hypot(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = (f32::cos(angle * 3.0) * 0.5 + 0.5) * distance.powi(4);
	let gray = if value < 0.001 { 0u8 } else { 255u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_27(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 8.0;
	let nosie_value_a = octaves_noise_a(10, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(10, &[rx * scale_b, ry * scale_b], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let distance = f32::hypot(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	let value = interpolate(&smoothcos, distance, 0.0, 0.4, 0.5, value);
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_28(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let scale_b = 8.0;
	let nosie_value_a = octaves_noise_a(10, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_b = octaves_noise_a(10, &[rx * scale_b, ry * scale_b], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let distance = f32::hypot(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	let value = interpolate(&smoothcos, distance, 0.0, 0.4, 0.5, value);
	image::Rgb([
		(distance * 4.0 * 255.0) as u8,
		(nosie_value_a * 255.0) as u8,
		(value * 255.0) as u8,
	])
}

fn image_generator_test_29(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(6, &[rx * scale, ry * scale], &[1]);
	let angle = nosie_value_a * TAU;
	let distance = interpolate(
		&smoothcos,
		f32::hypot(rx - 0.5, ry - 0.5),
		0.0,
		0.5,
		1.0,
		0.0,
	);
	let rx = rx + f32::cos(angle) * distance;
	let ry = ry + f32::sin(angle) * distance;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale, ry * scale], &[1]);
	image::Rgb([
		(nosie_value_b * 255.0) as u8,
		(distance * nosie_value_b * 255.0) as u8,
		(distance * nosie_value_a * nosie_value_b * 255.0) as u8,
	])
}

fn image_generator_test_30(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let nosie_value_a = octaves_noise_a(6, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let n = 4;
	let distance = 0.02;
	let mut rx_m = rx;
	let mut ry_m = ry;
	let scale_b = 10.0;
	let nosie_value_b = octaves_noise_a(6, &[rx * scale_b, ry * scale_b], &[2]);
	for i in 0..n {
		let angle_i = angle + TAU * (i as f32) / (n as f32);
		let rx_i = rx_m + f32::cos(angle_i) * distance;
		let ry_i = ry_m + f32::sin(angle_i) * distance;
		let nosie_value_b_i = octaves_noise_a(6, &[rx_i * scale_b, ry_i * scale_b], &[2]);
		if nosie_value_b < nosie_value_b_i {
			rx_m = rx_i;
			ry_m = ry_i;
			break;
		}
	}
	let scale_c = 10.0;
	let nosie_value_c = octaves_noise_a(6, &[rx_m * scale_c, ry_m * scale_c], &[3]);
	let gray = (nosie_value_c * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_31(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let nosie_value_a = octaves_noise_a(4, &[rx * scale_a, ry * scale_a], &[1]);
	let angle = nosie_value_a * TAU;
	let n = 5;
	let distance = 0.002;
	let mut rx_m = rx;
	let mut ry_m = ry;
	for _j in 0..30 {
		let scale_b = 10.0;
		let nosie_value_b = octaves_noise_a(4, &[rx * scale_b, ry * scale_b], &[2]);
		for i in 0..n {
			let angle_i = angle + TAU * (i as f32) / (n as f32);
			let rx_i = rx_m + f32::cos(angle_i) * distance;
			let ry_i = ry_m + f32::sin(angle_i) * distance;
			let nosie_value_b_i = octaves_noise_a(4, &[rx_i * scale_b, ry_i * scale_b], &[2]);
			if nosie_value_b < nosie_value_b_i {
				rx_m = rx_i;
				ry_m = ry_i;
				break;
			}
		}
	}
	let scale_c = 10.0;
	image::Rgb([
		(rx_m * 255.0) as u8,
		(ry_m * 255.0) as u8,
		(octaves_noise_a(5, &[rx_m * scale_c, ry_m * scale_c], &[3]) * 255.0) as u8,
	])
}

fn image_generator_test_32(rx: f32, ry: f32) -> image::Rgb<u8> {
	let angle = 0.0 * TAU;
	let n = 3;
	let distance = 0.005;
	let mut rx_m = rx;
	let mut ry_m = ry;
	for _j in 0..30 {
		let scale_b = 10.0;
		let nosie_value_b = octaves_noise_a(4, &[rx * scale_b, ry * scale_b], &[2]);
		for i in 0..n {
			let angle_i = angle + TAU * (i as f32) / (n as f32);
			let rx_i = rx_m + f32::cos(angle_i) * distance;
			let ry_i = ry_m + f32::sin(angle_i) * distance;
			let nosie_value_b_i = octaves_noise_a(4, &[rx_i * scale_b, ry_i * scale_b], &[2]);
			if nosie_value_b < nosie_value_b_i {
				rx_m = rx_i;
				ry_m = ry_i;
				break;
			}
		}
	}
	let scale_c = 10.0;
	image::Rgb([
		(rx_m * 255.0) as u8,
		(ry_m * 255.0) as u8,
		(octaves_noise_a(5, &[rx_m * scale_c, ry_m * scale_c], &[3]) * 255.0) as u8,
	])
}

fn image_generator_test_33(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	let gray = if 0.8 < value { 255u8 } else { 0u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_34(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	if 0.8 < value {
		image::Rgb([255u8, 200u8, 0u8])
	} else if value < 0.2 {
		image::Rgb([255u8, 80u8, 255u8])
	} else {
		image::Rgb([0u8, 0u8, 0u8])
	}
}

fn image_generator_test_35(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	if value < 0.2 || 0.8 < value {
		image::Rgb([0u8, 0u8, 0u8])
	} else {
		image::Rgb([255u8, 255u8, 255u8])
	}
}

fn image_generator_test_36(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let length = f32::hypot(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	if 0.8 < value {
		if length < 0.1 {
			image::Rgb([255u8, 80u8, 255u8])
		} else {
			image::Rgb([255u8, 255u8, 255u8])
		}
	} else {
		image::Rgb([0u8, 0u8, 0u8])
	}
}

fn image_generator_test_37(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value_a = f32::cos(angle) * 0.5 + 0.5;
	let value_b = f32::sin(angle) * 0.5 + 0.5;
	image::Rgb([(value_a * 255.0) as u8, (value_b * 255.0) as u8, 0u8])
}

fn image_generator_test_38(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::tan(angle);
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_39(rx: f32, ry: f32) -> image::Rgb<u8> {
	let offset_scale = 10.0;
	let offset_max = 3.0;
	let offset = octaves_noise_a(5, &[rx * offset_scale, ry * offset_scale], &[3]);
	let offset = offset * offset_max;
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale, offset], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale, offset], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_40(rx: f32, ry: f32) -> image::Rgb<u8> {
	let offset_scale = 10.0;
	let offset_max = 3.0;
	let offset = octaves_noise_a(5, &[rx * offset_scale, ry * offset_scale], &[3]);
	let offset = offset * offset_max;
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale, offset], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale, offset], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	let gray = if 0.8 < value { 255u8 } else { 0u8 };
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_41(rx: f32, ry: f32) -> image::Rgb<u8> {
	let offset_scale = 10.0;
	let offset_max = 3.0;
	let offset = octaves_noise_a(5, &[rx * offset_scale, ry * offset_scale], &[3]);
	let offset = offset * offset_max;
	let scale = 10.0;
	let nosie_value_a = octaves_noise_a(5, &[rx * scale, ry * scale, offset], &[1]);
	let nosie_value_b = octaves_noise_a(5, &[rx * scale, ry * scale, offset], &[2]);
	let angle = f32::atan2(nosie_value_a - 0.5, nosie_value_b - 0.5);
	let value = f32::cos(angle) * 0.5 + 0.5;
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_42(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale_a = 10.0;
	let nosie_value_x = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[1]);
	let nosie_value_y = octaves_noise_a(5, &[rx * scale_a, ry * scale_a], &[2]);
	let scale_b = 1.0;
	let value = octaves_noise_a(5, &[nosie_value_x * scale_b, nosie_value_y * scale_b], &[3]);
	let gray = (value * 255.0) as u8;
	image::Rgb([gray, gray, gray])
}

fn image_generator_test_43(rx: f32, ry: f32) -> image::Rgb<u8> {
	let scale = 10.0;
	let n = 10;
	let (i, _value) = (0..n)
		.map(|i| octaves_noise_a(5, &[rx * scale, ry * scale], &[i]))
		.enumerate()
		.max_by_key(|(_i, value)| (value * 100.0) as u32)
		.unwrap();
	image::Rgb([
		((i * 1827 + 237) % 256) as u8,
		((i * 1911 + 141) % 256) as u8,
		((i * 1137 + 883) % 256) as u8,
	])
}

fn render_to_file(
	generator: &dyn Fn(f32, f32) -> image::Rgb<u8>,
	side: u32,
	path: impl AsRef<std::path::Path>,
) {
	let mut image = image::ImageBuffer::new(side, side);
	for (px, py, pixel) in image.enumerate_pixels_mut() {
		let rx = px as f32 / side as f32;
		let ry = py as f32 / side as f32;
		*pixel = generator(rx, ry);
	}
	image.save(path).unwrap();
}

fn main() {
	if false {
		std::fs::create_dir_all("output").ok();
		render_to_file(&image_generator_test_43, 800, "output/output.png");
	} else {
		let generators = [
			image_generator_test_00,
			image_generator_test_01,
			image_generator_test_02,
			image_generator_test_03,
			image_generator_test_04,
			image_generator_test_05,
			image_generator_test_06,
			image_generator_test_07,
			image_generator_test_08,
			image_generator_test_09,
			image_generator_test_10,
			image_generator_test_11,
			image_generator_test_12,
			image_generator_test_13,
			image_generator_test_14,
			image_generator_test_15,
			image_generator_test_16,
			image_generator_test_17,
			image_generator_test_18,
			image_generator_test_19,
			image_generator_test_20,
			image_generator_test_21,
			image_generator_test_22,
			image_generator_test_23,
			image_generator_test_24,
			image_generator_test_25,
			image_generator_test_26,
			image_generator_test_27,
			image_generator_test_28,
			image_generator_test_29,
			image_generator_test_30,
			image_generator_test_31,
			image_generator_test_32,
			image_generator_test_33,
			image_generator_test_34,
			image_generator_test_35,
			image_generator_test_36,
			image_generator_test_37,
			image_generator_test_38,
			image_generator_test_39,
			image_generator_test_40,
			image_generator_test_41,
			image_generator_test_42,
			image_generator_test_43,
		];
		std::fs::create_dir_all("output").ok();
		for (i, generator) in generators.iter().enumerate() {
			let i_max = generators.len() - 1;
			println!("{i} / {i_max}");
			render_to_file(generator, 1000, format!("output/output_{i}.png"));
		}
	}
}
