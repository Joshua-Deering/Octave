# Wav Parser

Wav Parser is a console-based app that can do various processing on Wav files. Its main purpose is for me to to a deep dive into Discrete Fourier Transforms, and to get better at writing good, fast (ideally) rust code.

## What it can do:
* Discrete Fourier Transforms
    * Naive DFT
    * Fast Fourier Transform (Radix-2 algorithm)
* Play Audio from Wav Files
* Perform Short-Time Fourier Transforms with different window functions (Hann window, Square window)
* Generate Spectrograms from wav files

## Planned Features:
* Waveform generator
* Transient detection / visualizer
* Real-Time-Analyzer (RTA)
* Support for more Wav-type formats and possibly other formats (mp3, ALAC, AAC, etc.)

---

Spectrogram of a song:
![Spectrogram of a song](https://github.com/user-attachments/assets/eb9e5d96-d65a-440f-abde-c98774055e67)
Waveform of the same song:
![Waveform of a song](https://github.com/user-attachments/assets/09c471ac-299b-40bc-bc2a-6353cd6d70cf)
Spectrogram of some random oscillating frequencies:
<img widtg="1170" alt="Spectrogram of some random oscillating frequencies" src="https://github.com/user-attachments/assets/4ca14da9-b445-48f5-8853-07e63d223460">
![Waveform of the same oscillating frequencies](https://github.com/user-attachments/assets/d57da608-e608-4482-bbbc-f5595f1e4025)


The menu: (using [console-menu](https://github.com/Bdeering1/console-menu) by [Bryn Deering](https://github.com/Bdeering1))
<img width="1113" alt="The menu of wav_parser" src="https://github.com/user-attachments/assets/035fd9ba-bff6-46b7-895a-3f5f76ad42af" />

