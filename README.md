# Jolteon 

Music player that runs on the terminal.

![File Browser - Light Theme](assets/file_browser_light.png?raw=true)
![File Browser - Dark Theme](assets/file_browser_dark.png?raw=true)
![File Browser - Search](assets/file_browser_search.png?raw=true)

# Table of contents
1. [About](#About)
1. [Installation](#installation)
1. [Features](#Features)
1. [Supported Audio Formats](#supported-audio-formats)
1. [Customization](#customization)
1. [Philosophy](#philosophy)

## About 

Music player that runs on the terminal.

### Why Jolteon

I needed a replacement for `cmus`.

### Why a new music player?

I couldn't find one that I really liked.

### Why on the terminal?

Not sure, but I recently started thinking it's because TUIs are keyboard-first, and keyboard > mouse/trackpad.
Most non-terminal UIs are mouse/trackpad first. Keyboard shortcuts are an afterthought. 

And I _suspect_ I prefer the keyboard over the mouse/trackpad because of my ADHD and astigmatism. 

## Installation

*Tested only on Linux*

### Binary

Not yet available. Will add CI/CD in the future. Not a priority because I'm always running this with `cargo run`.

### Build From Source

```
git clone --depth=1 https://github.com/lautarodragan/jolteon.git
cd jolteon
cargo run --release
```

### Public Repositories

Won't be publishing this to `apt`, `yum`, etc. any time soon.

### Auto Updates

In the future, Jolteon will support detecting new versions automatically and offer downloading them.

## Features

- File Browser
- Search/Filter in File Browser (Ctrl+F)
- Playing Queue
- Controls
  - Play/Pause
  - Seek 5 seconds forward/backward
  - Media keys Play/Pause support via MPRIS in Linux
- `.cue` file support
- Gapless playback
- Persist app state:
  - The current directory of the browser
  - The queue
  - Current song (coming soon)
- Safe handling of application crashes, restoring the terminal to its normal state.

### Upcoming

- Playlists (like `cmus`)
- Audio library (like `cmus`)
- Media metadata overrides
  - This is something I've always wanted. I have many albums that belong to a same artist, but have the artist field spelled differently, so they'll show up as different entries in the `artists` list. And I absolutely do NOT want to edit the files, because reasons. So, for example, if I have `Dark Side of the Moon` by `Pink Floyd` and `The Piper at the Gates of Dawn` by `PINK FLOYD`, I want them both to show under `Pink Floyd`, and see `PINK FLOYD` nowhere, other than the list of overrides.  

## Supported Audio Formats

Jolteon uses Rodeo to play music, so it supports whatever formats Rodeo does.

I mainly use `flac` files, and some `mp3`. Other formats aren't usually tested, but the following should work: `aac`, `flac`, `mp3`, `mp4`, `m4a`, `ogg`, `wav`.

## Customization

The theme can be customized. Check out `config.rs`.

In the future, keyboard shortcuts will also be customizable.
Maybe texts too, since that'd enable translations, and there are so few of them anyway.
                  
## Philosophy

- Support features, UI and UX similar to `cmus`
- Statically linked, dependency free, single file binary that anyone can just download, `chmod a+x` and run.

### History & Rant

See [History & Rant](./HISTORY.md).

## Contribute

In general, I won't accept contributions, because I don't have enough time or Rust knowledge to properly do code review. 

You're free to have your own fork of Jolteon, though. Even if you're new to Rust, it's very friendly language with a very friendly community; and I try to keep the source code as clean and intuitive as I can, so modifying it should be relatively easy. 

To install Rust and Cargo, see https://www.rust-lang.org/tools/install or https://doc.rust-lang.org/cargo/getting-started/installation.html. Under macOS, Linux, etc, it's just copy-pasting a `curl ...` command. I won't copy it here for safety reasons.

Then, just clone the repo:

```
git clone --depth=1 https://github.com/lautarodragan/jolteon.git
cd jolteon
```

And `cargo run`, `cargo run --release` and `cargo build` are the commands you'll be running the most.

Keep in mind I'm using my own fork of `cpal` right now. I have an open [PR for cpal](https://github.com/RustAudio/cpal/pull/909), with a small bugfix, that hasn't been merged yet. 
