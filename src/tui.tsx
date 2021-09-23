//
// async function main() {
//   const fileContents = fs.readFileSync('../test_files/washpost.puz');
//
//   const puzzle = await Puzzle.fromPuz(Uint8Array.from(fileContents));
//   console.log('did the puzz', puzzle);
// }
//
// main();

import * as fs from 'fs';
import * as blessed from 'blessed';
import { render } from 'react-blessed';
import * as React from 'react';
import { Command } from 'commander';

import { Puzzle } from './lib';
import { useMemo } from 'react';

const commonBoxProperties = {
  border: { type: 'line' },
  style: { border: { fg: 'green' } },
} as const;

// Rendering a simple centered box
function App({ puzzle }: { puzzle: Puzzle }) {
  const { clues } = useMemo(() => {
    return {
      clues: puzzle.clues(),
    };
  }, [puzzle]);

  return (
    <>
      <box
        label={puzzle.title}
        top="center"
        width="75%"
        height="100%"
        {...commonBoxProperties}
      >
        Hello World!
      </box>
      <list
        label="Across"
        items={clues.across.map((clue) => `#${clue.clueNumber}. ${clue.text}`)}
        top={0}
        right={0}
        width="25%"
        height="50%"
        {...commonBoxProperties}
      ></list>

      <list
        label="Down"
        items={clues.down.map((clue) => `#${clue.clueNumber}. ${clue.text}`)}
        bottom={0}
        right={0}
        width="25%"
        height="50%"
        {...commonBoxProperties}
      ></list>
    </>
  );
}

const program = new Command();
program.option('-f --file <path>', 'file path');
program.action(async (args) => {
  const buffer = fs.readFileSync(args.file);
  const puzzle = await Puzzle.fromPuz(buffer);

  // Creating our screen
  const screen = blessed.screen({
    autoPadding: true,
    smartCSR: true,
    title: 'puzuzu',
  });

  // Adding a way to quit the program
  screen.key(['escape', 'q', 'C-c'], () => {
    return process.exit(0);
  });

  render(<App puzzle={puzzle} />, screen);
});

program.parse(process.argv);
