module.exports = {
    future: {},
    purge: [
        "./public/**/styles/*.css",
        "./public/**/*.html.tera",
        "./wasm/**/*.rs",
    ],
    theme: {
        extend: {},
    },
    variants: {},
    plugins: [
        require('tailwindcss'),
        require("autoprefixer")
    ],
}
