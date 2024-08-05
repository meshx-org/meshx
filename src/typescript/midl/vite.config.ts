import { defineConfig } from "vite"
import viteTsConfigPaths from "vite-tsconfig-paths"

export default defineConfig({
    cacheDir: "../../../node_modules/.vite/src-typescript-midl",

    plugins: [
        viteTsConfigPaths({
            root: "../../../",
        }),
        //nxViteTsPaths(),
    ],

    // Configuration for building your library.
    // See: https://vitejs.dev/guide/build.html#library-mode
    build: {
        lib: {
            // Could also be a dictionary or array of multiple entry points.
            entry: "src/index.ts",
            name: "@meshx-org/midl",
            fileName: "index",
            // Change this to the formats you want to support.
            // Don't forgot to update your package.json as well.
            formats: ["es", "cjs"],
        },
        rollupOptions: {
            // External packages that should not be bundled into your library.
            external: [],
        },
    },

    test: {
        globals: true,
        cache: { dir: "../../../node_modules/.vitest/src/typescript/midl" },
        environment: "jsdom",
        include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
        reporters: ["default"],
        coverage: { reportsDirectory: "../../../coverage/src/typescript/midl", provider: "v8" },
    },
})
