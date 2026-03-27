const path = require('path');
const VueLoaderPlugin = require('vue-loader/lib/plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

module.exports = (env, argv) => {
    const isDevelopment = argv.mode === 'development';
    return {
        mode: isDevelopment ? 'development' : 'production',
        devtool: isDevelopment ? 'inline-cheap-module-source-map' : false,
        entry: './src/main.js',
        output: {
            filename: 'SynoTelegramBot.js',
            path: path.resolve(__dirname, 'dist'),
        },
        module: {
            rules: [
                {
                    test: /\.vue$/,
                    loader: 'vue-loader',
                },
                {
                    test: /\.js$/,
                    exclude: /node_modules/,
                    use: {
                        loader: 'babel-loader',
                        options: {
                            rootMode: 'upward',
                        },
                    },
                },
                {
                    test: /\.css$/,
                    use: [MiniCssExtractPlugin.loader, 'css-loader'],
                },
            ],
        },
        resolve: {
            extensions: ['.js', '.vue', '.json'],
        },
        plugins: [
            new VueLoaderPlugin(),
            new MiniCssExtractPlugin({
                filename: 'style.css',
            }),
        ],
        externals: {
            vue: 'Vue',
        },
    };
};
