mod file_io;
mod audio;

use crate::file_io::*;
use crate::audio::*;

use std::fs::File;
use std::io::{stdin, Write, BufReader};
use cpal::{traits::{DeviceTrait, HostTrait}, Data, Device, Host, OutputCallbackInfo, SampleRate, StreamConfig};

fn main() -> std::io::Result<()> {
    let f = File::open("./res/11 Raining On Prom Night.wav")?;
    let mut reader = BufReader::new(f);
    
    let file_info = file_io::read_wav_meta(&mut reader);
    println!("{}", file_info);
    println!("{:?}", file_info.chunks);

    let signal = read_data(&mut reader, file_info, 0., 1.).unwrap();
    let mut original_signal = SignalPlayer::new(signal, 48000, 2);

    //let transform = original_signal.do_fourier_transform();
    //for t in &transform[0] {
    //    if t.amplitude > 0.01 {
    //        println!("{:?}", t);
    //    }
    //}
    //
    //let new_signal = generate_multichannel_signal(&transform, original_signal.sample_rate as usize, 1.);
    //let mut signal_player = SignalPlayer::new(new_signal, 48000, 2);
    //
    //let mut f = File::create("./res/test.csv").unwrap();
    //for i in 0..original_signal.samples[0].len() {
    //    f.write(format!("{},{}\n", original_signal.samples[0][i], signal_player.samples[0][i]).as_bytes()).unwrap();
    //}
    
    let stdft: Vec<Vec<Vec<FreqData>>> = original_signal.do_short_time_fourier_transform(0.01, 0.0);
    println!("{}, {}, {}", stdft.len(), stdft[0].len(), stdft[0][0].len());
    let mut max = 0.;
    for i in 0..stdft.len() {
        for j in 0..stdft[i].len() {
            for k in 0..stdft[i][j].len() {
                if stdft[i][j][k].amplitude > max {
                    max = stdft[i][j][k].amplitude;
                }
            }
        }
    }

    let imgx = stdft[0].len() * 9;
    let imgy = stdft[0][0].len();
    
    println!("{}", (1. / max) * 256.);
    let mut imgbuf = image::ImageBuffer::new(imgx as u32, imgy as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = stdft[0][x as usize % 9][y as usize].amplitude * 2560000.;
        *pixel = image::Rgb([r as u8, 0 as u8, 0 as u8]);
    }
    imgbuf.save("../test.png").unwrap();


    println!("done");
    panic!("");

    let host: Host = cpal::default_host();
    let device: Device = host.default_output_device().expect("No audio output device available!");

    let mut supported_stream_range = device.supported_output_configs().expect("Error while querying output configs!");
    let supported_config: StreamConfig = supported_stream_range.find(|&e| e.max_sample_rate() == SampleRate(48000)).expect("No supported configs!").with_sample_rate(SampleRate(48000)).config();

    let _stream = device.build_output_stream_raw(
        &supported_config, 
        cpal::SampleFormat::F32,
        move |data: &mut Data, _: &OutputCallbackInfo| {
            //signal_player.next_chunk(data);
        },
        move |_err| {
            panic!("bad things happened");
        },
        None
    ).unwrap();

    let mut inp: String = "".into();
    loop {
        stdin().read_line(&mut inp)?;
        if inp == "exit\n" {
            break;
        }
        inp = "".into();
    }

    Ok(())
}
