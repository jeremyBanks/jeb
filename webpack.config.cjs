module.exports = {
  context: __dirname,
  target: "webworker",
  entry: "./stadia.run/-/src/workers/cloudflare.js",
  mode: "development",
  devtool: "cheap-module-source-map",
  output: {
    globalObject: "this"
  }
};
