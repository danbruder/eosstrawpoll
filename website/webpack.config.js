const HtmlWebpackPlugin = require("html-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const CompressionPlugin = require("compression-webpack-plugin");
const path = require("path");
const webpack = require("webpack");

const DIST_DIR = path.resolve(__dirname, "dist");
const IS_PROD = process.env.NODE_ENV === "production";
const TARGET_DIR = "./build/";

console.log("PRODUCTION?", IS_PROD);

function plugins() {
    const common = [
        new HtmlWebpackPlugin({
            template: "static/index.html",
            filename: "index.html",
        }),
        new webpack.EnvironmentPlugin([
            "NODE_ENV",
            "DEFAULT_ENDPOINT",
            "DEFAULT_CHAIN_ID",
        ]),
    ]
    if (IS_PROD) {
        return [
            ...common,
            new CopyWebpackPlugin([{
                from: TARGET_DIR + "eosstrawpoll.wasm",
                to: "index.wasm"
            }, {
                from: "./dist/index.css",
                to: "index.css",
            }]),
            new CompressionPlugin({
                test: /\.(html|css|js|wasm)$/,
            })
        ];
    } else {
        return [
            ...common,
            new CopyWebpackPlugin([{
                from: TARGET_DIR + "eosstrawpoll.wasm",
                to: "index.wasm"
            }]),
        ];
    }
}

const config = {
    mode: IS_PROD ? "production" : "development",
    entry: {
        index: TARGET_DIR + "eosstrawpoll.js",
    },
    node: {
        fs: "empty"
    },
    output: {
        path: DIST_DIR,
        filename: "[name].js",
        publicPath: "/",
    },
    optimization: {
        splitChunks: {
            cacheGroups: {
                commons: {
                    test: /[\\/]node_modules[\\/]/,
                    name: 'vendor',
                    chunks: 'all'
                }
            }
        }
    },
    performance: {
        hints: false,
    },
    module: {
        rules: [
            {
                test: /eosstrawpoll\.js$/,
                loader: 'string-replace-loader',
                options: {
                    search: 'fetch( "eosstrawpoll.wasm"',
                    replace: 'fetch( "/index.wasm"',
                }
            },
        ]
    },
    plugins: plugins(),
};

module.exports = config;