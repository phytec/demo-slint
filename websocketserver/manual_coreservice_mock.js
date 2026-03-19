// Copyright (c) 2025 Cloudflight. All rights reserved. 
const keypress = require('keypress');
const api = require('./api');

keypress(process.stdin);

const WebSocketServer = require('ws');

// Creating a new websocket server
const wss_ui_events = new WebSocketServer.Server({ port: 3000 });

let product_id = 5;

// Creating connection using websocket
wss_ui_events.on('connection', (ws) => {
    console.log('New client connected');

    ws.on('message', (dataString) => {
        console.log(`Client has sent us: ${dataString}`);

        var data = JSON.parse(dataString);
        const response = api.getResponse(data);
        if (response) {
            wss_ui_events.clients.forEach((client) => client.send(response));
        }
    });

    ws.on('close', () => {
        console.log('The client has disconnected');
    });

    ws.onerror = () => {
        console.log('Some Error occurred');
    };
});
console.log('The WebSocket server is running on ws://localhost:3000/frontend');


process.stdin.on('keypress', function (ch, key) {
    console.log('got "keypress"', key);
    if (key && key.ctrl && key.name == 'c') {
        process.stdin.pause();
        process.exit();
    }
    if (key && Number(key.name)) { // Note: requires holding alt for some reason
        product_id = Number(key.name);
        console.log("set next product id to", product_id);
    }
    if (key && key.name == 'a') {
        product_id = 1;
        console.log("set next product id to", 1);
    }
    if (key && key.name == 's') {
        product_id = 2;
        console.log("set next product id to", 2);
    }
    if (key && key.name == 'd') {
        product_id = 3;
        console.log("set next product id to", 3);
    }
    if (key && key.name == 'f') {
        product_id = 4;
        console.log("set next product id to", 4);
    }
    if (key && key.name == 'g') {
        product_id = 5;
        console.log("set next product id to", 5);
    }
    if (key && key.name == 'q') {
        wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(0)));
    }
    if (key && key.name == 'w') {
        wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(1)));
    }
    if (key && key.name == 'e') {
        wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(2, product_id)));
    }
    if (key && key.name == 'r') {
        wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(3)));
    }
    if (key && key.name == 't') {
        wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(4)));
    }
    if (key && key.name == 'x') {
        wss_ui_events.clients.forEach((client) => client.send(
            JSON.stringify({"jsonrpc":"2.0", method: "updateErrorDialog", params: {text: "error!"}})
        ));
    }
    if (key && key.name == 'c') {
        wss_ui_events.clients.forEach((client) => client.send(
            JSON.stringify({"jsonrpc":"2.0", method: "updateErrorDialog", params: {}})
        ));
    }
});

process.stdin.setRawMode(true);
process.stdin.resume();
