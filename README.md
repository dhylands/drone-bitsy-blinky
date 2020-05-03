This is your basic "Hello World" program for the [1Bitsy](https://1bitsy.org/)
setup to work with [Drone OS](https://www.drone-os.com/)

It sets up the system clock to run at 168 MHz.

This example uses drone-os v0.12.1

## 1Bitsy notes
* STM32F415RGT6
* HSI - 16 MHz
* HSE - 25 MHz
* LSE - 32.768 kHz
* 512K Flash
* 128K RAM
* has a LED connected to PA8 (active low)

## Modifications
* `Drone.toml` changed `gdb-client` to be `arm-none-eabi-gdb`, since I haven't found gdb-multiarch for MacOS.
* `Drone.toml` changed `serial-endpoint` and `gdb-endpoint` to match the USB names that my Black Magic Probe shows up as under MacOS.

To build - assuming other setup has been done as per the [Drone Book](https://book.drone-os.com/)
```
git clone https://github.com/dhylands/drone-bitsy-blinky.git bitsy-blinky
cd bitsy-blinky
just build
```
