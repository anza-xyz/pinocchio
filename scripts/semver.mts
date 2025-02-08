#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, workingDirectory } from './setup/shared.mts';

const [folder, ...args] = cliArguments();
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

await $`cargo semver-checks --manifest-path ${manifestPath} ${args}`;
