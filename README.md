# Joyscreen

Use an ATTiny85 with a light resistor to monitor building lighting to turn off a monitor when there is no occupancy.

This is Linux only for now, but the ATTiny85 joystick mode works on Windows, which is why that mode was chosen over serial (which currently causes bluescreens on Windows 10).

## Usage

```bash
LOG=debug ./joyscreen
```

## Uses
 - http://www.obdev.at/vusb/
 - Rust
 - Arduino IDE
 - xcb
 - gilrs
 - SimpleMovingAverage

## TODO

 - Publish light status over MQTT so all other monitors and systems can use this information as well
 - Fix moving average calculation to account for time
 - Switch away from joystick library, support different chips (pull requests accepted)
 - Clean up code
 - Possibility to calibrate light level triggering automatically or at least as a command line switch