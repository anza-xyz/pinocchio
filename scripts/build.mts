#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, workingDirectory } from './setup/shared.mts';

const [folder, ...args] = cliArguments();

const buildArgs = [...args, '--', '--all-targets', '--all-features'];
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

await $`cargo-build-sbf --manifest-path ${manifestPath} ${buildArgs}`;
