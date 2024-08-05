/// <reference types="vitest" />

import { defineConfig } from "vite"
import viteTsConfigPaths from "vite-tsconfig-paths"

export default defineConfig({
    cacheDir: "../../../node_modules/.vite/src-typescript-fiber-kernel",

    plugins: [
        viteTsConfigPaths({
            root: "../../../",
        }),
    ],

    // Configuration for building your library.
    // See: https://vitejs.dev/guide/build.html#library-mode
    build: {
        lib: {
            // Could also be a dictionary or array of multiple entry points.
            entry: "src/index.ts",
            name: "@meshx-org/fiber-kernel",
            fileName: "index",
            // Change this to the formats you want to support.
            // Don't forgot to update your package.json as well.
            formats: ["es", "cjs"],
        },
        rollupOptions: {
            // External packages that should not be bundled into your library.
            external: ["@meshx-org/fiber-sys", "@meshx-org/fiber-types", "@meshx-org/fiber-ts"],
        },
    },

    test: {
        reporters: ["default"],
        globals: true,
        environment: "jsdom",
        include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
        cache: {
            dir: "../../../node_modules/.vitest",
        },
    },
})
