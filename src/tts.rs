use std::sync::{Arc, Mutex};
use std::thread;

use crate::runtime::{TTS_IDLE_AMP, TTS_SPEAKING_AMP, format_tts_state};

pub struct Tts {
    pub current: Arc<Mutex<String>>,
    pub amplitude: Arc<Mutex<f64>>,
    pub speaking: Arc<Mutex<bool>>,
    state_file: Arc<Mutex<Option<String>>>,
}

impl Tts {
    pub fn new() -> Self {
        Tts {
            current: Arc::new(Mutex::new(String::new())),
            amplitude: Arc::new(Mutex::new(0.0)),
            speaking: Arc::new(Mutex::new(false)),
            state_file: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_state_file(&self, path: String) {
        *self.state_file.lock().unwrap() = Some(path);
    }

    fn write_state_file(&self) {
        if let Some(path) = self.state_file.lock().unwrap().clone() {
            let text = self.current.lock().unwrap().clone();
            let amp = *self.amplitude.lock().unwrap();
            let speaking = *self.speaking.lock().unwrap();
            let _ = std::fs::write(path, format_tts_state(&text, amp, speaking));
        }
    }

    pub fn speak(&self, text: &str) {
        *self.current.lock().unwrap() = text.to_string();
        *self.speaking.lock().unwrap() = true;
        self.write_state_file();

        println!("{text}");

        match tts::Tts::default() {
            Ok(mut engine) => {
                let _ = engine.set_rate(engine.normal_rate());
                let _ = engine.set_volume(engine.max_volume());

                if engine.speak(text, false).is_ok() {
                    *self.amplitude.lock().unwrap() = TTS_SPEAKING_AMP;
                    self.write_state_file();

                    while let Ok(true) = engine.is_speaking() {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
            Err(e) => {
                eprintln!("tts unavailable: {e}");
            }
        }

        *self.amplitude.lock().unwrap() = TTS_IDLE_AMP;
        *self.speaking.lock().unwrap() = false;
        self.write_state_file();
    }

    pub fn speak_overlap(&self, text: &str) {
        *self.current.lock().unwrap() = text.to_string();
        self.write_state_file();

        println!("{text}");

        let text = text.to_string();
        let amplitude = Arc::clone(&self.amplitude);
        let state_file = Arc::clone(&self.state_file);

        thread::spawn(move || {
            match tts::Tts::default() {
                Ok(mut engine) => {
                    let _ = engine.set_rate(engine.normal_rate());
                    let _ = engine.set_volume(engine.max_volume());

                    if engine.speak(&text, false).is_ok() {
                        *amplitude.lock().unwrap() = TTS_SPEAKING_AMP;
                        if let Some(path) = state_file.lock().unwrap().clone() {
                            let _ = std::fs::write(
                                path,
                                format_tts_state(&text, TTS_SPEAKING_AMP, true),
                            );
                        }

                        while let Ok(true) = engine.is_speaking() {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("tts unavailable: {e}");
                }
            }
            *amplitude.lock().unwrap() = TTS_IDLE_AMP;
            if let Some(path) = state_file.lock().unwrap().clone() {
                let _ = std::fs::write(path, format_tts_state(&text, TTS_IDLE_AMP, false));
            }
        });
    }

    pub fn speak_audio_only(&self, text: &str, _overlap: bool) {
        *self.current.lock().unwrap() = text.to_string();
        *self.speaking.lock().unwrap() = true;
        self.write_state_file();

        let text = text.to_string();
        let amplitude = Arc::clone(&self.amplitude);
        let speaking = Arc::clone(&self.speaking);
        let state_file = Arc::clone(&self.state_file);

        match tts::Tts::default() {
            Ok(mut engine) => {
                let _ = engine.set_rate(engine.normal_rate());
                let _ = engine.set_volume(engine.max_volume());

                if engine.speak(&text, false).is_ok() {
                    *amplitude.lock().unwrap() = TTS_SPEAKING_AMP;
                    if let Some(path) = state_file.lock().unwrap().clone() {
                        let _ =
                            std::fs::write(path, format_tts_state(&text, TTS_SPEAKING_AMP, true));
                    }
                    while let Ok(true) = engine.is_speaking() {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
            Err(e) => {
                eprintln!("tts unavailable: {e}");
            }
        }
        *amplitude.lock().unwrap() = TTS_IDLE_AMP;
        *speaking.lock().unwrap() = false;
        if let Some(path) = state_file.lock().unwrap().clone() {
            let _ = std::fs::write(path, format_tts_state(&text, TTS_IDLE_AMP, false));
        }
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
