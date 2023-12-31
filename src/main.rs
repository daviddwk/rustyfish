use std::io::{self};
use std::fs::{create_dir, create_dir_all};
use std::path::{PathBuf, Path};
extern crate crossterm;
extern crate rand;
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, MoveTo},
    terminal::Clear,
};
extern crate home;
use home::home_dir;

use std::env;
use std::thread;
use std::time;use::color_glyph::*;

mod fish;
mod tank;
mod load_file;
mod color_glyph;
mod duck;

use fish::*;
use duck::*;
use tank::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut help_arg: bool = false;
    let mut list_arg: bool = false;
    let mut init_arg: bool = false;
    let mut speed_arg = String::new();
    let mut tank_arg = String::new();
    let mut fish_args: Vec<String> = Vec::new();
    let mut duck_args: Vec<String> = Vec::new();
    
    let freefish_dir = home::home_dir().unwrap().join(".config").join("freefish");
    let fish_dir = freefish_dir.join("fish");
    let tanks_dir = freefish_dir.join("tanks");
    let ducks_dir = freefish_dir.join("ducks");
    
    let mut args_iter = args.iter().skip(1);

    while let Some(arg) = args_iter.next() {
        if arg.eq("-h") {
            help_arg = true;
        } else if arg.eq("-i") {
            init_arg = true;
        } else if arg.eq("-l") {
            list_arg = true;
        } else if arg.eq("-s") {
            let mut tmp_args_iter = args_iter.clone();
            if let Some(arg) = tmp_args_iter.next() {
                if arg.chars().next() != Some('-') {
                    args_iter.next();
                    speed_arg = arg.clone();
                }
            }
        } else if arg.eq("-t") {
            let mut tmp_args_iter = args_iter.clone();
            if let Some(arg) = tmp_args_iter.next() {
                if arg.chars().next() != Some('-') {
                    args_iter.next();
                    tank_arg = arg.clone();
                }
            }
        } else if arg.eq("-f") {
            let mut tmp_args_iter = args_iter.clone();
            while let Some(arg) = tmp_args_iter.next() {
                if arg.chars().next() == Some('-') { break; }
                args_iter.next();
                fish_args.push(arg.clone()); 
            }
        } else if arg.eq("-d") {
            let mut tmp_args_iter = args_iter.clone();
            while let Some(arg) = tmp_args_iter.next() {
                if arg.chars().next() == Some('-') { break; }
                args_iter.next();
                duck_args.push(arg.clone()); 
            }
        } else {
            println!("invalid argument {}", arg);
            std::process::exit(1);
        }
    }
    
    if help_arg {
        println!("Help text");
        std::process::exit(0);
    }

    if init_arg {
        if let Err(e) = std::fs::create_dir_all(freefish_dir.clone()) { panic!("{}", e)};
        if let Err(e) = std::fs::create_dir(fish_dir) { panic!("{}", e) };
        if let Err(e) = std::fs::create_dir(tanks_dir) { panic!("{}", e) };
        if let Err(e) = std::fs::create_dir(ducks_dir) { panic!("{}", e) };
        std::process::exit(0);
    }

    let mut tank: Tank = Tank::new(&tanks_dir, &tank_arg);
    let mut fishies: Vec<Fish> = Vec::new();
    let mut duckies: Vec<Duck> = Vec::new();

    for arg in fish_args {
        fishies.push(Fish::new(&fish_dir, &arg, &tank));
    }
    for arg in duck_args {
        duckies.push(Duck::new(&ducks_dir, &arg, &tank));
    }
    
    if let Err(e) = io::stdout().execute(Hide) { panic!("{}", e); }
    if let Err(e) = io::stdout().execute(Clear(crossterm::terminal::ClearType::All)) { panic!("{}", e); }
    loop {
        if let Err(e) = io::stdout().execute(MoveTo(0, 0)) { panic!("{}", e); }
        for row_idx in 0..tank.size.0 {
            for glyph_idx in 0..tank.size.1 {
                let mut printed = false;
                if tank.fg_anim[tank.fg_frame][row_idx][glyph_idx].glyph != ' ' {
                    tank.fg_anim[tank.fg_frame][row_idx][glyph_idx].print(); 
                    printed = true;
                }
                // combine fishies and ducks vector to reduce this copied code
                if !printed {
                    for duck_idx in 0..duckies.len() {
                        if let Some(glyph) = duckies[duck_idx].get_glyph(row_idx, glyph_idx) {
                            glyph.print();
                            printed = true;
                            break;
                        }
                    }
                }
                if !printed {
                    for fish_idx in 0..fishies.len() {
                        if let Some(glyph) = fishies[fish_idx].get_glyph(row_idx, glyph_idx) {
                            glyph.print();
                            printed = true;
                            break;
                        }
                    }
                }
                if !printed {
                    tank.bg_anim[tank.bg_frame][row_idx][glyph_idx].print();
                }
            }
            print!("\n");
        }
        for fish_idx in 0..fishies.len() {
            fishies[fish_idx].update(&tank);
        }
        for duck_idx in 0..duckies.len() {
            duckies[duck_idx].update();
        }
        tank.update();
        thread::sleep(time::Duration::from_millis(200));
    }
}
