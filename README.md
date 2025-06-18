# LabVIEW-MIDI-XPLAT
Cross-platform MIDI driver library built in Rust for LabVIEW integration. Provides a C-compatible API wrapper around the midir library, enabling LabVIEW applications to handle MIDI input/output across Windows, macOS, and Linux platforms through compiled shared libraries (.dll/.so/.dylib).

A high-performance, cross-platform MIDI driver library written in Rust, designed specifically for integration with both 32-bit and 64-bit LabVIEW applications.

## Features

- **Cross-platform support**: Windows, macOS, and Linux
- **Multi-architecture**: 32-bit (i686) and 64-bit (x86_64) builds
- **Real-time MIDI I/O**: Low-latency MIDI input and output handling
- **LabVIEW-friendly API**: Simple C-compatible function interface
- **Device enumeration**: List and select available MIDI devices
- **Thread-safe**: Safe for use in multi-threaded LabVIEW applications
- **Zero-copy design**: Efficient memory management for real-time performance

## Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [midir](https://github.com/Boddlnagg/midir) - Cross-platform MIDI I/O library
- C FFI (Foreign Function Interface) for LabVIEW compatibility

## Pre-built Binaries

| Platform | 32-bit | 64-bit |
|----------|--------|--------|
| Windows  | `midi_driver_win32.dll` | `midi_driver_win64.dll` |
| macOS    | `libmidi_driver_mac32.dylib` | `libmidi_driver_mac64.dylib` |
| Linux    | `libmidi_driver_linux32.so` | `libmidi_driver_linux64.so` |

## Usage

1. Download the appropriate library for your LabVIEW architecture (32-bit or 64-bit)
2. Place the library file in your LabVIEW project directory
3. Use the Call Library Function Node to access the exported functions

## Supported Platforms

- Windows 7+ (x86/x64)
- macOS 10.12+ (Intel x86/x64, Apple Silicon via Rosetta)
- Linux (x86/x64)

## LabVIEW Compatibility

- LabVIEW 2018 and newer (32-bit and 64-bit)
- LabVIEW RT (where supported by platform)
