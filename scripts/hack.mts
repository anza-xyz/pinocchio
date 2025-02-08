#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getToolchainArgument,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...args] = cliArguments();
const checkArgs = ['--all-targets', '--feature-powerset', ...args];

const toolchain = getToolchainArgument('lint');
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

await $`cargo ${toolchain} hack check --manifest-path ${manifestPath} ${checkArgs}`;
