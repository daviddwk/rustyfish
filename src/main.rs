use std::process::exit;
use std::time::{Duration, SystemTime};
use std::io::stdout;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::{
    create_dir, 
    create_dir_all,
    read_dir, 
    copy
};
extern crate structopt;
use structopt::StructOpt;

extern crate crossterm;
use crossterm::{
    ExecutableCommand,
    cursor::{
        Hide, 
        Show, 
        MoveTo
    },
    terminal::{
        Clear, 
        disable_raw_mode, 
        enable_raw_mode
    },
    event::{
        Event, 
        poll, 
        read, 
        KeyCode, 
        KeyEvent, 
        KeyModifiers, 
        KeyEventKind, 
        KeyEventState
    },
};
extern crate rand;
extern crate home;
extern crate colored;
extern crate serde_json;

mod tank;
use tank::Tank;
mod fish;
use fish::Fish;
mod duck;
use duck::Duck;
mod animation;
mod color_glyph;
mod error;
use error::error;
mod open_json;

#[derive(StructOpt)]
#[structopt(name = "freefish", version = "0.0.1", about = "Displays an animated fish tank to your terminal!")]
struct Opt {
    #[structopt(short = "l", long = "list", help = "Lists available assets found in ~/.config/freefish/")]
    list: bool,
    #[structopt(short = "i", long = "init", help = "Copies assets from ./config to ~/.config/freefish/")]
    init: bool,
    #[structopt(short = "t", long = "tank", help = "Selects a tank")]
    tank: Vec<String>, 
    #[structopt(short = "s", long = "speed", default_value = "200", help = "Sets the delay between frames in ms")]
    speed: u64,
    #[structopt(short = "f", long = "fish", help = "Adds the specified fish to your fish tank")]
    fish: Vec<String>,
    #[structopt(short = "d", long = "ducks", help = "Adds the specified ducks to your fish tank")]
    ducks: Vec<String>,
}

struct Creatures {
    fishies: Vec<Fish>,
    duckies: Vec<Duck>,
}

fn main() {
    let args = Opt::from_args();
    let freefish_dir = home::home_dir().unwrap().join(".config").join("freefish");
    let init_dir = PathBuf::from("./config");

    let mut asset_names: HashMap<&str, &Vec<String>> = HashMap::new();
    asset_names.insert("tanks", &args.tank);
    asset_names.insert("fish", &args.fish);
    asset_names.insert("ducks", &args.ducks);

    if args.init {
        init_assets(&init_dir, &freefish_dir, &asset_names);
    }
    if args.list {
        list_assets(&freefish_dir, &asset_names);
    }

    let mut tank = load_tank(&freefish_dir, &asset_names); 
    let mut creatures = Creatures {
        fishies: asset_names["fish"].iter().map(|name| Fish::new(&freefish_dir.join("fish"), name, &tank)).collect(),
        duckies: asset_names["ducks"].iter().map(|name| Duck::new(&freefish_dir.join("ducks"), name, &tank)).collect(),
    };

    enable_raw_mode().unwrap();
    stdout().execute(Hide).unwrap();
    stdout().execute(Clear(crossterm::terminal::ClearType::All)).unwrap();

    draw(&args.speed, &mut tank, &mut creatures);
    
    stdout().execute(Show).unwrap();
    disable_raw_mode().unwrap();
    exit(0);
}

fn init_assets(from_dir: &PathBuf, to_dir: &PathBuf, assets: &HashMap<&str, &Vec<String>>) {
    create_dir_all(to_dir).unwrap();
    for (asset_type, _v) in assets {
        if !to_dir.join(asset_type).exists() {
            create_dir(to_dir.join(asset_type)).unwrap();
        }
        for file in read_dir(from_dir.join(asset_type)).unwrap() {
            let f = &file.unwrap();
            copy(f.path(), to_dir.join(asset_type).join(f.file_name())).unwrap();
        }
    }
    // print somthing here
    exit(0);
}

fn list_assets(asset_dir: &PathBuf, assets: &HashMap<&str, &Vec<String>>) {
    // TODO fix listing with no directories
    // and other things where this is no freefish dir
    for (asset_type, _v) in assets {
        println!("{}:", asset_type.to_uppercase());
        for file in read_dir(asset_dir.join(asset_type)).unwrap() {
            let f = &file.unwrap();
            if f.path().extension() == Some(OsStr::new("json")) {
                println!("    {}", f.path().file_stem().unwrap().to_str().unwrap()); 
            }
        }
    }
    exit(0);
}

fn load_tank(assets_dir: &PathBuf, asset_names: &HashMap<&str, &Vec<String>>) -> Tank {
    if asset_names["tanks"].len() < 1 {
        error("A tank was not provided", 1);
    }
    else if asset_names["tanks"].len() > 1 {
        error("Too many tanks were provided", 1);
    }
    return Tank::new(&assets_dir.join("tanks"), &asset_names["tanks"][0]);
}

fn draw(delay: &u64, tank: &mut Tank, creatures: &mut Creatures) {
    loop {
        stdout().execute(MoveTo(0, 0)).unwrap();
        for row_idx in 0..tank.size.height {
            for glyph_idx in 0..tank.size.width {
                // blesh, do some weird generic programmeing here if you can 
                let mut printed = draw_tank_char(tank, row_idx, glyph_idx);
                if !printed {
                    for duck_idx in 0..creatures.duckies.len() {
                        if let Some(glyph) = creatures.duckies[duck_idx].get_glyph(row_idx, glyph_idx) {
                            glyph.print();
                            printed = true;
                            break;
                        }
                    }
                }
                if !printed {
                    for fish_idx in 0..creatures.fishies.len() {
                        if let Some(glyph) = creatures.fishies[fish_idx].get_glyph(row_idx, glyph_idx) {
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
            if row_idx != tank.size.height - 1 {
                print!("\r\n");
            }
        }
        for fish in &mut creatures.fishies {
            fish.update();
        }
        for duck in &mut creatures.duckies {
            duck.update();
        }
        tank.update();
        
        // there must be a better way
        let frame_duration = Duration::from_millis(*delay);
        let frame_start = SystemTime::now();
        let mut now = SystemTime::now();
        while now.duration_since(frame_start).unwrap() < frame_duration {
            if poll(frame_duration - now.duration_since(frame_start).unwrap()).unwrap() {
                match read().unwrap() {
                    // TODO: Capital Q does not work here
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => return,
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => return,
                    _ => (),
                }
            }
            now = SystemTime::now();
        }
    }
}

fn draw_tank_char(tank: &Tank, row_idx: usize, glyph_idx: usize) -> bool {
    if tank.fg_anim[tank.fg_frame][row_idx][glyph_idx].glyph != ' ' {
        tank.fg_anim[tank.fg_frame][row_idx][glyph_idx].print(); 
        return true;
    }
    return false;
}
