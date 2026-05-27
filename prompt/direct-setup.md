# Direct-to-Nest network topology

## High-level description

You are running on an Ubuntu system called the primary host.

The primary host is connected to the Internet via a WiFi connection.

The primary host is connected to a Google Nest Wifi Pro with WiFi 6E directly
via an Ethernet cable. The cable is plugged into a port labeled as the main
connection.

## Machine-readable file schema

`data/network.json` in the project root contains an object with the following
keys:

* `primary` - object with keys:
    * `hostname` - name advertised via mDNS (without the `.local` suffix)
    * `eth-main` - objects with keys
        * `interface` - interface name, such as `eth0`
        * `ip` - local IPv4 address

* `wifi` - object with keys:
    * `ap-name` - the WiFi's SSID
    * `password` - the WiFi's WPA2 password
