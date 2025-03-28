# Octave

Octave is an app that has many features for analyzing and visualizing audio. Its main purpose is for me to to a deep dive into Discrete Fourier Transforms, and to get better at writing good, fast rust code. It started out as a simple Wav file parser, and has since evolved into something much bigger, with real-time processing, and in-app visualizations. It is built with a pure rust backend, and a [Slint](https://github.com/slint-ui/slint/tree/master) frontend.

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
* Customizable App color scheme
* Customizable colors for visualizations
* Support for more Wav-type formats and possibly other formats (mp3, ALAC, AAC, etc.)

## Images:
Audio Playback Menu:
<img width="1270" alt="Audio Playback Screen with Parametric EQ" src="https://github.com/user-attachments/assets/0d50cd76-3437-4c49-b99d-cd5c42afd55d" />
Visualization Menu (with spectrogram generated):
<img width="1270" alt="Audio Visualization menu with Spectrogram Displayed" src="https://github.com/user-attachments/assets/93a60419-2656-4f55-b04c-2f3813544b43" />
Real-Time Analyzer:
<img width="1270" alt="Real-Time Analyzer Demo" src="https://github.com/user-attachments/assets/73a75cac-3bf2-4132-a3ff-8eb24f30953b" />




