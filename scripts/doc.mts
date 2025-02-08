#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getToolchainArgument,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...args] = cliArguments();
const docArgs = ['--all-features', '--no-deps', ...args];

const toolchain = getToolchainArgument('lint');
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

$.env['RUSTDOCFLAGS'] = '--cfg docsrs -D warnings';
await $`cargo ${toolchain} doc --manifest-path ${manifestPath} ${docArgs}`;
