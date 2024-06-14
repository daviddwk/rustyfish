use std::path::PathBuf;
use std::convert::TryFrom;

use rand::Rng;

use animation::{Animation, load_animation, glyph_from_animation, Size, Position};
use color_glyph::ColorGlyph;
use error::error;
use open_json::open_json;

pub struct Layer {
    frame: usize,
    anim: Animation
}
impl Layer {
    pub fn get_glyph(&self, row_idx: usize, glyph_idx: usize) -> Option<&ColorGlyph> {
        let glyph = glyph_from_animation(&self.anim, self.frame, row_idx, glyph_idx, Position{x: 0, y: 0});
        if glyph.is_some() && glyph.as_ref().unwrap().glyph == ' ' {
            return None;
        }
        return glyph;
    }
}

pub struct Tank {
    pub size: Size,
    pub depth: usize,
    pub fg: Layer,
    pub bg: Layer
}
impl Tank {
    pub fn new(path: &PathBuf, name: &str) -> Self {
        let tank_json = open_json(path, name, "tank");
        let mut depth: usize = 0; 
        let fg_anim = load_animation(&tank_json, &format!("tank {}", name), "/foreground_animation");
        let bg_anim = load_animation(&tank_json, &format!("tank {}", name), "/background_animation");

        if fg_anim[0].len() != bg_anim[0].len() || fg_anim[0][0].len() != bg_anim[0][0].len() {
            error(&format!("tank {} has a mismatich in foreground and background size", name), 1);
        } 
        if tank_json["depth"].is_u64() {
            depth = usize::try_from(tank_json["depth"].as_u64().unwrap()).unwrap();
        }
    
        let mut rng = rand::thread_rng();
        return Self {
            size: Size {height: fg_anim[0].len(), width: fg_anim[0][0].len()},
            depth,
            fg: Layer{
                frame: rng.gen_range(0..fg_anim.len()),
                anim: fg_anim
            },
            bg: Layer{
                frame: rng.gen_range(0..bg_anim.len()),
                anim: bg_anim
            }
        }
    }
    pub fn update(&mut self) {
        self.fg.frame += 1;
        self.bg.frame += 1;
        if self.fg.frame >= self.fg.anim.len() { self.fg.frame = 0; }
        if self.bg.frame >= self.bg.anim.len() { self.bg.frame = 0; }
    }
}
