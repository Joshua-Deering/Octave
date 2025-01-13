mod file_io;
mod audio;
mod img_generator;

use crate::file_io::*;
use crate::audio::*;
use crate::img_generator::*;

use std::fs::File;
use std::io::{stdin, BufReader};
use cpal::{traits::{DeviceTrait, HostTrait}, Data, Device, Host, OutputCallbackInfo, SampleRate, StreamConfig};

fn main() -> std::io::Result<()> {
    let f = File::open("./res/11 Raining On Prom Night.wav")?;
    let mut reader = BufReader::new(f);
    
    let file_info = file_io::read_wav_meta(&mut reader);
    let file_sample_rate = file_info.sample_rate;
    println!("{}", file_info);
    println!("{:?}", file_info.chunks);

    let signal = read_data(&mut reader, file_info, 0., 1200.).unwrap();
    let original_signal = SignalPlayer::new(signal, file_sample_rate, 2);

    //let original_signal = SignalPlayer::new(vec![generate_signal(&vec![FreqData::new(250., 0.3, 0.), FreqData::new(700., 0.3, 0.)], 48000, 1.)], 48000, 1);
    //let transform = original_signal.do_fourier_transform();
    //println!("{:?}", transform.into_iter().filter(|x| x.amplitude > 0.).collect::<Vec<FreqData>>());
    //let transform = original_signal.do_fourier_transform();
    //for t in &transform[0] {
    //    if t.amplitude > 0.01 {
    //        println!("{:?}", t);
    //    }
    //}


    //let new_signal = generate_multichannel_signal(&transform, original_signal.sample_rate as usize, 1.);
    //let mut signal_player = SignalPlayer::new(new_signal, 48000, 2);
    
    let stdft: ShortTimeDftData = original_signal.do_short_time_fourier_transform(0.05, 0.0);
    println!("{}", stdft);

    generate_img("./res/test.png", 1200, 800, stdft, true).unwrap();

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
