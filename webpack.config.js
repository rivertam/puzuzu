const path = require('path');
const webpackNodeExternals = require('webpack-node-externals');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
  entry: {
    lib: './src/lib.ts',
    tui: './src/puzuzu-tui.js',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  externals: [webpackNodeExternals()],
  output: {
    filename: '[name].js',
    path: __dirname + '/dist',
    publicPath: '',
    globalObject: 'this',
    library: {
      name: 'puzuzu',
      type: 'umd',
    },
  },

  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, 'rust'),
    }),
  ],
  experiments: {
    syncWebAssembly: true,
  },
  target: 'node',
};
