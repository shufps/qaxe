# Qaxe

Qaxe is a  quad-BM1366 Miner based on the [PiAxe](https://github.com/shufps/piaxe) and [BitAxe](https://github.com/skot/bitaxe/tree/ultra-v1.3).

![image](https://github.com/shufps/qaxe/assets/3079832/4f741daf-940c-4ba4-a477-e8de91f4513c)

**rev1:** is tested and operating at about 1.7TH/s average speed.<br>
**rev2:** working fine with the expected speed of ~1.8TH/s avg after some minor modifications (330µF caps are wrongly placed, see rev3)<br>
**rev3:** Fixed Caps placement and added Boot-Switch. It should put the STM32 into DFU bootloader but not tested yet.<br>
**rev3.1:** Added pulldown on PB2 that is needed for booting the USB bootloader<br>
**rev3.2:** Board got 3mm larger for a perfect fit of low-profile coolers<br>


**note**: the `qaxe+` directory is for BM1368. It's **untested**.<br>
**note2**: If you have a board with `BOOT`-button (any rev3) please order the L072 STM32 (BOM has been updated) because usb bootloader is the easiest way to flash the STM.<br>

ASICs
=====

The QAxe uses 4 ASICs of type BM1366.

![image](https://github.com/shufps/qaxe/assets/3079832/da4b85cf-e7ba-4073-ae0d-08c4e82d4b8e)


Compilation (Bootloader or CMSIS-DAP)
======================================

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
#rustup target add thumbv6m-none-eabi

# or add rust target for STM32L1 variants
rustup target add thumbv7m-none-eabi

# build firmware for L072
cd firmware/fw-L072CB
./build.sh
```

Installation via USB Bootloader on board with `BOOT` button
===========================================================
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


Installation via CMSIS-DAP Programmer
=====================================

**note**: Using CMSIS-DAP and PicoProbe has been turned out to be quite a hassle for people who just want to flash the STM32 once, it's suggested to use the USB bootloader with the STM32 L072CB variant.

As programming/debug adapter the Picoprobe firmware running on a Raspi Pico works best: <br>
https://github.com/rp-rs/rp2040-project-template/blob/main/debug_probes.md / https://github.com/raspberrypi/picoprobe/releases/tag/picoprobe-cmsis-v1.0.3
<br>
<br>
There also is a little board with only 3 parts that gives a nice low-cost solution to flash the Qaxe:<br>
https://github.com/shufps/raspi-pico-dap

On `rev3` there should be the option to boot the stm32 (by pressing the `boot`-button on reset) into DFU-Bootloader mode what makes flashing via USB and without CMSIS-DAP programmer possible.

## Flashing

After the source was compiled it is flashed by:

```bash
# build firmware for L072
cd firmware/fw-L072CB
# run firmware (this also flashes it to the stm32)
./run.sh
```



Mining Client
=============

![image](https://github.com/shufps/qaxe/assets/3079832/5afb98b6-9153-454f-adc0-137706cad032)




Stratum Mining Client:<br>
https://github.com/shufps/piaxe-miner

Misc
====
If you like this project and want to support future work, feel free to donate to:
`bc1q29hp4fqtks2wzpmfwtpac64fnr8ujw2nvnra04`
