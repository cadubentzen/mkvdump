#!/usr/bin/env python3

import subprocess
import shutil
import sys


def run(command: str):
    print(command)
    subprocess.run(command.split(" "), check=True)


def main():
    if len(sys.argv) != 2:
        print("usage: ./build-cross.py <target>")
        sys.exit(1)

    target = sys.argv[1]
    arch = target.split("-")[0]

    run(f"cross test --release --target {target}")
    run(f"cross build --release --target {target}")

    shutil.copyfile(f"target/{target}/release/mkvdump", f"mkvdump-{arch}")


if __name__ == "__main__":
    main()
