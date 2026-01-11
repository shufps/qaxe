# Legacy Hardware Designs

⚠️ **Status: End of Support**

This directory contains deprecated hardware designs that are no longer maintained.

- No bug fixes
- No hardware revisions
- No firmware updates
- No user support

The designs are provided **as-is** for reference and historical documentation.

---

## QAxe+

<img src="https://github.com/shufps/qaxe/assets/3079832/79d25550-ae5b-4eae-92bb-4ff231449e13" width="600px">

Quad-BM1368 miner.

- ~2.4 TH/s at ~55 W (230 V measured)
- Final revisions: rev4 / rev4.1
- Known quirks with ASIC reset behavior

---

## QAxe

![image](https://github.com/shufps/qaxe/assets/3079832/4f741daf-940c-4ba4-a477-e8de91f4513c)

Quad-BM1366 miner.

- ~1.7–1.8 TH/s depending on revision
- Multiple early hardware revisions (rev1–rev3.2)

Notes:
- Boards with BOOT button require STM32L072CB
- USB DFU bootloader recommended for flashing

---

Firmware & Tooling
=====================

Legacy firmware, build instructions, and tooling are preserved for completeness.
They may require outdated dependencies and are not guaranteed to work on modern systems.



## Compilation (Bootloader or CMSIS-DAP) (QAxe, QAxe+)


### Dockered compilation and flashing
There is a wonderfull Docker based single-script compilation and flash tool for the QAxe Firmware:

https://github.com/AnimaI/QAxe-Docker-Bootloader

### Manual compilation and installation


```bash
# install curl
sudo apt install curl

# install rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# add to ~/.bash.rc (afterwards, opening a new terminal is needed)
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc

# clone repository
git clone https://github.com/shufps/qaxe

# clone submodules
cd qaxe
git submodule init
git submodule update

# add rust target for STM32L0 variants
rustup target add thumbv6m-none-eabi

# or add rust target for STM32L1 variants
#rustup target add thumbv7m-none-eabi

# build firmware for L072
cd firmware/fw-L072CB
./build.sh
```

### Installation via USB Bootloader on board with `BOOT` button (QAxe, QAxe+)
The STM32L072CB variant has an integrated DFU Bootloader that starts when pressing the `BOOT` button during reset.

Afterwards the firmware can be flashed via `dfu-utils`:

```bash
# install cargo-binutils and llvm tools
cargo install cargo-binutils
rustup component add llvm-tools-preview

# create the firmware.bin
DEFMT_LOG=info cargo objcopy --release --bin qaxe -- -O binary qaxe.bin

# install dfu-utils
sudo apt-get install dfu-util

now start the stm32 in DFU mode by pressing `boot` (only works with the STM32L072CB variant)

# after booting, list the devices
dfu-util --list

# flash the binary
dfu-util -a 0 -s 0x08000000:leave -D qaxe.bin
```


Mining Client (QAxe, QAxe+)
=============

![image](https://github.com/user-attachments/assets/95591dea-1ee0-4877-9318-95d7a2488da4)


Stratum Mining Client:<br>
https://github.com/shufps/piaxe-miner


---

## Disclaimer

These designs are **not recommended for new builds**.

If you are looking for a supported and actively developed project, refer to the main repository README.