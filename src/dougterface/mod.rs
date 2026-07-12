mod font;

use std::sync::{Arc, Mutex};
use std::thread;

use crate::tts::Tts;
use font::centered;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BG: u32 = 0xFF646464;

pub struct Dougterface {
    pub running: Arc<Mutex<bool>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Dougterface {
    pub fn new(_tts: &Tts) -> Self {
        let running = Arc::new(Mutex::new(false));
        Dougterface {
            running,
            thread: None,
        }
    }

    pub fn start(&mut self, tts: &Tts) {
        if *self.running.lock().unwrap() {
            return;
        }
        *self.running.lock().unwrap() = true;

        let running = Arc::clone(&self.running);
        let current = Arc::clone(&tts.current);
        let amplitude = Arc::clone(&tts.amplitude);

        let handle = thread::spawn(move || {
            use minifb::{Key, Window, WindowOptions};

            let sprite_bytes = include_bytes!("../assets/wario_pepper.png");
            let sprite_img = match image::load_from_memory(sprite_bytes) {
                Ok(img) => img.to_rgba8(),
                Err(e) => {
                    eprintln!("failed loading sprite: {e}");
                    return;
                }
            };

            let target_wid = 174u32;
            let target_ht = 193u32;
            let scaled = image::imageops::resize(
                &sprite_img,
                target_wid,
                target_ht,
                image::imageops::FilterType::Triangle,
            );

            let sprite_wid = target_wid as usize;
            let sprite_ht = target_ht as usize;
            let sprite_buf: Vec<u32> = scaled
                .pixels()
                .map(|p| {
                    let [r, g, b, a] = p.0;
                    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                })
                .collect();

            let mut window = match Window::new(
                "The Douglang Dougterface",
                WIDTH,
                HEIGHT,
                WindowOptions::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("failed creating window: {e}");
                    return;
                }
            };

            window.set_target_fps(60);

            let mut buffer: Vec<u32> = vec![BG; WIDTH * HEIGHT];

            while window.is_open() && !window.is_key_down(Key::Escape) {
                if !*running.lock().unwrap() {
                    break;
                }

                let text = current.lock().unwrap().clone();
                let amp = *amplitude.lock().unwrap();

                buffer.fill(BG);

                let txt_ht = 30usize;
                let total_ht = sprite_ht + txt_ht;
                let block_top = (HEIGHT.saturating_sub(total_ht)) / 2;

                let sprite_x = (WIDTH.saturating_sub(sprite_wid)) / 2;
                let amp_off = (amp * 20.0) as isize;
                let sprite_y = (block_top as isize - amp_off).max(0) as usize;

                for sy in 0..sprite_ht {
                    for sx in 0..sprite_wid {
                        let px = sprite_x + sx;
                        let py = sprite_y + sy;
                        if px < WIDTH && py < HEIGHT {
                            let src = sprite_buf[sy * sprite_wid + sx];
                            let alpha = (src >> 24) & 0xFF;
                            if alpha > 128 {
                                buffer[py * WIDTH + px] = src | 0xFF000000;
                            }
                        }
                    }
                }

                let txt_y = block_top + sprite_ht + 10;
                centered(&mut buffer, WIDTH, HEIGHT, &text, txt_y);

                let _ = window.update_with_buffer(&buffer, WIDTH, HEIGHT);
            }

            *running.lock().unwrap() = false;
        });

        self.thread = Some(handle);
    }

    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Dougterface {
    fn drop(&mut self) {
        self.stop();
    }
}
