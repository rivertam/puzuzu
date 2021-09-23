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

    println!("Across");
    println!("------");

    for clue in puzzle.clues.across.iter() {
        println!("{}. {}", clue.clue_number, clue.text);
    }

    println!("");
    println!("Down");
    println!("----");

    for clue in puzzle.clues.down.iter() {
        println!("{}. {}", clue.clue_number, clue.text);
    }

    Ok(())
}
