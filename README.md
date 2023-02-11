# Kronos
Terminal Music Player Written In Rust. For offline listening and local files.

# Controls
1. Enter => play
2. space => pause / resume
3. a => add song to queue
4. g => skip track
5. arrow keys / vim keys => navigate browser and queue 

# Install

1. clone the repo
``
    git clone https://github.com/TrevorSatori/Kronos
``
2. This app uses ffmpeg for file metadata and clang. To install on Debian based distros 

``
    sudo apt install ffmpeg clang
``

`On Arch`
``
    sudo pacman -S ffmpeg clang
``
3. change directories into Kronos repo 

``
    cd Kronos
``

4. run program
``
    cargo run
``

![Alt text](Kronos.png?raw=true "Title")

