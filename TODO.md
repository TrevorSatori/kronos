# TODO

## Custom Source Iterator

- CUE support out of the box, so we no longer need the initial seek and the calculation of the end time
  - Should still be able to not need the initial seek if the next song in the queue/playlist would be the next song in the same CUE, meaning we could just keep playing the same source.
- Whatever we need in it so we don't need the periodic_access
- The "stack of layers" (of iterators of `Source`s) approach may make sense for Rodio, since it's a general library supporting a ton of different use cases, but Jolteon can greatly simplify it by coupling, having a single Source/Iterator that does everything it needs.  

Note: Symphonia seems to be the only decoder that supports seeking in Rodio (that we really care about), but it can fail.
Rodio's `Source for TrackPosition` does have its own `try_seek`, though, as well as `Source for SamplesBuffer`.
Are we using those (indirectly), or just Symphonia?

## AtomicDuration

An `AtomicDuration` struct may be more ergonomic than a `Mutex<Duration>`, and might be marginally faster.

```rs
AtomicDuration { 
    seconds: AtomicU64,  
    nanos: AtomicU32, 
}
```

It also may not, and it could be marginally slower. Atomics can spin-lock, while Futexes will suspend the thread.

Still, a Mutex, in Unix, uses a Futex, that has an AtomicU32 inside, so we'd just be adding an AtomicU64 and saving us a lock.

```rs
pub struct Mutex {
    futex: Atomic,
}

pub struct Mutex {
    futex: Atomic,
}

type Atomic = futex::SmallAtomic;

pub type SmallAtomic = AtomicU32;
```

And futexes do a syscall.

All in all, it probably won't do any difference at all for Jolteon. We only lock threads on shared resources between the rendering and the playing thread,
and that only happens up to FPS on the rendering side and up to once per song in the queue or once per keyboard input, which is pretty low. Always above
1ms.

So the only difference might be in ergonomics: 

```rs
// now, with mutex:
total_time: Arc::new(Mutex::new(total_time)),
self.total_time.lock().unwrap().clone()
*self.total_time.lock().unwrap() = song_list_to_duration(&songs);

// then, with atomics:
total_time: Arc::new(AtomicDuration::from_duration(total_time)), // or Into trait?
self.total_time.clone()
self.total_time = song_list_to_duration(&songs);
```

Alternatively, since we don't really need nanosecond precision (for display and seek), nor more than... a few hours? a whole day? (for total queue length) of seconds, 
we could just store the millis as an AtomicU64, and do `Duration::from_millis()`. 
There are 86_400_000 millis in a day. The u64 max is 18446744073709551615... that is 213_503_982_334 days? Should be fine lol 
Even a U32 should give us more than 40 days, in which case we'd be using a single AtomicU32, which is what a Mutex is already using inside,
so, at worst, we'd have the same performance of the Mutex.
