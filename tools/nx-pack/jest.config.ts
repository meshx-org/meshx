/* eslint-disable */
export default {
    displayName: "tools-nx-pack",
    preset: "../../jest.preset.js",
    transform: {
        "^.+\\.[tj]s$": ["ts-jest", { tsconfig: "<rootDir>/tsconfig.spec.json" }],
    },
    moduleFileExtensions: ["ts", "js", "html"],
    coverageDirectory: "../../coverage/tools/nx-pack",
}
