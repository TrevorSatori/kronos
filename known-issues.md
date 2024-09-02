# Dirty Screen / Random Out-of-Place Text

Any code running in a process can write to stderr and stdout --- including 3rd party Rust or C code (*cough* ALSA *cough*). 
This is beyond our control.

Furthermore, Ratatui renders changes only, to improve performance. There is no way to disable this.

This causes two issues:
- Random text will get printed at a random position.
- This may cause the screen to scroll. Sometimes, the screen will start scrolling one line per frame.

We can "fix" the latter by running `terminal.set_cursor(0, 0)`.
Fortunately, this takes care of the ugliest part of this issue.

But the latter is not so trivial to fix. Some options are:

### `terminal.clear()` 

Clearing the whole screen on each frame is an option, although not a true one.
This isn't a `.clear` at Ratatui level --- it's at Crossterm level.
It [sends a control sequence to the terminal](https://github.com/crossterm-rs/crossterm/blob/a2b0e6a537cb45bafb4451a26d2993afc1feab39/src/terminal.rs#L344-L353) to clear the screen.

Other than being overkill, it causes flickering.

### `std::process::Command`

In theory, we could spawn an entire new process to run rodio/cpal/alsa. 

This is far from trivial —or elegant—. It'd probably involve writing a separate binary and IPC for rodio's Sink. At that point, moving to egui / gtk4-rs / Slint UI / etc would make more sense.
