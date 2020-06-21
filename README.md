# pulseaudio-headphones-connect

This tool detects when a bluetooth device is connected, and configure PulseAudio to use it.

It is roughly equivalent to the following script:

```bash
#!/bin/bash

MAC=01_23_45_67_89_AB

FILTER=$(paste -sd, <<FILTER
type='signal'
sender='org.bluez'
interface='org.freedesktop.DBus.Properties'
path='/org/bluez/hci0/dev_$MAC'
member='PropertiesChanged'
arg0='org.bluez.Device1'
FILTER
)

dbus-monitor --system "$FILTER"                        \
  | sed -u '/^signal/ { :s N; /]/be; bs; :e s/\n//g }' \
  | grep --line-buffered 'Connected.*true'             \
  | while read
    do
      dbus-send                  \
        --system                 \
        --print-reply            \
        --dest=org.bluez         \
        --type=method_call       \
        /org/bluez/hci0/dev_$MAC \
        org.bluez.Device1.Connect

      pactl set-card-profile "bluez_card.$MAC" "a2dp_sink"
    done
```

## Usage

```bash
$ DEVICE_MAC=01:23:45:67:89:AB

$ pulseaudio-headphones-connect $DEVICE_MAC
```
