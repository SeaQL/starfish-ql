const dotenv = require('dotenv-webpack');
const path = require('path');

module.exports = {
  entry: "./src/bootstrap.js",
  plugins: [
    new dotenv({
      systemvars: true
    }),
  ],
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },
};
