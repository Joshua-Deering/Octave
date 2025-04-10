# Octave

Octave is a full-stack app built for analyzing and visualizing audio. Its main purpose is for me to to a deep dive into the world of digial audio, and to get better at writing good, fast rust code. It started out as a simple Wav file parser, and has since evolved into something much bigger, with real-time processing, and in-app visualizations. It is built with a pure rust backend, and a [Slint](https://github.com/slint-ui/slint/tree/master) frontend.

## What it can do:
* Discrete Fourier Transforms
    * Naive DFT
    * Fast Fourier Transform (Radix-2 algorithm)
* Play Audio from Wav Files
* Perform Short-Time Fourier Transforms with different window functions (Hann window, Square window)
* Generate Spectrograms from wav files
* Generate Waveforms from wav files
* Real-Time Parametric EQ
* Real-Time-Analyzer (RTA)
* File Analyzer
     * LKFS/LUFS measurement
     * True Peak Measurement
     * File information (size, audio duration, etc.)

## Planned Features:
* File Resampling, Upsampling, compression, etc.
* Noise Generation (Sine tones, Pink Noise, White Noise, etc.)
* Customizable App color scheme
* Customizable colors for visualizations
* Support for more Wav-type formats and possibly other formats (mp3, ALAC, AAC, etc.)

## Images:
Audio Playback Menu:
<img width="1121" alt="eq img" src="https://github.com/user-attachments/assets/8e6a75cc-21eb-403f-a2a9-1d57209e4de9" />
Visualization Menu (with spectrogram generated):
<img width="1120" alt="spectrogram img" src="https://github.com/user-attachments/assets/d5fbfd9b-115f-4a07-86c7-4c1d8e5d9975" />
Real-Time Analyzer:
<img width="1119" alt="rta img" src="https://github.com/user-attachments/assets/4f28a98d-ea77-4788-a30b-064d39ccac51" />
File Analyzer:
<img width="1049" alt="image" src="https://github.com/user-attachments/assets/4b0a4eae-6bf3-4ba4-9a0d-11bb72e96ea0" />



