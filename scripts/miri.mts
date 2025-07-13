#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, getToolchainArgument } from './setup/shared.mts';

const args = cliArguments();
const toolchain = getToolchainArgument('lint');

await $`cargo ${toolchain} miri test ${args}`;
