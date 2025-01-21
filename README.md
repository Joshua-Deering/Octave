# Wav Parser

Wav Parser is a console-based app that can do various processing on Wav files. Its main purpose is for me to to a deep dive into Discrete Fourier Transforms, and to get better at writing good, fast (ideally) rust code.

## What it can do:
* Discrete Fourier Transforms
* Play Audio from Wav Files
* Perform Short-Time Fourier Transforms with different window functions (Hann window, Square window)
* Generate Spectrograms from wav files

## Planned Features:
* A Fast Fourier Transform (FFT) algorithm
* Waveform generator
* Transient detection / visualizer
* Real-Time-Analyzer (RTA)
* Support for more Wav-type formats and possibly other formats (mp3, ALAC, AAC, etc.)

---

Spectrogram of a song:
<img width="1200" alt="Spectrogram of a song" src="https://github.com/user-attachments/assets/31289645-8f3b-4c19-a71e-a4109d95a2c8"/>
Spectrogram of some random oscillating frequencies:
<img widtg="1170" alt="Spectrogram of some random oscillating frequencies" src="https://github.com/user-attachments/assets/4ca14da9-b445-48f5-8853-07e63d223460">


The menu: (using [console-menu](https://github.com/Bdeering1/console-menu) by [Bryn Deering](https://github.com/Bdeering1))
<img width="992" alt="Menu" src="https://github.com/user-attachments/assets/8c2d5720-4462-4cfb-ab0d-32f95e4e51fb" />
