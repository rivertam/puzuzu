import { cloneDeep } from 'lodash';
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
  clueNumber?: number;
  userSolution: string;
  realSolution: string;
};

function Scoreboard({ wrongCells }: { wrongCells: number }) {
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const success = wrongCells === 0;

  useEffect(() => {
    const start = Date.now();

    if (success) {
      return;
    }

    const interval = setInterval(() => {
      const now = Date.now();

      setElapsedSeconds(Math.round((now - start) / 1000));
    }, 1000);

    return () => {
      clearInterval(interval);
    };
  }, [success]);

  const content = success
    ? `Winnar! ${elapsedSeconds}s`
    : `${elapsedSeconds}s | ${wrongCells} left`;

  return (
    <text
      height={3}
      width={20}
      right={0}
      top={0}
      content={content}
      {...commonBoxProperties}
    />
  );
}

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
          bg: 'red',
        };
      }
      case 'activeClue': {
        return {
          fg: 'white',
          bg: 'blue',
        };
      }
    }
  })();
  const content = (() => {
    if (props.clueNumber != null) {
      return `⁰${props.userSolution}`;
    }
    return ` ${props.userSolution}`;
  })();

  return (
    <text
      style={style}
      top={props.row}
      left={props.column * 3}
      width={3}
      height={1}
      content={content}
    />
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
  transpose(): void;
  next(): void;
  back(): void;
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
    transpose() {
      setDirection((dir) => (dir === 'down' ? 'across' : 'down'));
    },
    next() {
      if (direction === 'down') {
        this.down();
      } else {
        this.right();
      }
    },
    back() {
      if (direction === 'down') {
        this.up();
      } else {
        this.left();
      }
    },
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

    screen.key('space', () => {
      activeCell.transpose();
    });
  }, [screen]);

  useEffect(() => {
    // don't let user keep typing if they have already succeeded
    if (wrongCells.length === 0) {
      return;
    }

    const alphabet = [
      'a',
      'b',
      'c',
      'd',
      'e',
      'f',
      'g',
      'h',
      'i',
      'j',
      'k',
      'l',
      'm',
      'n',
      'o',
      'p',
      'q',
      'r',
      's',
      't',
      'u',
      'v',
      'w',
      'x',
      'y',
      'z',
    ];

    const onKeyPress = (ch: string) => {
      setUserSolution((currentSolution) => {
        const newSolution = cloneDeep(currentSolution);
        newSolution[activeCell.row][activeCell.column] = ch.toUpperCase();
        return newSolution;
      });
      activeCell.next();
    };

    alphabet.forEach((letter) => {
      screen.key(letter, onKeyPress);
    });

    const onBackspace = () => {
      setUserSolution((currentSolution) => {
        const newSolution = cloneDeep(currentSolution);
        newSolution[activeCell.row][activeCell.column] = ' ';
        return newSolution;
      });

      activeCell.back();
    };

    screen.key('backspace', onBackspace);

    return () => {
      alphabet.forEach((letter) => {
        screen.unkey(letter, onKeyPress);
      });

      screen.unkey('backspace', onBackspace);
    };
  }, [screen, activeCell]);

  const downClue = puzzle.getDownClue(activeCell.row, activeCell.column);
  const acrossClue = puzzle.getAcrossClue(activeCell.row, activeCell.column);

  const activeClue = activeCell.direction === 'down' ? downClue : acrossClue;

  const [userSolution, setUserSolution] = useState(() => {
    return grid.map((rowCells) => rowCells.map(() => ' '));
  });

  // check whether the user's solution wins
  const wrongCells = useMemo(() => {
    const cells: Array<CellCoordinates> = [];
    for (let row = 0; row < grid.length; ++row) {
      for (let column = 0; column < grid[row].length; ++column) {
        if (grid[row][column].black) {
          continue;
        }

        if (grid[row][column].solution !== userSolution[row][column]) {
          cells.push({ row, column });
        }
      }
    }

    return cells;
  }, [userSolution, grid]);

  return (
    <>
      <box bottom={6} width="75%">
        <Scoreboard wrongCells={wrongCells.length} />
        <box
          label={puzzle.title}
          top="center"
          left="center"
          width={puzzle.width * 3 + 2}
          height={puzzle.height + 2}
          {...commonBoxProperties}
        >
          {grid.map((rowCells, row) =>
            rowCells.map(({ black }, column) => {
              const kind = (() => {
                if (black) {
                  return 'black';
                }

                if (activeCell.row === row && activeCell.column === column) {
                  return 'active';
                }

                if (activeCell.direction === 'down') {
                  if (
                    downClue.column === column &&
                    downClue.row <= row &&
                    downClue.row + downClue.length > row
                  ) {
                    return 'activeClue';
                  }
                } else {
                  if (
                    acrossClue.row === row &&
                    acrossClue.column <= column &&
                    acrossClue.column + acrossClue.length > column
                  ) {
                    return 'activeClue';
                  }
                }

                return 'inactive';
              })();

              return (
                <Cell
                  key={`${row}-${column}`}
                  row={row}
                  column={column}
                  kind={kind}
                  userSolution={userSolution[row][column]}
                  realSolution={grid[row][column].solution}
                  clueNumber={(() => {
                    const clueNumber = clues.across
                      .concat(clues.down)
                      .find(
                        (clue) => clue.row === row && clue.column === column,
                      )?.clueNumber;
                    if (clueNumber != null) {
                      return clueNumber;
                    }
                  })()}
                />
              );
            }),
          )}
        </box>
      </box>

      <text
        label={
          activeClue
            ? `Clue #${activeClue?.clueNumber} ${activeCell.direction}`
            : '<no clue>'
        }
        width="75%"
        height={3}
        bottom={0}
        left={0}
        {...commonBoxProperties}
      >
        {activeCell.direction === 'across' ? acrossClue.text : downClue.text}
      </text>

      <list
        label="Across"
        style={{ item: { fg: 'white' }, selected: { fg: 'blue' } }}
        items={clues.across.map((clue) => `#${clue.clueNumber}. ${clue.text}`)}
        selected={clues.across.findIndex(
          (clue) => clue.clueNumber === acrossClue?.clueNumber,
        )}
        top={0}
        right={0}
        width="25%"
        height="50%"
        {...commonBoxProperties}
      />

      <list
        label="Down"
        items={clues.down.map((clue) => `#${clue.clueNumber}. ${clue.text}`)}
        selected={clues.down.findIndex(
          (clue) => clue.clueNumber === downClue?.clueNumber,
        )}
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
  screen.key(['escape', 'C-c'], () => {
    process.exit(0);
  });

  render(<App puzzle={puzzle} screen={screen} />, screen);
});

program.parse(process.argv);
