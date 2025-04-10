# ruv

This provides a Rust-based utility for undervolting Ryzen processors using the Precision Boost Overdrive (PBO) feature. It interacts with the Ryzen SMU driver to read and modify curve offsets for individual cores.

## Features
- List current curve offsets for specified cores.
- Set negative curve offsets for undervolting.
- Reset all offsets to zero.

## Requirements
- Root privileges are required to run the script.
- The Ryzen SMU driver must be loaded at `/sys/kernel/ryzen_smu_drv/` [leogx9r/ryzen_smu](https://github.com/leogx9r/ryzen_smu).

## Usage
Run it with the following options:

-  `-l`, `--list`                   List curve offsets
-  `-o`, `--offset=<offset>`        Set curve offset
-  `-c`, `--corecount <corecount>`  Set offset to cores [0..corecount] (default: 1)
-  `-r`, `--reset`                  Reset offsets to 0
-  `-h`, `--help`                   Print help

Example:
```bash
sudo ./ruv -o=-30 -c 8
```
This sets a curve offset of `-30` for the first 8 cores.

## Acknowledgment
This program is based on the Python version of the Ryzen undervolting utility available at [svenlange2/Ryzen-5800x3d-linux-undervolting](https://github.com/svenlange2/Ryzen-5800x3d-linux-undervolting). Special thanks to the original author for their work.

## Migration Note
The migration from Python to Rust was entirely made using `Google Gemini 2.5 Pro`, utilizing `Visual Studio Code` and the `Roo Code` plugin.