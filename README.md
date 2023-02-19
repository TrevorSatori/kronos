[![Cargo Build](https://github.com/TrevorSatori/Kronos/actions/workflows/main.yml/badge.svg)](https://github.com/TrevorSatori/Kronos/actions/workflows/main.yml)
# Kronos - A Lightweight Terminal Music Player For Offline Listening

![Music](assets/music_tab.png?raw=true)
![Controls](assets/controls_tab.png?raw=true)

# Table of contents
1. [About](#About)
2. [Specification](#Specifications)
3. [Install](#Installation)
    1. [Arch](#Arch)
    2. [Binary](#Binary)
    2. [Build From Source](#Source)
4. [Customization](#Customization)
5. [Contribute](#Contribute)

## About 

Kronos is a modern take on terminal music players. Written entirely in the Rust programming language to ensure minimal resource usage and memory safety. Meant for consuming local audio files without the need for internet.  

## Specification

Supports the following formats

Format
--- | 
aac |
flac |
mp3 |
mp4 |
m4a |
ogg |
wav |


## Installation

*Works on Linux, untested on Mac*

### Arch

``
    paru -S satori-kronos-git
``

### Binary

1. Download Release 
``
    https://github.com/TrevorSatori/Kronos/releases/tag/v0.69
``
### Build From Source

1. Enter the following in Terminal, or download zip from the code dropdown at top of the page.
``
    git clone https://github.com/TrevorSatori/Kronos.git
``

2. Change into directory
``
    cd Kronos/
``

3. compile and run
``
    cargo run --release
``

## Customization

If the color scheme above isn't for you, it can be changed! Kronos default config path is

``
    ~/.config/kronos/config.toml
``

Below are all the color options for each parameter in the toml file. 

black | blue | green | red |
yellow | magenta | cyan | gray |
dark gray | light red | light green | light yellow | light blue | 
light magenta | light cyan | white | 
rgb 0 - 255 |


![Customized](assets/customized.png?raw=true)

The above makes use of both premade color options as well as rgb values and can be recreated with the following inside a config.toml.

```toml
[theme]
foreground = "200, 100, 250"
background = "black"
highlight_foreground = "white"
highlight_background = "255, 165, 0" 
```
                        
## Contribute

Currently this repo is being built by one full time college student. If you want to help besides leaving a star(Wink wink nudge nudge), make sure to open issues about all problems so they can be resolved immediately. Further, if anything could be added to make listening experience better feel free to fork the repo and send a pull request.

## Calling All Ricers
Actively looking for custom themes to integrate into Kronos, if you enjoy ricing leave a comment on the discussions page.




