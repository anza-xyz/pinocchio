#!/usr/bin/env zx
import { getSolanaVersion, getToolchain } from './shared.mts';

await $`echo "SOLANA_VERSION=${getSolanaVersion()}" >> $GITHUB_ENV`;
await $`echo "TOOLCHAIN_BUILD=${getToolchain('build')}" >> $GITHUB_ENV`;
await $`echo "TOOLCHAIN_FORMAT=${getToolchain('format')}" >> $GITHUB_ENV`;
await $`echo "TOOLCHAIN_LINT=${getToolchain('lint')}" >> $GITHUB_ENV`;
await $`echo "TOOLCHAIN_TEST=${getToolchain('test')}" >> $GITHUB_ENV`;
