# ruv

This provides a Rust-based utility for undervolting Ryzen 5800x3d processors using the Precision Boost Overdrive (PBO) feature. It interacts with the [Ryzen SMU driver](https://github.com/leogx9r/ryzen_smu) to read and modify curve offsets for individual cores.

## Features
- List current curve offsets for specified cores.
- Set negative curve offsets for undervolting.
- Reset all offsets to zero.
- Default action (no arguments): List current curve offsets.

## Requirements
- rust and cargo.
- Root privileges are required to run the script.
- The Ryzen SMU driver ([leogx9r/ryzen_smu](https://gitlab.com/leogx9r/ryzen_smu)) must be available and loaded.
- A systemd-based Linux distribution is required for the service installation.

## Installation

To install the `ruv` utility, follow these steps:

1. Build the project:
   ```bash
   make
   ```

2. Install the binary, copy the systemd service file, and enable the service:
   ```bash
   sudo make install
   ```
   This command performs the following actions:
   - Compiles the release binary (`target/release/ruv`).
   - Installs the binary to `/usr/local/bin/ruv`.
   - Copies the `ruv.service` file to `/etc/systemd/system/`.
   - Reloads the systemd daemon (`systemctl daemon-reload`).
   - Enables the `ruv.service` to start on boot and resume (`systemctl enable ruv.service`).

3. To clean up the build artifacts, you can run:
   ```bash
   make clean
   ```

After installation, the `ruv` utility can be run manually. The installed `ruv.service` will automatically apply the offset configured in `/etc/default/ruv_offset` (or -20 if the file is missing/invalid) on system startup and resume from suspend by explicitly calling `ruv --offset=<value>`.

## Uninstallation

To remove the utility and the systemd service:
```bash
sudo make uninstall
```
This command performs the following actions:
- Disables the `ruv.service` (`systemctl disable ruv.service`).
- Removes the service file (`/etc/systemd/system/ruv.service`).
- Removes the binary (`/usr/local/bin/ruv`).
- Reloads the systemd daemon (`systemctl daemon-reload`).

## Usage
Run it with the following options:

-  `-l`, `--list`                   List curve offsets
-  `-o`, `--offset=<offset>`        Set curve offset
-  `-r`, `--reset`                  Reset offsets to 0
-  `-h`, `--help`                   Print help

**Default Action & Service Configuration:**

If `ruv` is run without any arguments (`--list`, `--offset`, or `--reset`), its default action is now to **list** the current curve offsets for all cores.

The systemd service (`ruv.service`), however, still uses the `/etc/default/ruv_offset` file to apply a persistent undervolt on startup and resume.

- Create `/etc/default/ruv_offset` and place a single negative integer inside (e.g., `-25`) to specify the offset the service should apply.
- If this file does not exist, cannot be read, or contains an invalid value (not a negative integer), the service will default to applying an offset of `-20`.
- The service achieves this by reading the file (or using the default) and then explicitly running `/usr/local/bin/ruv --offset=<value>`.

Example:
```bash
sudo ruv --offset=-30
```
This explicitly sets a curve offset of `-30` for all 8 cores.

```bash
# Run ruv without arguments
sudo ruv
```
This will **list** the current offsets for all 8 cores, as listing is the default action.

```bash
# To have the service apply -25 on boot/resume:
echo "-25" | sudo tee /etc/default/ruv_offset
# The service (installed via 'sudo make install') will read this file
# and run 'ruv --offset=-25' automatically.
```

## Acknowledgment
This program is based on the Python version of the Ryzen undervolting utility available at [svenlange2/Ryzen-5800x3d-linux-undervolting](https://github.com/svenlange2/Ryzen-5800x3d-linux-undervolting). Special thanks to the original author for their work.

## Migration Note
The migration from Python to Rust was entirely made using `Google Gemini 2.5 Pro`, utilizing `Visual Studio Code` and the `Roo Code` plugin.