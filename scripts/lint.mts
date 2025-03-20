import 'zx/globals';

const args = process.argv.slice(2);

await Promise.all([
  $`tsx ./scripts/clippy.mts ${args}`,
  $`tsx ./scripts/doc.mts ${args}`,
  $`tsx ./scripts/hack.mts ${args}`,
]);
