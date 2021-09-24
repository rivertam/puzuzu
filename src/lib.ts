import type * as wasmType from '../rust/pkg';

type Clue = {
  clueNumber: number;
  text: string;
  row: number;
  column: number;
  length: number;
};

type Clues = {
  across: Array<Clue>;
  down: Array<Clue>;
};

export type Grid = Array<Array<{ black: boolean; solution: string }>>;

export class Puzzle {
  static async fromPuz(puzData: Uint8Array) {
    const wasm = await import('../rust/pkg');
    return new Puzzle(wasm.parsePuz(puzData));
  }

  private constructor(private puzzle: wasmType.Puzzle) {}

  public get title(): string {
    return this.puzzle.title;
  }

  public get height(): number {
    return this.puzzle.height;
  }

  public get width(): number {
    return this.puzzle.width;
  }

  public get solutionState(): 'Unlocked' | 'Locked' {
    return this.puzzle.solutionState as 'Unlocked' | 'Locked';
  }

  public clues(): Clues {
    return this.puzzle.clues();
  }

  public grid(): Grid {
    return this.puzzle.grid();
  }

  public getAcrossClue(row: number, column: number): Clue | null {
    return this.puzzle.getAcrossClue(row, column);
  }

  public getDownClue(row: number, column: number): Clue | null {
    return this.puzzle.getDownClue(row, column);
  }
}
