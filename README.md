# music-compat

## Description

`music-compat` is a program that creates a copy of a music library with maximum
compatibility. It uses widespread compression and container formats, and
generate file names compatible with even the most limited filesystems.

`music-compat` uses FFmpeg under the hood, and therefore requires it to be
installed.

## Installation

Building `music-compat` requires a Rust toolchain. Install rust at
https://www.rust-lang.org/tools/install.

Run the following command to install `music-compat`

```sh
$ cargo install --path <path-to-repo>
```

## Usage

Assuming that your music library is located at `~/Music`, create a portable
version of it like so:

```sh
$ music-compat ~/Music ~/portable-music-library
```
