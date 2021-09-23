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

import { Grid, Puzzle } from './lib';
import { useEffect, useMemo, useState } from 'react';

const commonBoxProperties = {
  border: { type: 'line' },
  style: { border: { fg: 'green' } },
} as const;

type CellProps = {
  key: string;
  column: number;
  row: number;
  kind: 'black' | 'active' | 'activeClue' | 'inactive';
};

function Cell(props: CellProps) {
  const style = (() => {
    switch (props.kind) {
      case 'black': {
        return {
          fg: 'white',
          bg: 'black',
        };
      }
      case 'inactive': {
        return {
          fg: 'black',
          bg: 'white',
        };
      }
      case 'active': {
        return {
          fg: 'white',
          bg: 'blue',
        };
      }
    }
  })();

  return (
    <text
      style={style}
      top={props.row + 1}
      left={props.column * 3 + 1}
      width={3}
      height={1}
      padding={0}
      align="center"
      valign="middle"
    >
      h
    </text>
  );
}

type CellCoordinates = { column: number; row: number };

function firstCell(grid: Grid): CellCoordinates {
  for (let row = 0; row < grid.length; ++row) {
    for (let column = 0; column < grid[column].length; ++column) {
      if (!grid[row][column].black) {
        return { column, row };
      }
    }
  }

  throw new Error('no first cell found in grid');
}

type ActiveCell = CellCoordinates & {
  next(): void;
  left(): void;
  right(): void;
  up(): void;
  down(): void;
  direction: 'across' | 'down';
};

function useActiveCell(grid: Grid): ActiveCell {
  const [activeCell, setActiveCell] = useState(() => firstCell(grid));
  const [direction, setDirection] = useState<'across' | 'down'>('across');

  return {
    ...activeCell,
    direction,
    left() {
      setActiveCell(({ row, column }) => {
        for (let newColumn = column - 1; newColumn >= 0; --newColumn) {
          if (!grid[row][newColumn].black) {
            return { row, column: newColumn };
          }
        }

        return { row, column };
      });

      setDirection('across');
    },
    right() {
      setActiveCell(({ row, column }) => {
        for (
          let newColumn = column + 1;
          newColumn < grid[row].length;
          ++newColumn
        ) {
          if (!grid[row][newColumn].black) {
            return { row, column: newColumn };
          }
        }

        return { row, column };
      });

      setDirection('across');
    },
    up() {
      setActiveCell(({ row, column }) => {
        for (let newRow = row - 1; newRow >= 0; --newRow) {
          if (!grid[newRow][column].black) {
            return { row: newRow, column };
          }
        }

        return { row, column };
      });

      setDirection('down');
    },
    down() {
      setActiveCell(({ row, column }) => {
        for (let newRow = row + 1; newRow < grid.length; ++newRow) {
          if (!grid[newRow][column].black) {
            return { row: newRow, column };
          }
        }

        return { row, column };
      });

      setDirection('down');
    },

    next() {},
  };
}

// Rendering a simple centered box
function App({
  puzzle,
  screen,
}: {
  puzzle: Puzzle;
  screen: blessed.Widgets.Screen;
}) {
  const { clues, grid } = useMemo(() => {
    return {
      clues: puzzle.clues(),
      grid: puzzle.grid(),
    };
  }, [puzzle]);

  const activeCell = useActiveCell(grid);

  useEffect(() => {
    screen.key('left', () => {
      activeCell.left();
    });

    screen.key('right', () => {
      activeCell.right();
    });

    screen.key('down', () => {
      activeCell.down();
    });

    screen.key('up', () => {
      activeCell.up();
    });
  }, [screen]);

  return (
    <>
      <box
        label={puzzle.title}
        top="center"
        width="75%"
        height="100%"
        {...commonBoxProperties}
      >
        {grid.map((row, rowIndex) =>
          row.map(({ black }, columnIndex) => {
            const kind = (() => {
              if (black) {
                return 'black';
              }

              if (
                activeCell.row === rowIndex &&
                activeCell.column === columnIndex
              ) {
                return 'active';
              }

              return 'inactive';
            })();

            return (
              <Cell
                key={`${rowIndex}-${columnIndex}`}
                row={rowIndex}
                column={columnIndex}
                kind={kind}
              />
            );
          }),
        )}
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

  render(<App puzzle={puzzle} screen={screen} />, screen);
});

program.parse(process.argv);
