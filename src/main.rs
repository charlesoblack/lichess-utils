use std::io;
use std::fs::File;
use pgn_reader::{Visitor, Skip, BufferedReader, SanPlus};

struct MoveCounter {
    moves: usize,
}

impl MoveCounter {
    fn new() -> MoveCounter {
        MoveCounter { moves: 0 }
    }
}

impl Visitor for MoveCounter {
    type Result = usize;

    fn begin_game(&mut self) {
        self.moves = 0;
    }

    fn san(&mut self, _san_plus: SanPlus) {
        self.moves += 1;
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) -> Self::Result {
        self.moves
    }
}

fn main() -> io::Result<()> {
    let pgn_file = File::open("../sample_games.pgn")?;
    let mut reader = BufferedReader::new(pgn_file);

    let mut counter = MoveCounter::new();
    reader.read_all(&mut counter)?;

    println!("{:?}", counter.moves);
    Ok(())
}