mod c8;

use std::{
    io::Read,
    fs::File,
    time::{Instant, Duration},
    thread,
    env,
    u32,
};
use c8::C8;

#[derive(Debug)]
pub enum Error {}

const SIZE: (usize, usize) = (64, 32);

fn main() -> Result<(), Error> {
    let mut timeout = vec![0.0; SIZE.0 * SIZE.1];
    let mut buf = vec![0; SIZE.0 * SIZE.1];
    let mut win = minifb::Window::new(
        "Emul8",
        SIZE.0,
        SIZE.1,
        minifb::WindowOptions {
            scale: minifb::Scale::X16,
            ..Default::default()
        }
    ).unwrap();

    let mut c8 = C8::default();

    let bin: Vec<_> = File::open(env::args()
        .nth(1)
        .unwrap())
            .unwrap()
            .bytes()
            .collect::<Result<_, _>>()
            .unwrap();
    c8.load(&bin);

    let mut last_tick = Instant::now();
    while win.is_open() {
        // Tick
        let now = Instant::now();
        for _ in 0..100 {
            c8.tick(now.duration_since(last_tick)).unwrap();
            last_tick = now;
        }

        // Update screen
        for (i, px) in c8.screen().iter().enumerate() {
            if *px {
                timeout[i] = 1.0
            } else {
                timeout[i] *= 0.0;
            }
        }
        for (i, t) in timeout.iter().enumerate() {
            buf[i] = u32::from_le_bytes([(*t * 255.0) as u8; 4]);
        }

        if win.is_key_pressed(minifb::Key::R, minifb::KeyRepeat::No) {
            c8.display_regs();
        }
        if win.is_key_pressed(minifb::Key::M, minifb::KeyRepeat::No) {
            c8.display_mem();
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

        thread::sleep(Duration::from_millis(1000 / 60));
    }

    Ok(())
}
