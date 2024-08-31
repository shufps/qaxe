# NerdQAxe+

NerdQaxe+ is a Qaxe+ with [Nerdminer](https://github.com/BitMaker-hub/NerdMiner_v2) / [Nerdaxe](https://github.com/BitMaker-hub/NerdAxeUltra) Display and is running the [BitAxe](https://github.com/skot/bitaxe) Firmware as its core.

It runs standalone without Raspberry Pi and uses 4 ASICs of type BM1368.

<img src="https://github.com/user-attachments/assets/9e9a51d5-f22e-4789-9750-17623fee1ff3" width="600px">

Highlights:

- uses the NerdAxe / NerdMiner display
- better Buck converter that runs a lot cooler (about 10°C) with oltage / current / power monitoring
- standalone, no Raspberry Pi or other PC needed
- AxeOS with improvements and enhancements
  - Influx DB support
  - Better charting (10m, 1h, 1d), data doesn't get lost on Web UI reloads 
  - ASIC clock and voltage adjustable without reboot
  - Stratum client stability improvements (TCP timeouts)

The NerdQAxe+ runs with a variant of the AxeOS firmware: https://github.com/shufps/ESP-Miner-NerdQAxePlus

**rev5.0** Good to go, no bug found 🥳🚀

# QAxe+

QAxe+ is a quad-BM1368 Variant. Mining speed is average 2.4TH/s at 55W (measured at 230V)<br>

<img src="https://github.com/shufps/qaxe/assets/3079832/79d25550-ae5b-4eae-92bb-4ff231449e13" width="600px">
<br>


**rev4:** QAxe+ with BM1368, working but ASIC reset behaves weird sometimes. Measured performance is 2.4TH/s average with 55W (measured on 230V)<br>
**rev4.1:** only change is 3 pull-down resistors on NRSTI pins<br>
<br>


# Qaxe

Qaxe is a  quad-BM1366 Miner based on the [PiAxe](https://github.com/shufps/piaxe) and [BitAxe](https://github.com/skot/bitaxe/tree/ultra-v1.3).

![image](https://github.com/shufps/qaxe/assets/3079832/4f741daf-940c-4ba4-a477-e8de91f4513c)

**rev1:** is tested and operating at about 1.7TH/s average speed.<br>
**rev2:** working fine with the expected speed of ~1.8TH/s avg after some minor modifications (330µF caps are wrongly placed, see rev3)<br>
**rev3:** Fixed Caps placement and added Boot-Switch. It should put the STM32 into DFU bootloader but not tested yet.<br>
**rev3.1:** Added pulldown on PB2 that is needed for booting the USB bootloader<br>
**rev3.2:** Board got 3mm larger for a perfect fit of low-profile coolers<br>
<br>
**note**: If you have a board with `BOOT`-button (any rev3) please order the L072 STM32 (BOM has been updated) because usb bootloader is the easiest way to flash the STM.<br>



ASICs
=====

The QAxe uses 4 ASICs of type BM1366.

![image](https://github.com/shufps/qaxe/assets/3079832/da4b85cf-e7ba-4073-ae0d-08c4e82d4b8e)

The QAxe+ and NerdQaxe+ uses 4 ASICs of type BM1368.



Compilation (Bootloader or CMSIS-DAP) (QAxe, QAxe+)
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

Installation via USB Bootloader on board with `BOOT` button (QAxe, QAxe+)
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


Mining Client (QAxe, QAxe+)
=============

![image](https://github.com/shufps/qaxe/assets/3079832/5afb98b6-9153-454f-adc0-137706cad032)




Stratum Mining Client:<br>
https://github.com/shufps/piaxe-miner

Misc
====
If you like this project and want to support future work, feel free to donate to:
`bc1q7n70rumyv6lvu8avpml0c3uggvssfu52egum3q`
