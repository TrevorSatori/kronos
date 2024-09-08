# History & Rant

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
