// Copyright (c) 2025 Cloudflight. All rights reserved. 
module.exports = {
    products: [
        {
            "description": "80ml",
            "feasible": true,
            "iconId": 0,
            "productId": 1,
            "name": "Kaffee",
            "displayedName": "Coffee",
            "volume": 80,
            "funFact": "... are often traditional, efficient and know how to get things done. They sometimes appear moody but eventually they just want to keep things simple, just like the coffee they drink.",
            "ingredients": [
                {
                    "name": "Coffee",
                    "percentage": 1,
                    "color": "#FFA4BC"
                }
            ],
            "default": false
        },
        {
            "description": "125ml",
            "feasible": true,
            "iconId": 0,
            "productId": 2,
            "name": "Kaffee",
            "displayedName": "Coffee",
            "volume": 125,
            "funFact": "... are often traditional, efficient and know how to get things done. They sometimes appear moody but eventually they just want to keep things simple, just like the coffee they drink.",
            "ingredients": [
                {
                    "name": "Coffee",
                    "percentage": 1,
                    "color": "#FFA4BC"
                }
            ],
            "default": true
        },
        {
            "description": "150ml",
            "feasible": true,
            "iconId": 0,
            "productId": 3,
            "name": "Kaffee",
            "displayedName": "Coffee",
            "volume": 150,
            "funFact": "... are often traditional, efficient and know how to get things done. They sometimes appear moody but eventually they just want to keep things simple, just like the coffee they drink.",
            "ingredients": [
                {
                    "name": "Coffee",
                    "percentage": 1,
                    "color": "#FFA4BC"
                }
            ],
            "default": false
        },
        {
            "description": "",
            "feasible": true,
            "iconId": 1,
            "productId": 4,
            "name": "Espresso",
            "displayedName": "Espresso",
            "volume": 30,
            "funFact": "... are hard-working and task driven. They are multi-tasking and know how to get the job done. The bold taste of an espresso shot reflects their bold appearance.",
            "ingredients": [
                {
                    "name": "Espresso",
                    "percentage": 1,
                    "color": "#F3825F"
                }
            ],
            "default": true
        },
        {
            "description": "",
            "feasible": true,
            "iconId": 2,
            "productId": 5,
            "name": "Latte Macchiato",
            "displayedName": "Latte Macchiato",
            "volume": 220,
            "funFact": "... are said to be helpful and open-minded people. While they are generous to others they also tend to overextend themselves and ignore their own needs.",
            "ingredients": [
                {
                    "name": "Milk",
                    "percentage": 0.7,
                    "color": "#FDF8FF"
                },
                {
                    "name": "Milk Foam",
                    "percentage": 0.2,
                    "color": "#FFD5AE"
                },
                {
                    "name": "Espresso",
                    "percentage": 0.1,
                    "color": "#F3825F"
                }
            ],
            "default": true
        }
    ],

    getStateJson: function (state, productId = 0) {
        if (state == 0) {
            return JSON.stringify({"jsonrpc": "2.0", method: "updateState", params: {state: "idle"}});
        }
        if (state == 1) {
            return JSON.stringify({"jsonrpc": "2.0", method: "updateState", params: {state: "detecting", productId}});
        }
        if (state == 2) {
            return JSON.stringify({"jsonrpc": "2.0", method: "updateState", params: {state: "detected", productId}});
        }
        if (state == 3) {
            return JSON.stringify({"jsonrpc": "2.0", method: "updateState", params: {state: "producing", productId}});
        }
        if (state == 4) {
            return JSON.stringify({"jsonrpc": "2.0", method: "updateState", params: {state: "done"}});
        }
        return JSON.stringify({state});
    },

    getResponse: function (request) {
        if (request.params && request.params.productId && request.method === "selectProduct") {
            if (!isNaN(request.params.productId)) {
                return this.getStateJson(3, request.params.productId);
            }
        }

        if (request.method === "abortProduction") {
            return this.getStateJson(4);
        }

        if (request.method === "ping") {
            return JSON.stringify({jsonrpc: "2.0", id: request.id});
        }

        if (request.method === "getProducts" && request.id) {
            return JSON.stringify({
                jsonrpc: "2.0",
                id: request.id,
                result: this.products
            });
        }
        return null;
    },

    getNewStateFromRequest: function (request) {
        if (request.params && request.params.productId && request.method === "selectProduct") {
            if (!isNaN(request.params.productId)) {
                return 3;
            }
        }
        if (request.method === "abortProduction") {
            return 0;
        }
        return null;
    }
}
