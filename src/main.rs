mod c8;
mod font;
mod instr;

use std::{
    io::Read,
    fs::File,
    path::PathBuf,
    num::ParseIntError,
    time::{Instant, Duration},
    thread,
    u32,
};
use structopt::StructOpt;
use crate::{
    instr::V,
    c8::{C8, SCREEN_SIZE},
};

fn parse_hex(src: &str) -> Result<u32, ParseIntError> {
    u32::from_str_radix(src, 16)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "emul8", )]
struct Config {
    #[structopt(short, long, help = "Enable debugging features")]
    debug: bool,
    #[structopt(short, long, help = "Specify the number of cycles to execute per frame", default_value = "250")]
    cycles_per_frame: usize,
    #[structopt(parse(from_os_str), help = "The CHIP-8 binary to execute")]
    input: PathBuf,
    #[structopt(short, long, help = "The foreground color to use (in RGB hexadecimal)", default_value = "FFFFFFFF", parse(try_from_str = parse_hex))]
    fg_color: u32,
    #[structopt(short, long, help = "The background color to use (in RGB hexadecimal)", default_value = "00000000", parse(try_from_str = parse_hex))]
    bg_color: u32,
    #[structopt(short, long, help = "The number of frames that a pixel should stay active for to reduce flicker", default_value = "1")]
    flicker_timeout: u8,
}

struct State {
    paused: bool,
}

fn main() {
    let config = Config::from_args();

    let mut state = State {
        paused: config.debug,
    };

    let mut timeout = vec![0; SCREEN_SIZE.0 * SCREEN_SIZE.1];
    let mut buf = vec![255; SCREEN_SIZE.0 * SCREEN_SIZE.1];
    let mut win = minifb::Window::new(
        "Emul8",
        SCREEN_SIZE.0,
        SCREEN_SIZE.1,
        minifb::WindowOptions {
            scale: minifb::Scale::X16,
            ..Default::default()
        }
    ).unwrap();

    let mut c8 = C8::default();

    let bin: Vec<_> = File::open(config.input)
            .unwrap()
            .bytes()
            .collect::<Result<_, _>>()
            .unwrap();
    c8.load(&bin);

    if config.debug {
        display_mem(c8.mem());
    }

    let mut last_tick = Instant::now();
    while win.is_open() {
        // Tick
        if !state.paused {
            let now = Instant::now();
            for _ in 0..config.cycles_per_frame {
                let result = c8.tick(now.duration_since(last_tick));

                if config.debug {
                    match result {
                        Ok((opcode, instr)) => println!("{:#04X} :: {:X?} => {}", c8.pc(), opcode, instr),
                        Err(err) => println!("ERROR AT {:#04X}: {:#X?}", c8.pc(), err),
                    }
                }

                last_tick = now;
            }
        }

        // Update screen
        for (i, px) in c8.screen().iter().enumerate() {
            if *px {
                timeout[i] = 0
            } else {
                timeout[i] += 1;
            }
        }
        for (i, t) in timeout.iter().enumerate() {
            buf[i] = if *t > config.flicker_timeout {
                config.bg_color
            } else {
                config.fg_color
            };
        }

        if win.is_key_pressed(minifb::Key::R, minifb::KeyRepeat::No) {
            println!("-- Registers --");
            for v in 0..16 {
                println!("{:2} = {:#04X}", V(v), c8.v(v));
            }
            println!(" i = {:#06X}", c8.i());
            println!("pc = {:#06X}", c8.pc());
        }
        if win.is_key_pressed(minifb::Key::M, minifb::KeyRepeat::No) {
            display_mem(c8.mem());
        }
        if win.is_key_pressed(minifb::Key::P, minifb::KeyRepeat::No) {
            state.paused ^= true;
        }

        c8.set_keys([
            false, // 0
            false, // 1
            false, // 2
            false, // 3
            win.is_key_down(minifb::Key::Q),
            win.is_key_down(minifb::Key::W),
            win.is_key_down(minifb::Key::E),
            win.is_key_down(minifb::Key::A),
            win.is_key_down(minifb::Key::S),
            win.is_key_down(minifb::Key::D),
            false, // A
            false, // B
            false, // C
            false, // D
            false, // E
            false, // F
        ]);

        win.update_with_buffer(&buf).unwrap();

        win.set_title(if state.paused { "Emul8 (paused)" } else { "Emul8" });

        thread::sleep(Duration::from_millis(1000 / 60));
    }
}

pub fn display_mem(mem: &[u8]) {
    let row_width = 16;
    for row_addr in (0..mem.len()).step_by(row_width) {
        print!("{:#06X} |", row_addr);
        for i in 0..row_width {
            match mem.get(row_addr + i) {
                Some(b) => print!(" {:02X}", b),
                None => print!("   "),
            }
        }
        println!("");
    }
}
