import type * as wasmType from '../rust/pkg';

type Clue = {
  clueNumber: number;
  text: string;
  cellIndex: number;
  length: number;
};

type Clues = {
  across: Array<Clue>;
  down: Array<Clue>;
};

type Grid = Array<Array<{ black: boolean }>>;

export class Puzzle {
  static async fromPuz(puzData: Uint8Array) {
    const wasm = await import('../rust/pkg');
    return new Puzzle(wasm.parsePuz(puzData));
  }

  private constructor(private puzzle: wasmType.Puzzle) {}

  public get title(): string {
    return this.puzzle.title;
  }

  public clues(): Clues {
    return this.puzzle.clues();
  }

  public grid(): Grid {
    return this.puzzle.grid();
  }
}
