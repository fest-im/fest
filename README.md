# Fest ![Logo](/res/icons/hicolor/24x24/apps/fest.svg)

[![TravisCI](https://api.travis-ci.org/fest-im/fest.svg?branch=master)](https://travis-ci.org/fest-im/fest) [![GitHub stars][badge]][repo]

[badge]: https://img.shields.io/github/stars/fest-im/fest.svg?style=social&label=Stars
[repo]: https://github.com/fest-im/fest

A GTK+ 3 [Matrix](https://matrix.org) chat client (work in progress). Right now,
Fest is not functional!

Available under the terms of the GNU GPL version 3 or later. See `LICENSE` for
details.

## Requirements

* [GTK+](https://www.gtk.org/download/index.php) ≥ 3.16
* [Rust](https://www.rust-lang.org/en-US/install.html) Nightly

The easiest way to install Rust nightly is with [rustup](https://www.rustup.rs):

```
rustup install nightly
```

If you are using Debian or Ubuntu you will need to install `libgtk-3-dev` in order
to compile Fest. You can install it using

```
sudo apt install libgtk-3-dev
```

## How to Run

First, clone the repository:

```
git clone https://github.com/fest-im/fest
```

Then run with Cargo:

```
cd fest
cargo +nightly run
```

## Contributing

Chat with us at [#fest-im:matrix.org][].

Issues or pull requests can be filed on the [GitHub tracker][issues].

[#fest-im:matrix.org]: https://matrix.to/#/#fest-im:matrix.org
[issues]: https://github.com/fest-im/fest/issues

