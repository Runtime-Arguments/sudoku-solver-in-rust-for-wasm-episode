# README

## About this Repo

### The origin and goal for this is Rust program is:

- Our friend Dave often learns new languages by writing a Sudoku solver in that new language
- He wrote one in Python
- He decided to compare his hand-coded Python implementation to an AI-generated version
- Jim and I decided a good topic for Runtime Arguments might be WASM
- C and Rust are two languages that can both be compiled down into WASM
- Starting with Dave's AI-generated solver, Jim hand-coded a (mostly identical) implementation in C
  - He changed it to run multiple times
  - He added timing
- I (Wolf) took his C implementation and converted it into Rust
  - I was unable to do so (emotionally) without making a few changes
  - My version also runs multiple times, and also has timing

### Steps to Install Rust and Run this Program:

- installed Rust tools live in `~/.cargo/bin`. Add that to `$PATH`.
- `rustup` is the tool you use to install everything else, and to stay up-to-date.
  - You can install `rustup` via `brew`...
  - ...but I used the following command: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - `rustup update` installs everything you need.
  - Now you have `cargo` (`rustup update` installed it). `cargo` is the tool that basically does everything else.
- `cd` into the project directory (that is, the top-level directory containing `Cargo.toml`.)
- `cargo run --release`

### Changes:

- These are things I didn't like about the original implementation that I changed to be truer to the type-system.
  - I disagreed with many of the names. I care about names a lot. So some things have new names.
    - `Cell` is a new type. In the original code this role is served by a simple integer. Yes, for me, it's basically a
      `u8` so it can represent the digits from `1` to `9`, but in the original, `0` means "empty". I made `Cell`
      something like an `Option` with an explicit case for `Empty`.
    - `Board` is a new type, instead of using an untyped global variable `grid`
    - `Position` instead of `Coord`
      - `Coord` had a `found` field. The idea was that a `Coord` either gave the location of a thing for which you were
        searching, or else `found` was `0` (meaning `false`), which said you didn't find that thing (and `row` and `col`
        should be ignored). Rust has something for exactly this situation: `Option<T>`.
  - All of the functions operate on the one global `grid` variable, which doesn't even have a type. I don't like that.
    In this implementation, there is no global. Functions are passed in what they need as an argument.
  - `grid` is an array of arrays.  `Board` holds a single `Vec<Cell>` of length `81`.  `Position` knows how to translate.
    You access `Cell`s by `Position` indexing into `Board`, e.g., `let cell = board[position];`.  My accessing code
    is basically exactly the same, it just appears as implementation of the `trait`s `Index` and `IndexMut`.
  - The original code has `print_grid`. I'll use the exact same code, but I'll call it `fmt` so I get it automatically
    when I use `print!("{}", &board);`.
  - I'm trying to write this as simply as possible, so I'm keeping it to one file. But the Sudoku-specific types and
    functions deserve to be their own module.

### Things I _didn't_ do:

- I didn't try to make this work on arbitrary generic types or vector sizes
- I didn't try to use the type system to ensure that what goes in a cell is 1..=9
- I didn't try to use the type system to ensure that rows and columns were exactly length 9
- I didn't try to make a `Board` work with both borrowed and owned data (it's always a `Vec`; never a slice).
  I didn't want to clone, and I only use one "work" `Board` at a time, so I implemented a function to quickly copy
  the contents of one `Board` to another.
- I didn't try to write all those `Board` iterators.  So many!  I **really** wanted to.

### Results

On my M4 Max MacBook Pro, the summary of a run was this:

  Rust solved 1_000_000 iterations in 137.998281667s seconds, backtracking 3_176 times

Whereas the C version printed this:

  C solved 1000000 iterations in 88.454639 seconds with 3176 backtracks

Plainly I have some other optimizations to do.

### Optimizations

I'll run this test on each new optimization:

  `RUSTFLAGS="-C target-cpu=native" cargo run --release`

Here's what I got:

- Rust solved 1_000_000 iterations in 137.998281667s seconds, backtracking 3_176 times (unoptimized)
- Rust solved 1_000_000 iterations in 81.49986175s seconds, backtracking 3_176 times (use NonZeroU8 instead of u8)
