mod emulator;
mod soc;
mod debug;

use minifb::{Key, Window, WindowOptions};
use std::{fs::File, io::Read, env};

use std::io::{stdin, stdout, Write};
use std::thread;
use std::sync::{Arc, Mutex};

use crate::emulator::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::debug::DebuggerCommand;

// Window parameters
const SCALE_FACTOR: usize = 3;
const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];

fn main() {
    // get arguments from the command line   
    let (boot_rom_path, game_rom_path, debug_mode) = parse_args();

    let mut file = File::open(boot_rom_path).unwrap();
    let mut bin_data = [0xFF as u8; 256];
    if let Err(message) = file.read_exact(&mut bin_data) {
        panic!("Cannot read file with error message: {}", message);
    }

    let mut rom_file = File::open(game_rom_path).unwrap();
    let mut rom_data = [0xFF as u8; 32768];
    if let Err(message) = rom_file.read_exact(&mut rom_data) {
        panic!("Cannot read file with error message: {}", message);
    }
    println!("rom file len: {:#06x}", rom_file.metadata().unwrap().len());

    // launch the debugger cli
    let debug_cmd = Arc::new(Mutex::new(Vec::new()));
    if debug_mode {
        let debug_cmd_ref = Arc::clone(&debug_cmd);
        thread::spawn(move || {
            println!("Rustboy debugger CLI");

            loop {
                // get next instruction from console
                let mut command = String::new();
                command.clear();
                stdout().flush().unwrap();
                stdin().read_line(&mut command).expect("Incorrect string is read.");

                // process command
                if command.trim().eq("break") {
                    println!("break command");
                }

                if command.trim().eq("run") {
                    (*debug_cmd_ref.lock().unwrap()).push(DebuggerCommand::RUN);
                }

                if command.trim().eq("halt") {
                    (*debug_cmd_ref.lock().unwrap()).push(DebuggerCommand::HALT);
                }

                if command.trim().eq("step") {
                    (*debug_cmd_ref.lock().unwrap()).push(DebuggerCommand::STEP);
                }

                if command.trim().eq("help") {
                    println!("supported commands: break <addr>, run, halt, step");
                }
            }
        });
    }

    // create the emulated system
    let mut emulator = Emulator::new(&bin_data, &rom_data, debug_mode);

    // run the emulator
    let mut buffer = [0; SCREEN_HEIGHT * SCREEN_WIDTH];

    let mut window = Window::new(
        "Rustboy",
        WINDOW_DIMENSIONS[0],
        WINDOW_DIMENSIONS[1],
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // run emulator until a new frame is ready
        emulator.run(&mut *debug_cmd.lock().unwrap());

        if emulator.frame_ready() {
            // copy the current frame from gpu frame buffer
            for i in 0..SCREEN_HEIGHT * SCREEN_WIDTH {
                buffer[i] =  255 << 24
                            | (emulator.get_frame_buffer(i) as u32) << 16
                            | (emulator.get_frame_buffer(i) as u32) << 8
                            | (emulator.get_frame_buffer(i) as u32) << 0;
            }
            // display the frame rendered by the gpu
            window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
        }
    }
}

fn parse_args() -> (String, String, bool) {
    let mut boot_rom_path = String::new();
    let mut game_rom_path = String::new();
    let mut debug_opt = false;

    for (index, argument) in env::args().enumerate() {
        match index {
            1 => {
                boot_rom_path = argument.clone();
                println!("boot_rom: {}", boot_rom_path);
            }
            2 => {
                game_rom_path = argument.clone();
                println!("game_rom: {}", game_rom_path);
            }
            3 => if argument.eq("--debug") {
                    debug_opt = true;
            }
            _ => {} // nothing to do
        }
    }

    (boot_rom_path, game_rom_path, debug_opt)
}