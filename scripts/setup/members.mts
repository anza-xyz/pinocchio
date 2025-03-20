import 'zx/globals';
import { getCargo } from './shared.mts';

const members = getCargo().workspace['members'] as string[];
await $`echo members=${JSON.stringify(members)} >> $GITHUB_OUTPUT`;
