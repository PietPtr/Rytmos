# Links

pickup construction information: https://www.electricherald.com/pickup-winding-guide-part-i-approach/

Pico 2 examples: https://github.com/ImplFerris/pico2-rp-projects/tree/main/blinky

# Physical Dev Platform

## Features

A platform to do real-time tests of the code and get very rudimentary feedback on playability of the instrument. Must have:

* Correctly spaced out frets
    * To test the note replacing algorithm
* At least 2 strings
    * To test how the algorithm behaves when two notes are ringing
* A pickup
    * To pickup the strings vibrations
* An I2S ADC
    * To be able to digitally process the pickup output
* An RPi Pico (probably RP2350 for better performance and hw floats)
    * As processing platform
* Tuners
    * To easily test different string tensions, how that plays, and how the algorithm responds.
* Basic bridge
    * Not to intonate, but at least to keep the strings in place and follow the fretboard radius.

It should not be:

* Focused on being ergonomic, too time consuming for now
* Pretty, idem

## Basic Form

Wooden plank for mounting the bridge, pickups, neck, ADC, and pico. A wooden beam as a neck. A 3D printed fretboard with frets and correct radius.