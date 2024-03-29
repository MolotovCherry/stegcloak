module.exports = {
    content: [
        "index.html",
        "src/**/*.rs"
    ],

    theme: {
        container: {
            center: true,
        },
    },

    safelist: [
        "hidden"
    ],

    darkMode: "media",

    daisyui: {
        darkTheme: "dark",
        themes: [
            "light",
            "dark",
            "dracula",
            "night",
            "dim",
            "cupcake",
            "valentine"
        ],
    },

    plugins: [require("@tailwindcss/typography"), require("daisyui")],
}
