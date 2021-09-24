use anyhow::Result;
use puzuzu::Puzzle;
use std::io;
enum Screen {
    Normal,
    Help,
    Stats,
}

fn main() -> Result<()> {
    let bytes = std::fs::read("../test_files/zack.puz").unwrap();
    let puzzle = Puzzle::from_puz(bytes).unwrap();

    // println!("Across");
    // println!("------");

    // for clue in puzzle.clues.across.iter() {
    //     println!("{}. {}", clue.clue_number, clue.text);
    // }

    // println!("");
    println!("Down");
    println!("----");

    for clue in puzzle.clues.down.iter() {
        println!("{}. {}", clue.clue_number, clue.text);
    }

    println!(
        "Grid JSON: {}",
        serde_json::to_string(&puzzle.grid()).unwrap()
    );

    for row in 0..puzzle.width() {
        for col in 0..puzzle.height() {
            print!(
                "{}",
                puzzle.fill.chars().nth(row * puzzle.width() + col).unwrap()
            )
        }
    }
    println!("");

    println!("Fill: {}", puzzle.fill);

    let clue = puzzle.get_down_clue(3, 2).unwrap();
    println!("{:?} :)", clue);

    Ok(())
}
