const { merge } = require("webpack-merge");
const common = require("./webpack.common.js");
const { ModuleFederationPlugin } = require("@rspack/core").container;
delete common.plugins;
module.exports = merge(common, {
  mode: "production",
  entry: "./apps/root.jsx",
  plugins: [
    new ModuleFederationPlugin({
      name: "remoteLibrary",
      filename: "remoteEntry.js",
      exposes: {
        // "./Button": "./src/components/button.jsx",
      },
    }),
  ],
  output: {
    filename: "bundle.js",
    library: {
      type: "commonjs",
      name: "remoteLibrary",
    },
  },
});
