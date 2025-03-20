#!/usr/bin/env zx
import 'zx/globals';
import { cliArguments, getCargo, workingDirectory } from './setup/shared.mts';

const [folder, ...args] = cliArguments();
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

const isProcMacro = getCargo(folder).lib['proc-macro'] === true;

if (isProcMacro) {
  echo(
    chalk.yellow.bold('[ SKIPPED ]'),
    'Proc-macro found: only library targets with API surface that can be checked for semver.'
  );
} else {
  await $`cargo semver-checks --manifest-path ${manifestPath} ${args}`;
}
