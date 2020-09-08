module.exports = {
  context: __dirname,
  target: "webworker",
  entry: "./stadia.run/-/edge-worker.js",
  mode: "development",
  devtool: "cheap-module-source-map",
  output: {
    globalObject: "this"
  }
};
