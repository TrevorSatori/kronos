# Jolteon 

A Terminal Music Player For Offline Listening

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

## Features

- File Browser
- Search/Filter in File Browser (Ctrl+F)
- Playing Queue
- Media keys Play/Pause support via MPRIS in Linux
- `.cue` file support
- gapless playback
- Persist app state. Closing and reopening Jolteon should feel similar to simply hitting pause and play. In particular, Jolteon persists the following:
  - The current directory of the browser
  - The queue
  - Current song (coming soon)
- Controls
  - Seek 5 seconds forward/backward
- UI improvements (according to my own personal preference). See screenshots.
- Performance improvements. Removed some redundant disk access.
- Panic handler to restore the terminal if/when the application crashes, rather than leaving it in a rather unusable state.

I've also extensively refactored the code base:
- Migrated from Tui to Ratatui
- Removed a thread that tracked the current playing position. 
  - _This was possible thanks to the migration to Ratatui._
- Broke down rendering code into smaller pieces.
- Moved some logic into `app`
- Reduced the number of mutable references overall
  - And plan to do so as much as I can. I want everything to have the least possible access.

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
                  
## Philosophy

- Support features, UI and UX similar to `cmus`
- Statically linked, dependency free, single file binary that anyone can just download, `chmod a+x` and run.

### History & Rant

I used `cmus` daily for years and loved it. Some time in the past I found a bothersome bug/limitation in it. Looking for a fix,
I realized I was using a very old version. `cmus` has CI/CD and automatically publishes builds, but they were not available — GitHub deleted them
after a while.

I was feeling brave and decided to `git clone` and build it myself. Having practically no experience with C/C++, 
I found this particularly challenging, but eventually succeeded.

Then, after a while, I upgraded from PopOS to Ubuntu 24, and found the `cmus` I had built wasn't working anymore.
I tried building again, but couldn't install one `apt` dependency due to one of them not being available in the repos
set up by Ubuntu. Thanks to the modular nature of `cmus`, I could run it and listen to music... except for `flac`, and
that is what I use the most.

I work with JavaScript/TypeScript every day. In this world, I can `npm install` anything and it'll be contained in the project's structure.
I can even `nvm use 16/20/22` to switch between NodeJS versions. If a project has a build step, it's usually no more than `npm run build`.
Most of the time, things Just Work™.

I find the way these old applications require me to install a bunch of dependencies in my machine just to run the build, 
and a bunch of others to be able to run the application, absolutely crazy.

So I went looking and found Kronos. Having zero Rust experience, I was able to download and run the binary. No dependencies to install. No nothing.
Then I wanted to dig into the code. With zero Rust experience, I just `git clone`'ed, `cargo run` and that was it. 
Took me less than 5 minutes to go from not even having `cargo` or `rust` (let alone RustRover or any other Rust IDE) installed to
making modifications to the code and running them.

I initially submitted a couple of fixes to upstream Kronos, but later decided I just wanted the freedom to commit and push to `main`
and break the application if needed, prioritizing speed, fun and the features _I_ wanted over process, quality and community.

This is my fork of Kronos. It is not aimed at being easily consumable by anyone, or customizable. At least not for now.
I use Jolteon daily, pretty much all day, and work on the features I want to have, and fix the bugs as I find them.

This is my first Rust application, and I'm learning as I go, so the code can be pretty bad at times.
But, so far, I'm amazed by how beautiful a language Rust is. And I don't say this lightly! 
I've been writing software for 20+ years. 

## Contribute

In general, I won't accept contributions, because I don't have enough time or Rust knowledge to properly do code review. 
