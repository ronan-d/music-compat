# music-compat

## Description

`music-compat` is a program that creates a copy of a music library with maximum
compatibility. It uses widespread compression and container formats, and
generate file names compatible with even the most limited filesystems.

`music-compat` uses FFmpeg under the hood, and therefore requires it to be
installed.

## Usage

Assuming that your music library is located at `~/Music`, create a portable
version of it like so:

```sh
$ music-compat ~/Music ~/portable-music-library
```
