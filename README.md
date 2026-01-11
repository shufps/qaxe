# NerdQ Hardware Repository

This repository contains multiple generations of small-scale Bitcoin miner hardware designs.

## NerdQAxe++

<img src="https://github.com/user-attachments/assets/e4ff81a3-103d-487d-b92c-4151bf7aecff" width="600px">

Current high-performance design based on 4× BM1370 ASICs.

<img src="https://github.com/user-attachments/assets/e59b47c3-221b-4ca1-80bd-fd26008c72ec" width="300px">

- ~4.8 TH/s at ~76 W (~15.8 J/TH)
- Previous stable version (that was used for most NQ++ boars): [rev5.1](https://github.com/shufps/qaxe/releases/tag/rev5.1)
- Latest:  [rev5.1.1](https://github.com/shufps/qaxe/releases/tag/rev5.1.1)*

*: contains some improvements but is untested yet.

## NerdQAxe+

<img src="https://github.com/user-attachments/assets/9e9a51d5-f22e-4789-9750-17623fee1ff3" width="600px">

Design based on 4x BM1368 ASICs.

- ~2.5TH/s at ~55 W (~22 J/TH)
- Latest revision 5.0

## Firmware for NQ+ and NQ++

The latest version can be found here:

https://github.com/shufps/ESP-Miner-NerdQAxePlus


## Legacy Designs

The following designs are **end of life** and no longer supported:

- QAxe+
- QAxe

They are provided for reference, documentation, and historical reasons only.

**No support, fixes, or updates will be provided for legacy designs.**

See `legacy/README.md` for full documentation of deprecated hardware.

---

## Compatible Replacement Parts

### For NerdQAxe++
- 25 MHz Oscillator
  Original: IQD LFSPXO076024
  Replacement: Taitien OXLTDLJANF-25.000000

### For NerdQAxe++ and NerdQAxe+
- 0.8 V LDO
  Original: Microchip MCP1824T-0802E-OT
  Replacement: TI TPS78408QDBVRQ1

