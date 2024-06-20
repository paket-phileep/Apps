const HtmlWebpackPlugin = require("html-webpack-plugin");
const { merge } = require("webpack-merge");
const { ModuleFederationPlugin } = require("@rspack/core").container;
const common = require("./webpack.common.js");
// const { rspack } = require("@rspack/core");

delete common.plugins;
module.exports = merge(common, {
  experiments: {
    css: true,
  },
  mode: "development",
  entry: "./src",
  plugins: [
    // new pluginLightningcss(),
    new HtmlWebpackPlugin({
      template: "./public/index.html",
    }),
    new ModuleFederationPlugin({
      name: "remoteLibrary",
      filename: "remoteEntry.js",
      exposes: {
        // "./Button": "./src/components/Button.jsx",
        // "./Header": "./src/components/Header.jsx",
      },
    }),
  ],
  devServer: {
    port: 49155,
    hot: true,
    compress: true,
    watchFiles: ["src/**/*"],
  },
});
