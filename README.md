# CHAOS_OSC v0.2.0

## Installation
_**Disclaimer:** this plugin will only work on 64-bit windows machines!_ \
Download the `.dll` file in the `bin/` directory and place it into your DAW's VST folder.

## Compiling The Source Code
_**Note:** you don't need to compile the source code if you just want to use the plugin, just download the `.dll`._ \
Make sure you have Cargo installed on your computer (the Rust compiler). Then in the root of the repository run `cargo build`. Once Cargo is done building, there should be a `chaos_osc.dll` file in the newly created `debug/` directory. Place this file into your DAW's VST folder.

## What is CHAOS_OSC ?
Before giving a technical description, we'd like to introduce the plugin in more
musical terms. CHAOS_OSC is essentially a harsh noise / drone generator. It's very
chaotic and unpredictable, experimenting a bit with the parameters reveals lots
of unique sweet spots. While CHAOS_OSC is most often quite abrasive, by adding
a bit of reverb and eq-ing the high end a bit, you can quickly make more ambient
sounding soundscapes. You will notice that the controls are labelled with some
somewhat obscure names, this is because the relationship between controls and
sound isn't super intuitive, and we want to incourage you to experiment with the
controls and find your own favorite settings. Now for the technical jargon.

CHAOS_OSC is a chaotic oscillator based on a double pendulum system of equations
(with some considerable modifications to make it work at audio rate). The pendulum
can be driven by two oscillators to introduce some tonal characteristics. The 
outputs are the sin function of the inner pendulum for the left channel and the
outer pendulum for the right channel. The two oscillators can be tuned and can
FM each other (introducing an additional layer of chaos in the form of feedback
FM).

## Controls Explained
If you really want to know what each control does, here is a short description of
each:
+ **L1 <=> L2** - change the length relationship between the two pendulums. The
sum of the lengths is always 2.0, in the middle position the two pendulums are
both 1.0 units long.
+ **- <=> +** - change the overall tone of the oscillator, this control affects
many of the internal parameters of the plugin, mainly it changes the size of the
time step, making the pendulums oscillate slower (to the left) or faster (to the
right). Additionally, this control filters the angular acceleration and angular
momentum in a way that avoids the pendulum stabilizing onto a consistent pitch.
+ **O1** - amount of oscillator 1. Crossfades the angular momentum of the inner
pendulum with a constant, to make it transition from oscillation to a constant
rotation. The most interesting settings are in between.
+ **O2** - the same as O1, but for the outer pendulum.
+ **F1** - coarse frequency of oscillator 1 (in octaves).
+ **F2** - coarse frequency of oscillator 2 (in octaves).
+ **F1.f** - fine frequency of oscillator 1.
+ **F2.f** - fine frequency of oscillator 2.
+ **M1** - amount of FM from oscillator 2 to oscillator 1.
+ **M2** - amount of FM from oscillator 1 to oscillator 2.
