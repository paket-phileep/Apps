const { merge } = require("webpack-merge");
const common = require("./webpack.common.js");

module.exports = merge(common, {
  mode: "production",
  entry: "./apps/root.jsx",
  output: {
    filename: "bundle.js",
    library: {
      type: "commonjs",
      name: "remoteLibrary",
    },
  },
});
