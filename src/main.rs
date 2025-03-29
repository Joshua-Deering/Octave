mod audio;
mod circular_buffer;
mod fft;
mod file_analyzer;
mod file_io;
mod fir_filter;
mod img_generator;
mod parametric_eq;
mod players;
mod rta;
mod util;

use audio::{do_short_time_fourier_transform, ShortTimeDftData, WindowFunction};
use file_analyzer::analyze_file;
use file_io::{read_data, read_wav_meta, read_wav_sample_rate};
use img_generator::{
    generate_eq_fill_response, generate_eq_response, generate_rta_line, generate_spectrogram_img,
    generate_waveform_img, generate_waveform_preview,
};
use parametric_eq::{FilterType, ParametricEq};
use players::AudioPlayer;
use rta::ExternalRta;
use util::*;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    SampleRate,
};
use slint::{run_event_loop, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};

use std::cell::RefCell;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{
    fs::{create_dir, File},
    path::Path,
};

slint::include_modules!();

//standard initial 2-stage weighting curve for LKFS measurement
//param_eq.add_biquad(Biquad::with_coefficients(1.53512485958697, -2.69169618940638, 1.19839281085285, -1.69065929318241, 0.73248077421585, 48000));
//param_eq.add_biquad(Biquad::with_coefficients(1., -2., 1., -1.99004745483398, 0.99007225036621, 48000));

// Color palette:
// {"Prussian blue":"273043","Cool gray":"aaadc4","Chamoisee":"8f7e4f","Magenta haze":"a14a76","Carmine":"9b1d20"}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("./res/").exists() {
        println!("\"./res\" directory does not exist. Creating res directory...");
        create_dir("./res").unwrap();
        create_dir("./res/audio").unwrap();
        println!("\"./res\" directory created. Please place files you'd like to parse in \"./res/audio\"");
        println!(
            "(full path to \"./res/\": \"{}\")",
            Path::new("./res/")
                .canonicalize()
                .unwrap()
                .to_string_lossy()
        );
        //add these once I add saving files again
        //create_dir("./res/spectrograms").unwrap();
        //create_dir("./res/waveforms").unwrap();
    }

    let main_window = MainWindow::new()?;

    let player: Rc<RefCell<Option<AudioPlayer>>> = Rc::new(RefCell::new(None));
    let player_eq = Arc::new(Mutex::new(ParametricEq::new(vec![], 48000)));
    // TODO: implement this
    //let _player_rta: Arc<Mutex<Option<RTA>>> = Arc::new(Mutex::new(None));

    let rta: Rc<RefCell<Option<ExternalRta>>> = Rc::new(RefCell::new(None));

    // UI Initialization Logic (called when any menu is opened) ----------------
    let init_ptr = main_window.as_weak();
    main_window.on_init_menu(move |menu: i32| {
        let main_window = init_ptr.upgrade().unwrap();

        match menu {
            0 => {
                let files: Vec<SharedString> = query_directory("./res/audio/")
                    .into_iter()
                    .map(|e| SharedString::from(e))
                    .collect();

                let host = cpal::default_host();
                let device = host.default_output_device().unwrap();
                let supported_configs: Vec<_> =
                    device.supported_output_configs().unwrap().collect();

                let mut supported_files: Vec<SharedString> = vec![];
                for f in files {
                    let sample_rate = read_wav_sample_rate(f.clone().into());

                    if let Some(_) = supported_configs
                        .iter()
                        .find(|e| e.max_sample_rate() == SampleRate(sample_rate))
                    {
                        supported_files.push(f);
                    }
                }

                let model_rc = Rc::new(VecModel::from(supported_files));
                main_window.set_player_audio_files(ModelRc::from(model_rc.clone()));

                main_window.set_selected_file("".into());
            }
            1 => {
                let files: Vec<SharedString> = query_directory("./res/audio/")
                    .into_iter()
                    .map(|e| SharedString::from(e))
                    .collect();

                let model_rc = Rc::new(VecModel::from(files));
                main_window.set_vis_audio_files(ModelRc::from(model_rc.clone()));

                main_window.set_selected_file("".into());
            }
            3 => {
                let files: Vec<SharedString> = query_directory("./res/audio/")
                    .into_iter()
                    .map(|e| SharedString::from(e))
                    .collect();

                let model_rc = Rc::new(VecModel::from(files));
                main_window.set_f_analyzer_files(ModelRc::from(model_rc.clone()));
            }
            _ => {}
        }
    });

    // UI Closure logic (called when any menu is closed) -----------------------
    {
        let close_menu_player_ptr = Rc::clone(&player);
        let close_menu_window_ptr = main_window.as_weak();
        let rta_clone = Rc::clone(&rta);
        main_window.on_close_menu(move |menu: i32| {
            let main_window = close_menu_window_ptr.upgrade().unwrap();
            match menu {
                0 => {
                    *close_menu_player_ptr.borrow_mut() = None;
                    main_window.set_slider_pos(0.);
                    main_window.set_is_playing(false);
                    main_window.set_selected_file("".into());
                }
                2 => {
                    *rta_clone.borrow_mut() = None;
                }
                _ => {}
            }
        });
    }

    // Audio Playback Stop/Start Logic -----------------------------------------
    {
        let audio_player_ref = Rc::clone(&player);
        let play_ptr = main_window.as_weak();
        main_window.on_toggle_play(move |new_state: bool| {
            let main_window = play_ptr.upgrade().unwrap();

            // Cant play if no file is selected
            if main_window.get_selected_file().trim().is_empty() {
                main_window.set_is_playing(false);
                return;
            }
            if let Some(ref mut player) = *audio_player_ref.borrow_mut() {
                if !new_state {
                    player.pause();
                } else {
                    player.start();
                }
            }
        });
    }

    // Parametric Eq Node Initialization logic ---------------------------------
    main_window.on_init_eq_nodes(move |n: i32| {
        let mut nodes = Vec::new();
        let min_freq: f32 = 20.;
        let max_freq: f32 = 20000.;

        nodes.push(NodeData {
            f_type: "HPF".into(),
            gain: 0.,
            freq: min_freq + 10.,
            q: 1.0,
        });
        for i in 1..(n - 1) {
            let freq = (min_freq * (max_freq / min_freq).powf(i as f32 / (n as f32 - 1.0))).round();
            nodes.push(NodeData {
                f_type: "Peak".into(),
                gain: 0.,
                freq,
                q: 1.0,
            });
        }
        nodes.push(NodeData {
            f_type: "High Shelf".into(),
            gain: 0.,
            freq: max_freq / 2.,
            q: 1.0,
        });
        return ModelRc::new(Rc::new(VecModel::from(nodes)));
    });

    // Draw Eq Image
    main_window.on_request_eq_response(
        move |eq_nodes: ModelRc<NodeData>,
              min_freq: f32,
              max_freq: f32,
              min_gain: f32,
              max_gain: f32,
              imgx: f32,
              imgy: f32| {
            let mut drawn_eq = ParametricEq::new(vec![], 48000);
            if let Some(nodes) = eq_nodes.as_any().downcast_ref::<VecModel<NodeData>>() {
                for n in nodes.iter() {
                    drawn_eq.add_node(
                        FilterType::from_string(n.f_type.into()),
                        n.freq,
                        n.gain,
                        n.q,
                    );
                }
            }

            generate_eq_response(
                &drawn_eq,
                min_freq,
                max_freq,
                min_gain,
                max_gain,
                imgx as u32,
                imgy as u32,
            )
        },
    );

    // Draw Eq fill
    main_window.on_request_eq_fill_response(
        move |eq_nodes: ModelRc<NodeData>,
              min_freq: f32,
              max_freq: f32,
              min_gain: f32,
              max_gain: f32,
              imgx: f32,
              imgy: f32| {
            let mut drawn_eq = ParametricEq::new(vec![], 48000);
            if let Some(nodes) = eq_nodes.as_any().downcast_ref::<VecModel<NodeData>>() {
                for n in nodes.iter() {
                    drawn_eq.add_node(
                        FilterType::from_string(n.f_type.into()),
                        n.freq,
                        n.gain,
                        n.q,
                    );
                }
            }

            generate_eq_fill_response(
                &drawn_eq,
                min_freq,
                max_freq,
                min_gain,
                max_gain,
                imgx as u32,
                imgy as u32,
            )
        },
    );

    // Set audio player EQ
    let player_eq_ptr = Arc::clone(&player_eq);
    main_window.on_set_eq(move |eq_nodes: ModelRc<NodeData>| {
        let mut player_eq = player_eq_ptr.lock().unwrap();
        player_eq.reset();
        if let Some(nodes) = eq_nodes.as_any().downcast_ref::<VecModel<NodeData>>() {
            for n in nodes.iter() {
                player_eq.add_node(
                    FilterType::from_string(n.f_type.into()),
                    n.freq,
                    n.gain,
                    n.q,
                );
            }
        }
    });

    // Slider-to-audio behaviour -----------------------------------------------
    {
        let audio_player_ref = Rc::clone(&player);
        main_window.on_slider_released(move |value: f32| {
            if let Some(ref mut player) = *audio_player_ref.borrow_mut() {
                player.set_progress(value / 100.);
            }
        });
    }

    // Slider update logic -----------------------------------------------------
    let timer_ptr = main_window.as_weak();
    let audio_player_timer_ref = Rc::clone(&player);
    let timer = Timer::default();
    timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(50),
        move || {
            let main_window = timer_ptr.upgrade().unwrap();
            if let Some(ref mut player) = *audio_player_timer_ref.borrow_mut() {
                if player.is_finished() {
                    main_window.set_is_playing(false);
                    player.set_finished(false);
                }
                if !main_window.get_slider_pressed() {
                    main_window.set_slider_pos(player.get_progress() * 100.);
                }
            }
        },
    );

    // On Audio file select ----------------------------------------------------
    {
        let audio_player_ref = Rc::clone(&player);
        let file_sel_ptr = main_window.as_weak();
        let player_eq_ptr = Arc::clone(&player_eq);
        main_window.on_file_select(move |file: SharedString| {
            let sample_rate = read_wav_sample_rate(file.clone().into());
            player_eq_ptr.lock().unwrap().set_sample_rate(sample_rate);

            *audio_player_ref.borrow_mut() =
                Some(AudioPlayer::new(file.into(), Arc::clone(&player_eq_ptr)));
            let main_window = file_sel_ptr.upgrade().unwrap();
            let file_dur = audio_player_ref.borrow().as_ref().unwrap().duration;
            main_window.set_file_duration(file_dur);
        });
    }

    // Generate Waveform Preview for Audio Player
    {
        let window_weak = main_window.as_weak();
        main_window.on_render_waveform(move |file: SharedString, imgx: f32, imgy: f32| {
            let window_weak = window_weak.clone();
            thread::spawn(move || {
                let img = generate_waveform_preview(file, imgx, imgy);
                window_weak.upgrade_in_event_loop(|handle| {
                    handle.set_waveform_img(Image::from_rgba8(img));
                })
            });
        });
    }

    // Spectrogram Generation --------------------------------------------------
    {
        let window_weak = main_window.as_weak();
        main_window.on_generate_spectrogram(
            move |file: SharedString,
                  imgx: f32,
                  imgy: f32,
                  window_size: i32,
                  window_overlap: f32,
                  window_type: SharedString| {
                let main_window = window_weak.clone();

                thread::spawn(move || {
                    let mut reader =
                        BufReader::new(File::open(format!("./res/audio/{}", file)).unwrap());
                    let file_info = read_wav_meta(&mut reader);
                    let file_dur = file_info.audio_duration;
                    let sample_rate = file_info.sample_rate;

                    let samples = read_data(&mut reader, &file_info, 0., file_dur).unwrap();
                    let window_func = WindowFunction::from_str(window_type.as_str()).unwrap();

                    let stdft = do_short_time_fourier_transform(
                        &samples[0],
                        sample_rate,
                        window_size as f32 / 1000.,
                        window_overlap / 100.,
                        window_func,
                    );
                    let num_dfts = stdft.len() as u32;
                    let num_freqs = stdft[0].len() as u32;
                    let stdft_data = ShortTimeDftData::new(stdft, num_dfts, num_freqs, sample_rate);
                    let img = generate_spectrogram_img(imgx as u32, imgy as u32, stdft_data);

                    main_window
                        .upgrade_in_event_loop(move |handle| {
                            handle.set_vis_source(Image::from_rgba8(img));
                            handle.set_vis_loading(false);
                        })
                        .unwrap();
                });
            },
        );
    }

    // Waveform Generation -----------------------------------------------------
    {
        let window_weak = main_window.as_weak();
        main_window.on_generate_waveform(move |file: SharedString, imgx: f32, imgy: f32| {
            let main_window = window_weak.clone();

            thread::spawn(move || {
                let mut reader =
                    BufReader::new(File::open(format!("./res/audio/{}", file)).unwrap());
                let file_info = read_wav_meta(&mut reader);
                let file_dur = file_info.audio_duration;

                let samples = read_data(&mut reader, &file_info, 0., file_dur).unwrap();

                let img = generate_waveform_img(imgx as u32, imgy as u32, samples);

                main_window
                    .upgrade_in_event_loop(move |handle| {
                        handle.set_vis_source(Image::from_rgba8(img));
                        handle.set_vis_loading(false);
                    })
                    .unwrap();
            });
        });
    }

    // Start RTA ---------------------------------------------------------------
    {
        let rta_clone = Rc::clone(&rta);
        main_window.on_start_rta(move |_rta_type: SharedString, rta_response: SharedString| {
            let new_cache_size = match rta_response.as_str() {
                "Fast" => 6000,
                "Medium" => 24000,
                "Slow" => 48000,
                _ => 6000,
            };

            let mut rta = rta_clone.borrow_mut();
            let mut new_rta = false;

            if let Some(active_rta) = rta.as_mut() {
                if active_rta.cache_size != new_cache_size {
                    new_rta = true;
                } else {
                    active_rta.start();
                }
            } else {
                new_rta = true;
            }
            if new_rta {
                *rta = Some(ExternalRta::new(new_cache_size));
                rta.as_mut().unwrap().start();
            }
        });
    }

    // Stop RTA ---------------------------------------------------------------
    {
        let rta_clone = Rc::clone(&rta);
        main_window.on_stop_rta(move || {
            let mut rta = rta_clone.borrow_mut();
            if let Some(active_rta) = rta.as_mut() {
                active_rta.stop();
            }
        });
    }

    // Generate RTA SVG Path ---------------------------------------------------
    {
        let rta_clone = Rc::clone(&rta);
        main_window.on_req_rta_img(
            move |imgx: f32,
                  imgy: f32,
                  min_freq: f32,
                  max_freq: f32,
                  min_level: f32,
                  max_level: f32,
                  octave_bandwidth: f32| {
                let mut rta = rta_clone.borrow_mut();
                if let Some(active_rta) = rta.as_mut() {
                    let fft = active_rta.get_fft();

                    generate_rta_line(
                        imgx as u32,
                        imgy as u32,
                        min_freq,
                        max_freq,
                        min_level,
                        max_level,
                        octave_bandwidth,
                        fft,
                    )
                } else {
                    //if no active rta, return empty image
                    SharedString::new()
                }
            },
        );
    }

    // Analyze Audio File ------------------------------------------------------
    {
        let analyzer_clone = main_window.as_weak();
        main_window.on_analyze_file( move | file: SharedString | {
            let main_window = analyzer_clone.clone();

            thread::spawn(move || {
                let res = match analyze_file(format!("./res/audio/{}", file)) {
                    None => {
                        main_window.upgrade_in_event_loop(| handle | {
                            handle.set_analyzing_file(false);
                        }).unwrap();
                        return;
                    }
                    Some(res) => res
                };

                main_window.upgrade_in_event_loop(move | handle | {
                    let res_parsed = FileResults {
                        sample_type_str: res.metadata.sample_type_str.into(),
                        channels: res.metadata.channels as i32,
                        sample_rate: res.metadata.sample_rate as i32,
                        data_rate: res.metadata.data_rate as i32,
                        data_block_size: res.metadata.data_block_size as i32,
                        bit_depth: res.metadata.bit_depth as i32,
                        file_size: res.metadata.file_size as i32,
                        channel_map: ModelRc::new(Rc::new(VecModel::from(
                                    res.metadata.channel_map
                                    .iter()
                                    .map(|(_, x)| x.to_string().into())
                                    .collect::<Vec<SharedString>>()))),
                        channel_map_short: ModelRc::new(Rc::new(VecModel::from(
                                    res.metadata.channel_map
                                    .iter()
                                    .map(|(_, x)| x.short_name().into())
                                    .collect::<Vec<SharedString>>()))),

                        
                        audio_duration: res.metadata.audio_duration,
                        lkfs_i: res.lkfs_i as f32,
                        lkfs_s: res.lkfs_s as f32,
                        lkfs_m: res.lkfs_m as f32,
                        true_peaks: ModelRc::new(Rc::new(VecModel::from(res.true_peaks))),
                    };

                    handle.set_cur_f_results(res_parsed);
                    handle.set_analyzing_finished(true);
                    handle.set_analyzing_file(false);
                }).unwrap();
            });
        });
    }

    main_window.show()?;
    run_event_loop()?;

    Ok(())
}
