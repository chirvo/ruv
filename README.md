# ruv

This provides a Rust-based utility for undervolting Ryzen 5800x3d processors using the Precision Boost Overdrive (PBO) feature. It interacts with the [Ryzen SMU driver](https://github.com/leogx9r/ryzen_smu) to read and modify curve offsets for individual cores.

## Features
- List current curve offsets for specified cores.
- Set negative curve offsets for undervolting.
- Reset all offsets to zero.

## Requirements
- rust and cargo.
- Root privileges are required to run the script.
- The Ryzen SMU driver ([leogx9r/ryzen_smu](https://github.com/leogx9r/ryzen_smu)) must be available and loaded.

## Installation

To install the `ruv` utility, follow these steps:

1. Build the project:
   ```bash
   make
   ```

2. Install the binary to `/usr/local/bin`:
   ```bash
   sudo make install
   ```

3. To clean up the build artifacts, you can run:
   ```bash
   make clean
   ```

After installation, you can run the `ruv` utility from anywhere using the `ruv` command.

## Usage
Run it with the following options:

-  `-l`, `--list`                   List curve offsets
-  `-o`, `--offset=<offset>`        Set curve offset
-  `-r`, `--reset`                  Reset offsets to 0
-  `-h`, `--help`                   Print help

Example:
```bash
sudo ruv --offset=-30
```
This sets a curve offset of `-30` for all 8 cores (core count is hardwired to 8).

## Acknowledgment
This program is based on the Python version of the Ryzen undervolting utility available at [svenlange2/Ryzen-5800x3d-linux-undervolting](https://github.com/svenlange2/Ryzen-5800x3d-linux-undervolting). Special thanks to the original author for their work.

## Migration Note
The migration from Python to Rust was entirely made using `Google Gemini 2.5 Pro`, utilizing `Visual Studio Code` and the `Roo Code` plugin.