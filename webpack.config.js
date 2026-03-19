// Copyright (c) 2025 Cloudflight. All rights reserved. 
const path = require('path');

module.exports = {
  entry: './websocketserver/auto_coreservice_mock.js',
  target: 'node',
  output: {
    filename: 'auto_coreservice_mock.js',
    path: path.resolve(__dirname, 'dist'),
  },
};
