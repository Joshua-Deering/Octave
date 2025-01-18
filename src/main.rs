mod file_io;
mod audio;
mod img_generator;
mod players;
mod util;

use crate::file_io::*;
use crate::players::*;
use crate::audio::*;
use crate::util::*;

use std::fs::File;
use std::io::{BufReader, stdin};
use std::sync::{Arc, Mutex};
use std::{thread, thread::sleep};
use std::time::Duration;

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Data, Device, Host, OutputCallbackInfo, SampleRate, StreamConfig};
use console_menu::{Menu, MenuOption, MenuProps};
use img_generator::generate_img;

fn main() -> std::io::Result<()> {
    let menu_options = vec![
        MenuOption::new("Play Audio", || play_audio_menu()),
        MenuOption::new("Perform Short-Time DFT", || do_stdft_menu()),
        MenuOption::new("Create Spectrogram", || create_spectrogram_menu()),
    ];

    let mut menu = Menu::new(menu_options, MenuProps {
        title: ".Wav Parser",
        exit_on_action: false,
        message: "<esc> to close",
        ..MenuProps::default()
    });

    menu.show();

    Ok(())
}

fn do_stdft_menu() {

    let files = query_directory("./res/audio");

    let mut menu_options: Vec<MenuOption> = vec![];
    for file in files {
        menu_options.push(MenuOption::new(file.clone().as_str(), move || do_stdft(&file)));
    }

    let mut audio_file_menu = Menu::new(
        menu_options,
        MenuProps {
            title: "Choose a File:",
            message: "<esc> to close",
            ..MenuProps::default()
        }
    );
    audio_file_menu.show();
    
}

fn do_stdft(file_choice: &str) {
    //clear the console output
    print!("{}[2J", 27 as char);

    println!("Enter the start point of the Short-Time DFT (seconds)");
    let start = read_stdin_f32();
    println!("Enter the duration of the Short-Time DFT (seconds)");
    let duration = read_stdin_f32();
    println!("Enter the window size (duration of each DFT)");
    let window_size = read_stdin_f32();
    println!("Enter the window overlap (percent, in from 0-1)");
    let overlap = read_stdin_f32();
    println!("Enter which window function to use: (0: Square, 1: BellCurve)");
    let window_func = WindowFunction::from(read_stdin_usize());
    println!("Enter the filename for the resulting Short-Time DFT (without the extension)");
    let mut dest_file = String::new();
    stdin().read_line(&mut dest_file).unwrap();
    

    let f = File::open(format!("./res/audio/{}", file_choice)).expect("Failed to open file!");
    let mut reader = BufReader::new(f);
    let file_info = file_io::read_wav_meta(&mut reader);
    let sample_rate = file_info.sample_rate;
    let channels = file_info.channels;

    let signal = read_data(&mut reader, file_info, start, duration).unwrap();
    let original_signal = SignalPlayer::new(signal, sample_rate, channels as usize);

    println!("Performing Short-Time DFT...");
    let stdft = original_signal.do_short_time_fourier_transform(window_size, overlap, window_func);
    println!("Short-Time DFT Complete!");

    println!("Writing to File...");
    write_stdft_to_file(format!("./res/stdfts/{}.stdft", dest_file.trim()), &stdft);
    println!("Done!");
    thread::sleep(Duration::from_millis(500));
}

fn play_audio_menu() {
    let files = query_directory("./res/audio");

    let mut menu_options: Vec<MenuOption> = vec![];
    for file in files {
        menu_options.push(MenuOption::new(file.clone().as_str(), move || play_audio(&file)));
    }

    let mut audio_file_menu = Menu::new(
        menu_options,
        MenuProps {
            title: "Choose a File:",
            message: "<esc> to close",
            ..MenuProps::default()
        }
    );
    audio_file_menu.show();
}

fn play_audio(file_path: &str) {
    println!("Audio player starting ('exit' to stop playback and return to menu)");
    let file_player = Arc::new(Mutex::new(FilePlayer::new(file_path.into())));

    let host: Host = cpal::default_host();
    let device: Device = host.default_output_device().expect("No audio output device available!");

    let mut supported_stream_range = device.supported_output_configs().expect("Error while querying output configs!");
    let supported_config: StreamConfig = supported_stream_range.find(|&e| e.max_sample_rate() == SampleRate(48000)).expect("No supported configs!").with_sample_rate(SampleRate(48000)).config();

    let file_player_clone = Arc::clone(&file_player);
    let stream = device.build_output_stream_raw(
        &supported_config, 
        cpal::SampleFormat::F32,
        move |data: &mut Data, _: &OutputCallbackInfo| {
            file_player_clone.lock().unwrap().next_chunk(data);
        },
        move |_err| {
            panic!("bad things happened");
        },
        None
    ).unwrap();
    println!("Playing Audio...");
    stream.play().unwrap();

    // listen to stdin for the exit command on another thread
    let inp_file_player = Arc::clone(&file_player);
    thread::spawn(move || {
        let mut inp = "".to_string();
        loop {
            stdin().read_line(&mut inp).unwrap();
            if inp.trim() == "exit" {
                inp_file_player.lock().unwrap().finished = true;
                println!("Exiting...");
                return;
            }
            inp = "".to_string();
            sleep(Duration::from_millis(100));
        }
    });

    // keep track of whether the audio has finished playing on the current thread
    loop {
        if file_player.lock().unwrap().finished == true {
            println!("Audio playback complete!");
            return;
        }
        sleep(Duration::from_millis(1000));
    }
}

fn create_spectrogram_menu() {
    let files = query_directory("./res/stdfts");

    let mut menu_options: Vec<MenuOption> = vec![];
    for file in files {
        menu_options.push(MenuOption::new(file.clone().as_str(), move || create_spectrogram(&file)));
    }

    let mut stdft_file_menu = Menu::new(
        menu_options,
        MenuProps {
            title: "Choose a File:",
            message: "<esc> to close",
            ..MenuProps::default()
        }
    );
    stdft_file_menu.show();
}

fn create_spectrogram(dir: &str) {
    print!("{}[2J", 27 as char);

    println!("Enter the width of the image:");
    let imgx = read_stdin_u32();
    println!("Enter the height of the image:");
    let imgy = read_stdin_u32();
    println!("Should the generated image have a logarithmic y-axis?");
    let is_log_scale = read_stdin_bool();
    println!("Enter the file name of the resulting image (without the extension)");
    let mut dest_file = String::new();
    stdin().read_line(&mut dest_file).expect("Failed to read input");

    let stdft = read_stdft_from_file(("./res/stdfts/".to_string() + dir).as_str());
    
    println!("Generating image...");
    generate_img(format!("./res/spectrograms/{}.png", dest_file.trim()), imgx, imgy, stdft, is_log_scale).expect("Failed to save image!");
    println!("Done!");
    thread::sleep(Duration::from_millis(500));
}
