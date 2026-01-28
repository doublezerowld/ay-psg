[This](https://youtu.be/8JomtZwbxoY) is a project I've been working on for the past few weeks. With the help of the rust official community server ( #embedded & #rust-help-1 ) and my own hard work, I'm happy to say that I've completed an early working version that supports\* MIDI :ferrisCat: 

## Some lore
-# Skip if you're here for the technical stuff!

It all started with me deciding to learn Rust a bit over a month (or two) ago. After understanding the basics and writing a few smaller projects, I decided to take on something much bigger. Because I had this idea of making a chiptune player in my head for a long time, I finally committed to it.

I looked for chips I could easily buy online and came across the YM2149F from Yamaha: a simple, easy to use and affordable PSG, clone of the General Instruments AY-3-8910.

Writing the code for this thing was a pain at first, as I had zero experience with any Raspberry Pi, let alone writing ``no_std`` code for the Pico in Rust, which I gotta say is *not the best experience*. It mainly boils down to it being niche, which means barely any approachable tutorials exist, hence I had to rely on the docs & asking people how to do basic stuff instead. Really wish one of y'all would make some! I would, if only I had enough experience...
## Technical stuff

The chip itself has 3 channels that each have their own (50% duty cycle square wave) tone generators and a shared noise & envelope generator. It takes data in through an 8 bit data bus, and along with the bus control decoder lets you write data to any one of its 16 registers. By controlling these pins and supplying a clock (1-4MHz, I generated mine using a 2MHz XOSC), one can get the chip to generate all sorts of fun noises.

The generic driver works across all architectures because it's ``no_std`` and it's essentially just a hardware abstraction layer. You can use it in your own code by implementing the ``CommandOutput`` trait, which requires the handling of a ``Command`` parameter by ``fn execute(&mut self, command: Command);``. ``Command`` is a simple wrapper struct that contains the fields `register` and `value`, which as the names imply correspond to a value to be written to a specific register. 

The way I have it set up in the video is:
- I flashed the Pi Pico with a program that creates a fake serial device over USB (CDC-ACM), then reads 2 bytes at a time, forever.
- On my computer, I configured a fake MIDI device with `snd-seq-dummy` and wrote a bridge that turns the MIDI commands from Renoise (the DAW you can see in the demo) into bytes that are then sent to ``/dev/ttyACM0`` (which just so happens to be the RPi :ferrisHmm:)
- Speaking of the devil, I have Renoise set up in a way that allows me to use all 3 channels of the YM2149. The demo song I used ("I understand now" by Sunjammer, btw!) had all notes on one channel used one instrument, so I cloned the instrument two times, configured each version to output to its' own MIDI channel, and manually edited the notes in the file to switch between all 3 (in hindsight, I should've probably made the MIDI bridge do that :ferrisClueless:)

The coolest part, though, is that I managed to have each instrument output audio to its own audio channel (in the DAW) using some trickery, then used signal followers, math and the MIDI instrument control plug-in to have the level of each channel (in the chip) match the one in Renoise. And since the instruments all had envelopes and fx, that means I didn't have to write any envelopes of my own manually!

I could go more in depth, but I'd be rewriting the datasheet at that point :ferrisCluelesser: 

I'll release the source code I wrote for both the MIDI bridge and RPi soon™. For now, [here's the generic driver code!](https://github.com/OrdinarySoftwareDev/ym2149-core-rs)

I'm open to feedback and suggestions, so feel free to reply!
