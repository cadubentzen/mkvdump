# mkvdump

[![codecov](https://codecov.io/gh/cadubentzen/mkvdump/branch/main/graph/badge.svg?token=2Q2LOK4J95)](https://codecov.io/gh/cadubentzen/mkvdump)
![build](https://github.com/cadubentzen/mkvdump/actions/workflows/rust.yml/badge.svg)
![cross](https://github.com/cadubentzen/mkvdump/actions/workflows/cross.yml/badge.svg)

A command-line tool for debugging Matroska/WebM files in common formats.

```
$ mkvdump --help
mkvdump 0.2.0
Carlos Bentzen <cadubentzen@gmail.com>
MKV and WebM parser CLI tool

USAGE:
    mkvdump [OPTIONS] <FILENAME>

ARGS:
    <FILENAME>    Name of the MKV/WebM file to be parsed

OPTIONS:
    -f, --format <FORMAT>           Output format [default: yaml] [possible values: json, yaml]
    -h, --help                      Print help information
    -p, --show-element-positions    Add element positions in the output
    -V, --version                   Print version information
```


Sample output:
```yaml
# mkvdump sample.mkv
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

# Getting mkvdump

## Cargo

If you have `cargo` installed, you can install mkvdump from [crates.io](https://crates.io) with:
```bash
$ cargo install mkvdump
```

## Docker

To pull latest mkvdump from Dockerhub:
```bash
$ docker pull cadubentzen/mkvdump
```

Images are available for `linux/amd64` and `linux/arm64`. Need a new architecture? PRs are welcome!

### Running the container

Asssuming a Mastroska file in the host located in `/host-path/sample.mkv`. You could run mkvdump on it with the following command, by mounting a volume:
```bash
$ docker run -v /host-path:/media cadubentzen/mkvdump /media/sample.mkv
```

## Prebuilt binaries

Download prebuilt binaries for **Linux** on `x86_64` and `aarch64` from the [release page](https://github.com/cadubentzen/mkvdump/releases).

Download it somewhere accessible in your PATH and make it runnable:
```bash
$ sudo curl -L https://github.com/cadubentzen/mkvdump/releases/download/v0.2.0/mkvdump-linux-x86_64 -o /usr/local/bin/mkvdump
$ sudo chmod +x /usr/local/bin/mkvdump
```

# License

&copy; 2022 Carlos Bentzen <cadubentzen@gmail.com>.

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.

