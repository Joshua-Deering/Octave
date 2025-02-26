# Octave

Octave is an app that has many features for analyzing and visualizing audio. Its main purpose is for me to to a deep dive into Discrete Fourier Transforms, and to get better at writing good, fast (ideally) rust code. It started out as a simple Wav file parser, and has since evolved into something much bigger, with real-time processing, and in-app visualizations. It is built with a pure rust backend, and a [Slint](https://github.com/slint-ui/slint/tree/master) frontend.

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

## Planned Features:
* Customizable App color scheme
* Customizable colors for visualizations
* File Analyzer (LUFS, True Peak, etc.)
* Support for more Wav-type formats and possibly other formats (mp3, ALAC, AAC, etc.)

## Images:
Audio Playback Menu:
<img width="1200" alt="image" src="https://github.com/user-attachments/assets/5e43f08e-8edf-497d-935c-7561f30b2204" />
Visualization Menu (with spectrogram generated):
<img width="1200" alt="image" src="https://github.com/user-attachments/assets/ff84e172-fd47-480b-80e7-644866bec7ba" />
Real-Time Analyzer:
<img width="1200" alt="RTA Demo" src="https://github.com/user-attachments/assets/94a4477c-9a4d-4de2-9cb3-013829ac6a7b" />



