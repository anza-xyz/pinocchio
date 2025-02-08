#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getCargo,
  popArgument,
  workingDirectory,
} from './setup/shared.mts';

const [folder, ...args] = cliArguments();
const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

const fix = popArgument(args, '--dry-run');
const dryRun = argv['dry-run'] ?? false;

const [level] = args;
if (!level) {
  throw new Error('A version level — e.g. "patch" — must be provided.');
}

// Go to the client directory and install the dependencies.
cd(path.dirname(manifestPath));

// Publish the new version.
const releaseArgs = dryRun
  ? []
  : ['--no-push', '--no-tag', '--no-confirm', '--execute'];
await $`cargo release ${level} ${releaseArgs}`;

// Get the crate information.
const toml = getCargo(folder);
const crate = toml.package['name'];
const version = toml.package['version'];

// Expose the new version to CI if needed.
if (process.env.CI) {
  await $`echo "crate=${crate}" >> $GITHUB_OUTPUT`;
  await $`echo "version=${version}" >> $GITHUB_OUTPUT`;
}

// Stop here if this is a dry run.
if (dryRun) {
  process.exit(0);
}

// Soft reset the last commit so we can create our own commit and tag.
await $`git reset --soft HEAD~1`;

// Commit the new version.
await $`git commit -am "Publish ${crate} v${version}"`;

// Tag the new version.
await $`git tag -a ${crate}@v${version} -m "${crate} v${version}"`;
