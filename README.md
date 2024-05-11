# STM32G431 experiments

A package with examples for stm32g431 chip on a [creapunk CLN17 board](https://creapunk.com/)

For now I own only a early v1.0 board, so pins will be different on your board, make sure you know that you are doing.

# Prerequisites

## Install rust 

```
curl - proto '=https' - tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Install rust target 

Install the compilation target for your MCU.

- Cortex-M4F and Cortex-M7F (eg F, L4, G4, H7):
```
rustup target add thumbv7em-none-eabihf
```

- Cortex-M0 and Cortex-M0+ (eg G0):
```
rustup target add thumbv6m-none-eabi
```

- Cortex-M33F and Cortex-M35F (eg L5, U5, H5):

```
rustup target add thumbv8m.main-none-eabihf
```

## Install tools

- probe-rt: https://probe.rs/

```
curl - proto '=https' - tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-installer.sh | sh
```

- Flip-link
```
cargo install flip-link
```

- dfu-util
  - [instruction for diference OS](https://github.com/redbear/Duo/blob/master/docs/dfu-util_installation_guide.md)
  - ubuntu
    - ```sudo apt-get install dfu-util```
  - ARC
    - ```sudo pacman -S dfu-util```
- screen
  - instal via apt-get\pacman - used for serial port monitoring

# Using ST Link

positives

* debug print feedback

negatives

* you need a ST Link

## examples/blink

check options bytes, 

- nSWBOOT0 = 0
- nBOOT0 = 1

so chip run a code after its flashed

```
cargo run --release --package blink
```

after this the LED should blink and debug print a message

```
<lvl> Hello, world!
└─ blink::__cortex_m_rt_main @ examples/blink/src/main.rs:62
```

# Using DFU

positives

* you need only USB

negatives

* no debug print feedback

## examples/blink

compile a code to binary file
```
cargo objcopy --release --package blink -- -O binary blink.bin
```

make sure a device is in DFU mode

```
# dfu-util --list

dfu-util 0.11

Copyright 2005-2009 Weston Schmidt, Harald Welte and OpenMoko Inc.
Copyright 2010-2021 Tormod Volden and Stefan Schmidt
This program is Free Software and has ABSOLUTELY NO WARRANTY
Please report bugs to http://sourceforge.net/p/dfu-util/tickets/

dfu-util: Cannot open DFU device 04f2:b7b6 found on devnum 5 (LIBUSB_ERROR_ACCESS)
Found DFU: [0483:df11] ver=0200, devnum=7, cfg=1, intf=0, path="1-3", alt=2, name="@OTP Memory   /0x1FFF7000/01*01Ke", serial="208A31574253"
Found DFU: [0483:df11] ver=0200, devnum=7, cfg=1, intf=0, path="1-3", alt=1, name="@Option Bytes   /0x1FFF7800/01*40Be", serial="208A31574253"
Found DFU: [0483:df11] ver=0200, devnum=7, cfg=1, intf=0, path="1-3", alt=0, name="@Internal Flash   /0x08000000/64*02Kg", serial="208A31574253"
```

for download and run bin file
```
sudo dfu-util -a 0 -i 0 -s 0x08000000:leave -D blink.bin -S 208A31574253
```

after this the LED should blink

### dfu-util permissions problem

```
todo
```