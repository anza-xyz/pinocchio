#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getToolchainArgument,
  popArgument,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...args] = cliArguments();

const lintArgs = [
  '-Zunstable-options',
  '--all-targets',
  '--all-features',
  '--no-deps',
  '--',
  '--deny=warnings',
  ...args,
];

const fix = popArgument(lintArgs, '--fix');
const toolchain = getToolchainArgument('lint');
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

// Check the client using Clippy.
if (fix) {
  await $`cargo ${toolchain} clippy --manifest-path ${manifestPath} --fix ${lintArgs}`;
} else {
  await $`cargo ${toolchain} clippy --manifest-path ${manifestPath} ${lintArgs}`;
}
