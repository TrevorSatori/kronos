# Known Bugs

Jolteon is a work-in-progress, and currently undergoing heavy development.

I use Jolteon every day, almost all day, to listen to music, so I generally find bugs quickly, and fix them quickly, if I can.

There are probably many bugs there yet, and I'm probably going to introduce new ones as I keep adding features and refactoring code.

Having said that, the rate at which I find bugs has lowered considerably over time, and the speed with which I fix them, gone up.

Some known issues:
- I still use `unwrap` in some places I'd prefer not to. This will panic if something unexpected happens.
- Deadlocks. There may be some out there that I haven't caught.
