#!/usr/bin/env node
const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const localBinary = path.resolve(__dirname, "../../target/release/async-shell");
let command = "async-shell";
if (fs.existsSync(localBinary)) {
    command = localBinary;
}

const args = process.argv.slice(2);
const result = spawnSync(command, args, { stdio: "inherit" });

if (result.error) {
    if (result.error.code === "ENOENT") {
        console.error("\x1b[31mError: Native `async-shell` binary not found in PATH.\x1b[0m");
        process.exit(1);
    }
    console.error(result.error);
    process.exit(1);
}

process.exit(result.status ?? 0);
