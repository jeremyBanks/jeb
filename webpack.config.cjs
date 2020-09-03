module.exports = {
  context: __dirname,
  target: "webworker",
  entry: "./stadia.run/-/edge-worker.mjs",
  mode: "development",
  devtool: "cheap-module-source-map",
  output: {
    globalObject: "this"
  }
};
