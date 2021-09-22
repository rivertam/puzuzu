use anyhow::Result;
use puzuzu::Puzzle;
use std::io;
enum Screen {
    Normal,
    Help,
    Stats,
}

fn main() -> Result<()> {
    let bytes = std::fs::read("./test_files/washpost.puz").unwrap();
    let puzzle = Puzzle::from_puz(bytes).unwrap();

    let clues = puzzle.clues()?;

    println!("Across");
    println!("------");

    for clue in clues.across.iter() {
        println!("{}. {}", clue.number, clue.clue);
    }

    println!("");
    println!("Down");
    println!("----");

    for clue in clues.down.iter() {
        println!("{}. {}", clue.number, clue.clue);
    }

    Ok(())
}
