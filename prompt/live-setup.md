# Live network topology

## High-level description

You are running on an Ubuntu system called the primary host. The primary host is
connected to a managed Ethernet switch via an Ethernet connection called the
main connection.

The main Ethernet connection is set up to access the rest of the internal
network, using DHCP for IPv4 and IPv6 address provisioning. The main Ethernet
connection also has a secondary fixed IPv4 address and netmask, which is used
to access the switch.

You have SSH access to another Ubuntu system on the local network. That system
exposes its IP addresses via via mDNS.

The internal network's main router is a Google Nest Wifi Pro with WiFi 6E. The
router is connected to the managed Ethernet switch, and is also configured to
work as a wireless Access Point. The main router's WAN port is connected to a
cable modem, and the main router's LAN port is connected to the Ethernet switch.

The internal network has three more Google Net Wifi Pro units. The units' LAN
ports are connected to the Ethernet switch.

The main router is connected to a cable modem, which provides Internet
connectivity for entire internal network.

The primary host's Internet connection may be provided by one of the following.

1. Main Ethernet connection, when the internal network works well.
2. WiFi connection from the Nest Wifi Pro connected to the main router.
3. Backup WiFi connection - when the home network is unstable.

## Machine-readable file schema

`data/network.json` in the project root contains an object with the following
keys:

* `primary` - object with keys:
    * `hostname` - name advertised via mDNS (without the `.local` suffix)
    * `eth-main` - object with keys
        * `interface` - interface name, such as `eth0`
        * `ip` - local IPv4 address

* `secondary` - object with keys:
    * `hostname` - name advertised via mDNS (without the `.local` suffix)

* `switch` - object with keys:
    * `ip` - IPv4 address accessible from the main Ethernet connection
    * `username`, `password` - credentials for the admin console

* `router` - object with keys:
    * `ip` - IPv4 address accessible from the main Ethernet connection

* `modem` - object with keys:
    * `ip` - IPv4 address accessible from the main Ethernet connection
    * `username`, `password` - credentials for the admin console

* `wifi` - object with keys:
    * `ap-name` - the Nest Wifi Pro's SSID
    * `password` - the Nest Wifi Pro's WPA2 password

* `backup-wifi` - object with keys:
    * `ap-name` - the backup Wifi's SSID
    * `password` - the backup Wifi's WPA2 password
