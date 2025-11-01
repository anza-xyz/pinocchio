#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...args] = cliArguments();

const targetDir = path.join(workingDirectory, folder);

await $`tombi lint -- ${targetDir}`;
