use std::sync::{Arc, Mutex};
use std::thread;

pub struct Tts {
    pub current: Arc<Mutex<String>>,
    pub amplitude: Arc<Mutex<f64>>,
    pub speaking: Arc<Mutex<bool>>,
}

impl Tts {
    pub fn new() -> Self {
        Tts {
            current: Arc::new(Mutex::new(String::new())),
            amplitude: Arc::new(Mutex::new(0.0)),
            speaking: Arc::new(Mutex::new(false)),
        }
    }

    pub fn speak(&self, text: &str) {
        *self.current.lock().unwrap() = text.to_string();
        *self.speaking.lock().unwrap() = true;

        println!("{text}");

        match tts::Tts::default() {
            Ok(mut engine) => {
                let _ = engine.set_rate(engine.normal_rate());
                let _ = engine.set_volume(engine.max_volume());

                if engine.speak(text, false).is_ok() {
                    *self.amplitude.lock().unwrap() = 0.5;

                    loop {
                        match engine.is_speaking() {
                            Ok(true) => std::thread::sleep(std::time::Duration::from_millis(50)),
                            _ => break,
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("tts unavailable: {e}");
            }
        }

        *self.amplitude.lock().unwrap() = 0.0;
        *self.speaking.lock().unwrap() = false;
    }

    pub fn speak_overlap(&self, text: &str) {
        *self.current.lock().unwrap() = text.to_string();

        println!("{text}");

        let text = text.to_string();
        let amplitude = Arc::clone(&self.amplitude);

        thread::spawn(move || {
            match tts::Tts::default() {
                Ok(mut engine) => {
                    let _ = engine.set_rate(engine.normal_rate());
                    let _ = engine.set_volume(engine.max_volume());

                    if engine.speak(&text, false).is_ok() {
                        *amplitude.lock().unwrap() = 0.5;

                        loop {
                            match engine.is_speaking() {
                                Ok(true) => {
                                    std::thread::sleep(std::time::Duration::from_millis(50))
                                }
                                _ => break,
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("tts unavailable: {e}");
                }
            }
            *amplitude.lock().unwrap() = 0.0;
        });
    }

    pub fn wait(&self) {
        loop {
            if !*self.speaking.lock().unwrap() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    #[allow(dead_code)]
    pub fn get_amplitude(&self) -> f64 {
        *self.amplitude.lock().unwrap()
    }
}
