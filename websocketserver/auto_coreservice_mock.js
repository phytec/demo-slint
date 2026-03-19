#!/usr/bin/node
// Copyright (c) 2025 Cloudflight. All rights reserved. 

const WebSocketServer = require('ws');
const api = require('./api');

// Creating a new websocket server
const wss_ui_events = new WebSocketServer.Server({ port: 3000 });

let currentState = 0;
let autoProductId = 0;

setInterval(() => {
    if (currentState === 0) {
        setTimeout(() => {
            if (currentState === 0) {
                wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(1)));
                console.log("Auto changing state to detecting");
                wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(2, autoProductId + 3)));
                console.log("Auto changing state to detected with product id", autoProductId + 3);
                currentState = 2;
                autoProductId = (autoProductId + 1) % 3;
            }
        }, 5000)
    }
    if (currentState === 3) {
        setTimeout(() => {
            if (currentState === 3) {
                currentState = 4;
                wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(4)));
                console.log("Auto changing state to done");
            }
        }, 6000)
    }
    if (currentState === 4) {
        setTimeout(() => {
            if (currentState === 4) {
                currentState = 0;
                wss_ui_events.clients.forEach((client) => client.send(api.getStateJson(0)));
                console.log("Auto changing state to idle");
            }
        }, 9000)
    }
}, 1000);


// Creating connection using websocket
wss_ui_events.on('connection', (ws) => {
    console.log('New client connected');

    // reset state machine on any new clients
    currentState = 0;
    autoProductId = 0;

    ws.on('message', (dataString) => {
        console.log(`Client has sent us: ${dataString}`);

        var data = JSON.parse(dataString);
        const response = api.getResponse(data);
        if (response) {
            wss_ui_events.clients.forEach((client) => client.send(response));
        }
        const updatedState = api.getNewStateFromRequest(data);
        if (updatedState != null) {
            currentState = updatedState;
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

process.stdin.resume();
