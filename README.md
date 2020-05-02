This is your basic "Hello World" program for the [1Bitsy](https://1bitsy.org/)
setup to work with [Drone OS](https://www.drone-os.com/)

It sets up the system clock to run at 168 MHz.

This build uses a patched version of drone-os v0.12

## 1Bitsy notes
* STM32F415RGT6
* HSI - 16 MHz
* HSE - 25 MHz
* LSE - 32.768 kHz
* 512K Flash
* 128K RAM
* has a LED connected to PA8 (active low)

## Other modifications
* `Drone.toml` changed `gdb-client` to be `arm-none-eabi-gdb`, since I haven't found gdb-multiarch for MacOS.
* `Drone.toml` changed `serial-endpoint` and `gdb-endpoint` to match the USB names that my Black Magic Probe shows up as under MacOS.
* `Cargo.toml` points `drone-stm32-map` to locally patched `drone-stm32-map` repository.

To build - assuming other setup has been done as per the [Drone Book](https://book.drone-os.com/)
```
git clone https://github.com/dhylands/drone-weact-blinky.git weact-blinky
mkdir drone-os-patched
cd drone-os-patched
git clone --single-branch --branch dhylands-patches https://github.com/dhylands/drone-stm32-map.git
cd ../weact-blinky
just build
```

I have a [PR](https://github.com/drone-os/drone-stm32-map/pull/9) to do some fixups on the RCC_PLLCFGR register.
This has been merged into master, but I don't think it's released yet. Until it's released, the
[dhylands-patches](https://github.com/dhylands/drone-stm32-map/tree/dhylands-patches) includes this change
plus a bunch of edits to the `Cargo.toml` files to make them use crates from github rather than locally.
