/// <reference types="vitest" />

import { defineConfig } from "vite"
import viteTsConfigPaths from "vite-tsconfig-paths"

export default defineConfig({
    cacheDir: "../../../node_modules/.vite/src-typescript-fiber-sys",

    plugins: [
        viteTsConfigPaths({
            root: "../../../",
        }),
    ],

    test: {
        reporters: ["default"],
        globals: true,
        cache: {
            dir: "../../../node_modules/.vitest",
        },
        environment: "jsdom",
        include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
    },
})
