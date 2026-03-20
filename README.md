# Slint Frontend Showcase for an AI-powered Coffee Machine

Embedded frontend showcase made with [slint](https://slint.dev/) for controlling an AI-powered coffee machine, which detects coffee cups and selects a product based on the type of cup. The frontend itself communicates with the backend (the coffee machine) via a websocket connection. A mock backend is included in this repository.

## Prerequisites

Slint uses Rust, so Rust has to be installed. For development (mock websocket/REST Server) NodeJS is needed.

  * **[Rust compiler](https://www.rust-lang.org/tools/install)** (1.75 or newer)
  * **[Node.js](https://nodejs.org/download/release/v16.19.1/)**
  * **[npm](https://www.npmjs.com/)**

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

## Adapt websocket endpoints
The application expects the websocket server endpoint to be reachable on the same device at
`ws:://localhost:3000/frontend`.

To use a different websocket server endpoint, you can place a `config.yaml` file in
`/usr/share/coffee-app`, with the custom address set in the parameter `websocket_address`. E.g.:
```sh
echo 'websocket_address: ws://192.168.3.10:3000/frontend' >| /usr/share/coffee-app/config.yaml
```

## Licensing

The code in this repository is licensed under MIT, see [LICENSE](LICENSE), except for the fonts
in `ui/assets/fonts/` which are licensed under [OFL](ui/assets/fonts/Roboto/OFL.txt).

Note that this project uses slint, which is licensed under
[GPLv3](https://choosealicense.com/licenses/gpl-3.0/). If this software is distributed
together with that library, then it must be distributed under the conditions of GPL. If this is not
an option, then a [different license can be purchased for slint](https://slint.dev/pricing).
