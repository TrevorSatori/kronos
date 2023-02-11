# Kronos
Terminal Music Player Written In Rust. Used for offline listening and local files.

# State
Main branch is stable. features and perfomance improvements on the way.

# Controls
1. Enter => play
2. space => pause / resume
3. a => add song to queue
4. g => skip track
5. arrow keys / vim keys => navigate browser and queue 

# TODO
1. recursive search for auto adding valid items
2. Keybind options
3. man page

# Note
Current build uses metadata crate for song length, while good requires users to install ffmpeg.

``shell 

    sudo apt install ffmpeg clang
``
![Alt text](Kronos.png?raw=true "Title")

