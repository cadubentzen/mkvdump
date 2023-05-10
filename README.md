# mkvdump

[![coverage](https://codecov.io/gh/cadubentzen/mkvdump/branch/main/graph/badge.svg?token=2Q2LOK4J95)](https://codecov.io/gh/cadubentzen/mkvdump)
[![test](https://github.com/cadubentzen/mkvdump/actions/workflows/test.yml/badge.svg)](https://github.com/cadubentzen/mkvdump/blob/main/.github/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/mkvdump.svg)](https://crates.io/crates/mkvdump)


A command-line tool for debugging Matroska/WebM files in common formats.

```yaml
- id: EBML
  header_size: 5
  size: 36
  children:
  - id: EBMLVersion
    header_size: 3
    size: 4
    value: 1
  - id: EBMLReadVersion
    header_size: 3
    size: 4
    value: 1
  - id: EBMLMaxIDLength
    header_size: 3
    size: 4
    value: 4
  - id: EBMLMaxSizeLength
    header_size: 3
    size: 4
    value: 8
  - id: DocType
    header_size: 3
    size: 7
    value: webm
  - id: DocTypeVersion
    header_size: 3
    size: 4
    value: 2
  - id: DocTypeReadVersion
    header_size: 3
    size: 4
    value: 2
- id: Segment
  header_size: 12
  size: Unknown
  children:
  - id: Void
    header_size: 9
    size: 229
    value: null
  - id: Info
    header_size: 5
    size: 44
    children:
    - id: TimestampScale
      header_size: 4
      size: 7
      value: 1000000
    - id: MuxingApp
      header_size: 3
      size: 16
      value: Lavf58.29.100
    - id: WritingApp
      header_size: 3
      size: 16
      value: Lavf58.29.100
  - id: Tracks
    header_size: 5
    size: 101
    children:
    - id: TrackEntry
      header_size: 9
      size: 96
      children:
      - id: TrackNumber
        header_size: 2
        size: 3
        value: 1
      - id: TrackUID
        header_size: 3
        size: 4
        value: 1
      - id: FlagLacing
        header_size: 2
        size: 3
        value: 0
      - id: Language
        header_size: 4
        size: 7
        value: und
      - id: CodecID
        header_size: 2
        size: 7
        value: V_AV1
      - id: TrackType
        header_size: 2
        size: 3
        value: video
      - id: DefaultDuration
        header_size: 4
        size: 8
        value: 41708333
      - id: Video
        header_size: 9
        size: 32
        children:
        - id: PixelWidth
          header_size: 2
          size: 4
          value: 1280
        - id: PixelHeight
          header_size: 2
          size: 4
          value: 720
        - id: Colour
          header_size: 3
          size: 15
          children:
          - id: Range
            header_size: 3
            size: 4
            value: broadcast range
          - id: ChromaSitingHorz
            header_size: 3
            size: 4
            value: left collocated
          - id: ChromaSitingVert
            header_size: 3
            size: 4
            value: half
      - id: CodecPrivate
        header_size: 3
        size: 20
        value: '[81 05 0c 00 0a 0b 00 00 00 2d 4c ff b3 df ff 98 04]'
  - id: Tags
    header_size: 5
    size: 61
    children:
    - id: Tag
      header_size: 10
      size: 56
      children:
      - id: Targets
        header_size: 10
        size: 10
        children: []
      - id: SimpleTag
        header_size: 10
        size: 36
        children:
        - id: TagName
          header_size: 3
          size: 10
          value: ENCODER
        - id: TagString
          header_size: 3
          size: 16
          value: Lavf58.29.100
  - id: Cluster
    header_size: 6
    size: 2679
    children:
    - id: Timestamp
      header_size: 2
      size: 3
      value: 0
    - id: SimpleBlock
      header_size: 2
      size: 45
      value:
        track_number: 1
        timestamp: 0
        keyframe: true
    - id: SimpleBlock
      header_size: 2
      size: 59
      value:
        track_number: 1
        timestamp: 42
    - id: SimpleBlock
      header_size: 2
      size: 32
      value:
        track_number: 1
        timestamp: 83
    # ...
```

## Getting mkvdump

### Debian package

Ubuntu users (>= 20.04) can install mkvdump via the DEB package available in the [releases page](https://github.com/cadubentzen/mkvdump/releases).

### Cargo

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed, you can install mkvdump with

```bash
$ cargo binstall mkvdump
```

Else, you can install by building it from source with:

```bash
$ cargo install mkvdump
```

### Docker

To pull latest mkvdump from [Docker Hub](https://hub.docker.com/r/cadubentzen/mkvdump):

```bash
$ docker pull cadubentzen/mkvdump
```

A [GitHub package](https://github.com/cadubentzen/mkvdump/pkgs/container/mkvdump) is also available via

```bash
$ docker pull ghcr.io/cadubentzen/mkvdump
```

Images are multi-arch with support for `linux/amd64`, `linux/386`, `linux/arm64`, `linux/arm/v7` and `linux/arm/v6`.

#### Running the container

Asssuming a Mastroska file in the host located at `/host-path/sample.mkv`. You can run mkvdump on it with the following command, by mounting a volume:
```bash
$ docker run -v /host-path:/media cadubentzen/mkvdump /media/sample.mkv
```

### Prebuilt binaries

Download prebuilt binaries from the [release page](https://github.com/cadubentzen/mkvdump/releases). There are binaries for the following targets:
- Linux
  - statically linked with musl: `x86_64`, `x86`, `aarch64`, `armv7l` and `armv6l`
  - with GNU libc: `x86_64` and `x86` (built on Ubuntu 20.04)
- macOS
  - `x86_64` and `aarch64` (>= macOS 11 Big Sur)
- Windows
  - `x86_64` and `x86` with MSVC and MinGW

## License

&copy; 2022 Carlos Bentzen <cadubentzen@gmail.com>.

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.

