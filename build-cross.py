#!/usr/bin/env python3

import subprocess
import shutil

TARGETS = [
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
]


def run(command: str):
    print(command)
    subprocess.run(command.split(" "), check=True)


def main():
    run("cargo install cross --git https://github.com/cross-rs/cross")

    for target in TARGETS:
        run(f"cross test --release --target {target}")

    for target in TARGETS:
        run(f"cross build --release --target {target}")

    for target in TARGETS:
        arch = target.split("-")[0]
        shutil.copyfile(f"target/{target}/release/mkvdump", f"mkvdump-{arch}")


if __name__ == "__main__":
    main()
