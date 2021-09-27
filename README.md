# Puzuzu

> [Puzuuuuuzuuuuuu](https://www.youtube.com/watch?v=u0DhVzw6LrQ)

[![asciicast](https://asciinema.org/a/QyRtCVSFU3m0RgQY1njprx2Og.svg)](https://asciinema.org/a/QyRtCVSFU3m0RgQY1njprx2Og)

## CLI program

The primary deliverable of the Puzuzu project is currently a CLI program which
allows the user to solve crossword puzzles with a TUI.

### Usage

#### Installation

Packaged and delivered using [npm](https://www.npmjs.com/), the node package
manager.

```sh
npm i -g puzuzu
```

#### Running

```sh
puzuzu -f path/to/puz-file.puz
```

##### Controls

> TODO: Make controls visible from inside the TUI

- Typing will automatically advance the cursor to the next slot on the same row
- Space bar to go from "across" clue to "down" clue
- Backspace deletes the current square and moves back one
- C-c or Esc to quit

### Compatibility

This program has been tested on a small handful of files. The `./test_files`
directory is currently just cloned from
[puzpy](https://github.com/alexdej/puzpy), but only the `washpost.puz` test has
been fairly thoroughly tested. Additionally, my brother (Zack Berman)
contributed a mini (`zack.puz`) that has been thoroughly tested.

Known absent features include

- Rebus
- Pre-filled squares

### Known bugs

- The right panes don't highlight the active clues. The code seems to imply that
  it should, yet it doesn't. I'm fairly certain the active clues are properly
  kept because the lists jump around when you move to different clues on
  sufficiently large puzzles, but the styling is broken right now.
- Spacing is "best effort", meaning sometimes the puzzle will appear way too
  small and sometimes it will be too large for your view to render with no
  recourse. This is primarily a limitation that comes from being a TUI, so
  porting this to ReactDOM would be a good next step for making a full featured
  puzuzu client.
- All of the clues are noted with "⁰" rather than their actual number. While it
  seems possible to resolve this for clues 1-9, I haven't thought of a great way
  to solve it for clues with multiple digits, so I haven't really tried to
  resolve this.

### Implementation

The UI has been built using the following technologies:

- [Node.js](https://nodejs.org/en/)
  - Runtime for JavaScript
- [TypeScript](https://www.typescriptlang.org/)
  - Preprocessor to add compile-time types to JavaScript
- [React.js](https://reactjs.org/)
  - UI framework famous for web development but capable of hosting other types
    of UIs
- [react-blessed](https://github.com/Yomguithereal/react-blessed)
  - An experimental backend for React that renders the React component tree to
    [blessed](https://github.com/chjj/blessed)

## NPM Library

The library is a relatively thin wrapper around the wasm/Rust library. It's been
tested using both node and webpack, though it has yet to be tested in the
browser.

### Example usage

```typescript
import { Puzzle } from 'puzuzu';
import * as fs from 'fs';

async main() {
  const buffer = fs.readFileSync('./path/to/puzzle.puz');
  const puzzle = await Puzzle.fromPuz(buffer);

  const clues = puzzle.clues();
  // clues.across and clues.down are arrays of {
  //   clueNumber: number;
  //   text: string;
  //   row: number;
  //   column: number;
  //   length: number;
  // }
  console.log(clues.across);
  console.log(clues.down);
  // grid is a two dimensional array of { black: boolean, solution: string }
  const grid = puzzle.grid();
}
```

## Rust Crate

While the Rust Crate currently primarily exists to support TypeScript usecase,
it can be used standalone in either a wasm or Rust context.

The source code for the crate can be found in the `./rust` directory.

> TODO: More documentation for the Rust

### Acknowledgements

- I used @alexdej's [puzpy](https://github.com/alexdej/puzpy) as a reference
  implementation for much of the initial implementation and testing. I hope to
  eventually port all of the tests and functionality from

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
