# Slint Frontend Showcase for an AI-powered Coffee Machine

Embedded frontend showcase made with [slint](https://slint.dev/) for controlling an AI-powered coffee machine, which detects coffee cups and selects a product based on the type of cup. The frontend itself communicates with the backend (the coffee machine) via a websocket connection. A mock backend is included in this repository.

## Prerequisites

Slint uses Rust, so Rust has to be installed. To compile for a PHYTEC board cross compiler tools are needed. For development (mock websocket/REST Server) NodeJS is needed.

  * **[Rust compiler](https://www.rust-lang.org/tools/install)** (1.73 or newer)
  * Cross compile tools: `cargo install cross --git https://github.com/cross-rs/cross`
  * **[Node.js](https://nodejs.org/download/release/v16.19.1/)**
  * **[npm](https://www.npmjs.com/)**

To build the application for the embedded device, a recent linux distribution (e.g. Ubuntu 22.04)
with docker installed is required.

## Build locally

- Precondition:
  Websocket and REST Server must run, otherwise the application will fail.

  You can also start the Mock websocket and REST Server:
  ```
  npm install --include dev
  cd websocketserver
  node manual_coreservice_mock.js
  ```

- Build and run
  ```
  cargo run
  ```

## Build for PHYTEC board

Tested on the phyBOARD®-i.MX 8MP with the following build version:
```
ampliPHY Vendor xwayland (Phytec Vendor Distribution) BSP-Yocto-NXP-i.MX8MP-PD22.1.1
```
Expects weston wayland to be running on the board.

To build and deploy it, follow the following steps:
- Cross compile the application for the phyBOARD:
  ```sh
  cross build --target aarch64-unknown-linux-gnu --release
  ```
- Deploy the application to the device:
  ```sh
  ./deploy.sh --target-ip=192.168.3.11 --release
  ```
  This will copy the application to `/opt/coffeemachine-app/` and install it as an systemd service
  and start it. Be sure that the websocket backend is reachable (defaults to
  `ws://localhost:3000/frontend` on the device itself).
- To run the application manually from a ssh terminal, simply stop the service and run it with an
  appropriate environment:
  ```sh
  systemctl stop coffeeapp.service
  WAYLAND_DISPLAY=wayland-0 SLINT_FULLSCREEN=1 /opt/coffeemachine-app/NextCoffee
  ```
- To run the application in fullscreen and hide the toolbar, edit `/etc/xdg/weston/weston.ini` on
  the device and add `shell=kiosk-shell.so` to the `core` section:
  ```toml
  #/etc/xdg/weston/weston.ini
  [core]
  shell=kiosk-shell.so
  ```
  Then, restart weston with `systemctl restart weston`, or restart the device.


## Adapt websocket endpoints
The application expects the websocket server endpoint to be reachable on the same device at
`ws:://localhost:3000/frontend`.

To use a different websocket server endpoint, you can place a `config.yaml` file next to the
application, with the custom address set in the parameter `websocket_address`. E.g.:
```sh
echo 'websocket_address: ws://192.168.3.10:3000/frontend' >| /opt/coffeemachine-app/config.yaml
```

## Build and deploy auto CoreService mock

To run the application without an actual coffeemachine, there is a Mock CoreService available which
will automatically switch through the states. To install it on the phyBOARD, you need to bundle it
first. Also, NodeJS needs to be installed on the board (tested with v12.21.0).

To bundle the mock (on the dev machine):
```sh
npm install --include dev
npx webpack
```

This should create a bundled mock in `dist/auto_coreservice_mock.js`. To install it together with a
service and the application itself, run the `deploy.sh` script with the additional argument
`--install-mock`:
```sh
./deploy.sh --target-ip=192.168.3.11 --release --install-mock
```

## Licensing

The code in this repository is licensed under MIT, see [LICENSE](LICENSE), except for the fonts
in `ui/assets/fonts/` which are licensed under [OFL](ui/assets/fonts/Roboto/OFL.txt).

Note that this project uses slint, which is licensed under
[GPLv3](https://choosealicense.com/licenses/gpl-3.0/). If this software is distributed
together with that library, then it must be distributed under the conditions of GPL. If this is not
an option, then a [different license can be purchased for slint](https://slint.dev/pricing).
