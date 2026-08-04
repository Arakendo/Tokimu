import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { XsltProcessor } from '../../../third-party/weaver-xslt/src/index.ts';

const consumerRoot = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(consumerRoot, '../../..');
const fixturesRoot = path.join(consumerRoot, 'fixtures');
const sourceIdentity = 'tokimu-resource://weaver-xslt-resource-space/stylesheet.xsl';

async function fixture(name: string): Promise<string> {
  return readFile(path.join(fixturesRoot, name), 'utf8');
}

async function main(): Promise<void> {
  const [source, stylesheet, expected, related] = await Promise.all([
    fixture('source.xml'),
    fixture('stylesheet.xsl'),
    fixture('expected.xml'),
    fixture('related.xml'),
  ]);
  const processor = new XsltProcessor(stylesheet, { sourceName: sourceIdentity });
  const interpreter = processor.transform(source, { execution: 'interpreter' });
  const automatic = processor.transform(source, { execution: 'auto' });

  if (interpreter.output.trim() !== expected.trim()) {
    throw new Error(
      `interpreter output diverged from fixtures/expected.xml: expected ${JSON.stringify(expected.trim())}, received ${JSON.stringify(interpreter.output)}`,
    );
  }
  if (automatic.output.trim() !== interpreter.output.trim()) {
    throw new Error(
      `auto execution output diverged from interpreter output: interpreter ${JSON.stringify(interpreter.output)}, auto ${JSON.stringify(automatic.output)}`,
    );
  }

  const evidence = {
    schema: 1,
    consumer: 'weaver-xslt-resource-space',
    selectedResources: ['source.xml', 'stylesheet.xsl', 'related.xml'],
    sourceIdentity,
    interpreter: interpreter.execution ?? { resolved: 'interpreter' },
    automatic: automatic.execution ?? { resolved: 'interpreter' },
    rawOutput: {
      interpreter: interpreter.output,
      automatic: automatic.output,
    },
    relatedResource: 'retained but intentionally unresolved pending Weaver public ResourceResolver API',
    output: interpreter.output,
  };
  const outputDirectory = path.join(repositoryRoot, 'target', 'weaver-xslt-resource-space');
  await mkdir(outputDirectory, { recursive: true });
  await writeFile(
    path.join(outputDirectory, 'baseline.json'),
    `${JSON.stringify(evidence, null, 2)}\n`,
    'utf8',
  );
  console.log(`weaver-xslt-resource-space baseline passed: ${JSON.stringify(evidence)}`);
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
