use std::io::Read;

use ab_glyph::{Font as _, FontArc, PxScale, ScaleFont, point};
use flate2::read::ZlibDecoder;

const FONT_BYTES: &[u8] = include_bytes!("../assets/ChaparralPro-Bold.woff");
const OUTLINE: [(i32, i32); 8] = [
	(-1, -1),
	(-1, 0),
	(-1, 1),
	(0, -1),
	(0, 1),
	(1, -1),
	(1, 0),
	(1, 1),
];
const PX_SIZE: f32 = 32.0;
const LINE_HEIGHT: usize = 38;

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
	Some(u16::from_be_bytes(
		data.get(offset..offset + 2)?.try_into().ok()?,
	))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
	Some(u32::from_be_bytes(
		data.get(offset..offset + 4)?.try_into().ok()?,
	))
}

fn write_u16(out: &mut [u8], offset: usize, value: u16) {
	out[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn write_u32(out: &mut [u8], offset: usize, value: u32) {
	out[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn align4(value: usize) -> usize {
	(value + 3) & !3
}

fn decode_woff(data: &[u8]) -> Option<Vec<u8>> {
	if data.get(0..4)? != b"wOFF" {
		return None;
	}

	let flavor = read_u32(data, 4)?;
	let num_tables = read_u16(data, 12)?;
	let table_count = num_tables as usize;
	let total_size = read_u32(data, 16)? as usize;
	let directory_size = 12 + table_count * 16;
	let mut tables = Vec::with_capacity(table_count);
	let mut data_offset = align4(directory_size);

	for i in 0..table_count {
		let entry = 44 + i * 20;
		let tag = read_u32(data, entry)?;
		let offset = read_u32(data, entry + 4)? as usize;
		let comp_len = read_u32(data, entry + 8)? as usize;
		let orig_len = read_u32(data, entry + 12)? as usize;
		let checksum = read_u32(data, entry + 16)?;
		let bytes = data.get(offset..offset + comp_len)?;
		let table_data = if comp_len < orig_len {
			let mut decoder = ZlibDecoder::new(bytes);
			let mut decoded = Vec::with_capacity(orig_len);
			decoder.read_to_end(&mut decoded).ok()?;
			if decoded.len() != orig_len {
				return None;
			}
			decoded
		} else {
			bytes.to_vec()
		};
		tables.push((tag, checksum, data_offset, table_data));
		data_offset = align4(data_offset + orig_len);
	}

	let mut out = vec![0; total_size.max(data_offset)];
	write_u32(&mut out, 0, flavor);
	write_u16(&mut out, 4, num_tables);

	let max_pow = 1usize << (usize::BITS - table_count.leading_zeros() - 1);
	let search_range = (max_pow * 16) as u16;
	let entry_selector = max_pow.trailing_zeros() as u16;
	let range_shift = (table_count * 16 - search_range as usize) as u16;
	write_u16(&mut out, 6, search_range);
	write_u16(&mut out, 8, entry_selector);
	write_u16(&mut out, 10, range_shift);

	for (i, (tag, checksum, offset, table_data)) in tables.iter().enumerate() {
		let entry = 12 + i * 16;
		write_u32(&mut out, entry, *tag);
		write_u32(&mut out, entry + 4, *checksum);
		write_u32(&mut out, entry + 8, *offset as u32);
		write_u32(&mut out, entry + 12, table_data.len() as u32);
		out[*offset..*offset + table_data.len()].copy_from_slice(table_data);
	}

	Some(out)
}

fn load_font() -> FontArc {
	if let Ok(font) = FontArc::try_from_slice(FONT_BYTES) {
		return font;
	}

	let decoded = decode_woff(FONT_BYTES).expect("failed decoding ChaparralPro-Bold.woff");
	FontArc::try_from_vec(decoded).expect("failed loading ChaparralPro-Bold.woff")
}

fn blend_pixel(bg: u32, r: u32, g: u32, b: u32, a: u32) -> u32 {
	let inv = 255 - a;
	let ro = (r * a + ((bg >> 16) & 0xFF) * inv) / 255;
	let go = (g * a + ((bg >> 8) & 0xFF) * inv) / 255;
	let bo = (b * a + (bg & 0xFF) * inv) / 255;
	0xFF000000 | (ro << 16) | (go << 8) | bo
}

fn text_width(font: &FontArc, text: &str) -> f32 {
	let scale = PxScale::from(PX_SIZE);
	let scaled = font.as_scaled(scale);
	text.chars()
		.map(|c| scaled.h_advance(scaled.glyph_id(c)))
		.sum()
}

fn wrapped_lines(font: &FontArc, text: &str, max_w: usize) -> Vec<String> {
	let max_w = max_w as f32;
	let mut lines = Vec::new();

	for raw_line in text.lines() {
		let mut line = String::new();
		for word in raw_line.split_whitespace() {
			let candidate = if line.is_empty() {
				word.to_string()
			} else {
				format!("{line} {word}")
			};

			if !line.is_empty() && text_width(font, &candidate) > max_w {
				lines.push(line);
				line = word.to_string();
			} else {
				line = candidate;
			}
		}

		if line.is_empty() {
			lines.push(String::new());
		} else {
			lines.push(line);
		}
	}

	if lines.is_empty() {
		lines.push(String::new());
	}

	lines
}

pub fn wrapped_height(text: &str, max_w: usize) -> usize {
	let font = load_font();
	wrapped_lines(&font, text, max_w).len() * LINE_HEIGHT
}

fn centered_line(
	buffer: &mut [u32],
	buf_w: usize,
	buf_h: usize,
	font: &FontArc,
	text: &str,
	y: usize,
) {
	let scale = PxScale::from(PX_SIZE);
	let scaled = font.as_scaled(scale);
	let total_w = text_width(font, text);
	let start_x = (buf_w as f32 - total_w) / 2.0;

	struct Cpx {
		sx: i32,
		sy: i32,
		cov: f32,
	}

	let mut pixels = Vec::new();
	let mut cx = start_x;
	for ch in text.chars() {
		let gid = scaled.glyph_id(ch);
		let glyph = gid.with_scale_and_position(scale, point(cx, y as f32));
		if let Some(outline) = font.outline_glyph(glyph) {
			let bounds = outline.px_bounds();
			outline.draw(|lx, ly, cov| {
				if cov <= 0.0 {
					return;
				}
				pixels.push(Cpx {
					sx: bounds.min.x as i32 + lx as i32,
					sy: bounds.min.y as i32 + ly as i32,
					cov,
				});
			});
		}
		cx += scaled.h_advance(gid);
	}

	for p in &pixels {
		let a = (p.cov * 255.0) as u32;
		for &(dx, dy) in &OUTLINE {
			let px = p.sx + dx;
			let py = p.sy + dy;
			if px >= 0 && (px as usize) < buf_w && py >= 0 && (py as usize) < buf_h {
				let idx = py as usize * buf_w + px as usize;
				buffer[idx] = blend_pixel(buffer[idx], 0, 0, 0, a);
			}
		}
	}

	for p in &pixels {
		let a = (p.cov * 255.0) as u32;
		if p.sx >= 0 && (p.sx as usize) < buf_w && p.sy >= 0 && (p.sy as usize) < buf_h {
			let idx = p.sy as usize * buf_w + p.sx as usize;
			buffer[idx] = blend_pixel(buffer[idx], 255, 255, 255, a);
		}
	}
}

pub fn wrapped_centered(
	buffer: &mut [u32],
	buf_w: usize,
	buf_h: usize,
	text: &str,
	y: usize,
	max_w: usize,
) {
	let font = load_font();
	for (i, line) in wrapped_lines(&font, text, max_w).iter().enumerate() {
		centered_line(buffer, buf_w, buf_h, &font, line, y + i * LINE_HEIGHT);
	}
}
