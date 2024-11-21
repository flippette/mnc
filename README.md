# Moist-Not-Critical: A plant status monitoring device (Aalto DTEP Team 10 Project)

## Components

- Raspberry Pi Pico
- Adafruit BH1750 Light Sensor
- Seeed Studio Grove Moisture Sensor
- GMT020-02 320x240 SPI TFT LCD

## Wiring

Pin names are taken from product pages for the sensors and
<https://pico.pinout.xyz/> for the Pico.

- BH1750 -> Pico:
  - VIN -> VSYS
  - 3Vo -> nil
  - GND -> GND
  - SCL -> GP17
  - SDA -> GP16
- Grove Moisture Sensor -> Pico:
  - VCC -> 3V3 Out
  - GND -> GND
  - SIG -> GP26
- GMT020-02 -> Pico:
  - GND -> GND
  - VCC -> VSYS
  - SCL -> GP14
  - SDA -> GP15
  - RST -> GP10
  - DC -> GP11
  - CS -> GP13
