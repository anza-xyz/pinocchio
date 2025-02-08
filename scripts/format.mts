#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getToolchainArgument,
  partitionArguments,
  popArgument,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...formatArgs] = cliArguments();

const toolchain = getToolchainArgument('format');
const fix = popArgument(formatArgs, '--fix');

const [cargoArgs, fmtArgs] = partitionArguments(formatArgs, '--');
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

// Format the client.
if (fix) {
  await $`cargo ${toolchain} fmt --manifest-path ${manifestPath} ${cargoArgs} -- ${fmtArgs}`;
} else {
  await $`cargo ${toolchain} fmt --manifest-path ${manifestPath} ${cargoArgs} -- --check ${fmtArgs}`;
}
