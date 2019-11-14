mod c8;
mod font;
mod instr;

use std::{
    io::Read,
    fs::File,
    path::PathBuf,
    time::{Instant, Duration},
    thread,
    u32,
};
use structopt::StructOpt;
use crate::{
    instr::V,
    c8::{C8, SCREEN_SIZE},
};

#[derive(Debug, StructOpt)]
#[structopt(name = "emul8", )]
struct Config {
    #[structopt(short, long, help = "Enable debugging features")]
    debug: bool,
    #[structopt(short, long, help = "Specify the number of cycles to execute per frame", default_value = "250")]
    cycles_per_frame: usize,
    #[structopt(parse(from_os_str), help = "The CHIP-8 binary to execute")]
    input: PathBuf,
}

struct State {
    paused: bool,
}

fn main() {
    let config = Config::from_args();

    let mut state = State {
        paused: config.debug,
    };

    let mut timeout = vec![0.0; SCREEN_SIZE.0 * SCREEN_SIZE.1];
    let mut buf = vec![0; SCREEN_SIZE.0 * SCREEN_SIZE.1];
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
                timeout[i] = 1.0
            } else {
                timeout[i] *= 0.5;
            }
        }
        for (i, t) in timeout.iter().enumerate() {
            buf[i] = u32::from_le_bytes([(*t * 255.0) as u8; 4]);
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
