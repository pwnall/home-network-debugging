# Lab network topology

## High-level description

You are running on an Ubuntu system called the primary host.

The primary host is connected to the Internet via a WiFi connection. The
connection is provided by a Google Nest Wifi Pro unit, which is the internal
network's main router.

The main router is connected to a cable modem, which provides Internet
connectivity for entire internal network.

The primary host is connected to a managed Ethernet switch via an Ethernet
connection, called the main connection.

The managed Ethernet switch also has a Google Nest Wifi Pro unit connected to it
via Ethernet. This is not the Google Nest Wifi Pro unit connected to the router.

## Machine-readable file schema

`data/network.json` in the project root contains an object with the following
keys:

* `primary` - object with keys:
    * `hostname` - name advertised via mDNS (without the `.local` suffix)
    * `eth-main` - objects with keys
        * `interface` - interface name, such as `eth0`
        * `ip` - local IPv4 address

* `switch` - object with keys:
    * `ip` - IPv4 address accessible from the main Ethernet connection
    * `username`, `password` - credentials for the admin console

* `wifi` - object with keys:
    * `ap-name` - the Nest Wifi Pro's SSID
    * `password` - the Nest Wifi Pro's WPA2 password
