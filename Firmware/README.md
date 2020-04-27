
# DWM1000

### Libraries
- https://github.com/F-Army/arduino-dw1000-ng
- https://github.com/thotro/arduino-dw1000

### Install instructions:
- Extract to arduino/libraries folder

# Mini Core

### Core Code:
https://github.com/MCUdude/MiniCore

### Guide:
- https://www.circuito.io/blog/atmega328p-bootloader/
- https://andreasrohner.at/posts/Electronics/How-to-make-the-Watchdog-Timer-work-on-an-Arduino-Pro-Mini-by-replacing-the-bootloader/

# ESP32

### ESP32 Core Code:
https://github.com/espressif/arduino-esp32

### Installing with Boards Manager
- Install the current upstream Arduino IDE at the 1.8 level or later. The current version is at the [Arduino website](http://www.arduino.cc/en/main/software).
- Start Arduino and open Preferences window.
- Enter ```https://dl.espressif.com/dl/package_esp32_index.json``` into *Additional Board Manager URLs* field. You can add multiple URLs, separating them with commas.
- Open Boards Manager from Tools > Board menu and install *esp32* platform (and don't forget to select your ESP32 board from Tools > Board menu after installation).

Stable release link: `https://dl.espressif.com/dl/package_esp32_index.json`
Development release link: `https://dl.espressif.com/dl/package_esp32_dev_index.json`


# ESP8266

### ESP8266 Core Code:
https://github.com/esp8266/Arduino

### Installing with Boards Manager
- Install the current upstream Arduino IDE at the 1.8.7 level or later. The current version is on the [Arduino website](https://www.arduino.cc/en/main/software).
- Start Arduino and open the Preferences window.
- Enter ```https://arduino.esp8266.com/stable/package_esp8266com_index.json``` into the *Additional Board Manager URLs* field. You can add multiple URLs, separating them with commas.
- Open Boards Manager from Tools > Board menu and install *esp8266* platform (and don't forget to select your ESP8266 board from Tools > Board menu after installation).
- install the pyserial module (needed by the ESP module, though they dont seem to mention it in the install instructions..)
  $ pip install pyserial

# Anchor Beacons

### Examples:
- https://github.com/nkolban/ESP32_BLE_Arduino
- https://github.com/jarkko-hautakorpi/iBeacon-indoor-positioning-demo
- https://www.instructables.com/id/JARVAS-Indoor-Positioning-System/

# ID TAG

### Examples:
- https://lastminuteengineers.com/esp32-sleep-modes-power-consumption/
- https://randomnerdtutorials.com/esp32-bluetooth-low-energy-ble-arduino-ide/
- https://randomnerdtutorials.com/esp32-deep-sleep-arduino-ide-wake-up-sources/
