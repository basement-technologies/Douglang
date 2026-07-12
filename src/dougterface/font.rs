use ab_glyph::{FontArc, Font as _, PxScale, ScaleFont, point};

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

fn load_font() -> FontArc {
    let candidates = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\Arial.ttf",
        r"C:\Windows\Fonts\consola.ttf",
        r"C:\Windows\Fonts\cour.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];
    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(font) = FontArc::try_from_vec(data) {
                return font;
            }
        }
    }
    panic!("no system font found D:");
}

fn blend_pixel(bg: u32, r: u32, g: u32, b: u32, a: u32) -> u32 {
    let inv = 255 - a;
    let ro = (r * a + ((bg >> 16) & 0xFF) * inv) / 255;
    let go = (g * a + ((bg >> 8) & 0xFF) * inv) / 255;
    let bo = (b * a + (bg & 0xFF) * inv) / 255;
    0xFF000000 | (ro << 16) | (go << 8) | bo
}

pub fn centered(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    y: usize,
) {
    let font = load_font();
    let px_size = 32.0;
    let scale = PxScale::from(px_size);
    let scaled = font.as_scaled(scale);

    let total_w: f32 = text
        .chars()
        .map(|c| scaled.h_advance(scaled.glyph_id(c)))
        .sum();
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
