# Goldfisher web tool 🎣

This repository has the web tool implementation for [goldfisher](https://github.com/Cadiac/goldfisher) to [goldfish](https://mtg.fandom.com/wiki/Goldfishing) the fastest possible wins with the given decks in non-interactive games of Magic: The Gathering. This data can be used to gather statistics on average winning turns with different versions of the decks, helping to gauge effects of deck building.

## Installation

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.


To build the WASM based [yew](https://yew.rs/) UI, further wasm tooling is required

```
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cargo install wasm-bindgen-cli
```

## Development

For normal development, start the web server with

```
trunk serve
```

This should make the UI available at 0.0.0.0:8080 with hot reload on code changes.

To change the default port, use

```
trunk serve --port=9090
```
