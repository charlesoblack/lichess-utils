extern crate csv;

use std::io;
use std::fs::File;
use std::string::String;

use csv::Writer;
use serde::Serialize;

use pgn_reader::{Visitor, Skip, BufferedReader,
                 RawComment, RawHeader, SanPlus};

#[derive(Debug, Serialize, Clone)]
struct GameHeaders {
    event: String,
    game_link: String,
    white_player: String,
    black_player: String,
    result: String,
    date_played: String,
    time_played: String,
    white_elo: String,
    black_elo: String,
    white_rating_diff: String,
    black_rating_diff: String,
    eco: String,
    opening_name: String,
    time_control: String,
    initial_time: usize,
    increment: usize,
    termination: String,
}

impl GameHeaders {
    fn new() -> GameHeaders {
        GameHeaders {
            event: String::from(""),
            game_link: String::from(""),
            white_player: String::from(""),
            black_player: String::from(""),
            result: String::from(""),
            date_played: String::from(""),
            time_played: String::from(""),
            white_elo: String::from(""),
            black_elo: String::from(""),
            white_rating_diff: String::from(""),
            black_rating_diff: String::from(""),
            eco: String::from(""),
            opening_name: String::from(""),
            time_control: String::from(""),
            initial_time: 0,
            increment: 0,
            termination: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct GameMoves {
    game_id: String,
    half_move: usize,
    fen: String,
    clock: String,
    eval: String,
}

impl GameMoves {
    fn new() -> GameMoves {
        GameMoves {
            game_id: String::from(""),
            half_move: 0,
            fen: String::from(""),
            clock: String::from(""),
            eval: String::from(""),
        }
    }
}

struct PgnExtractor {
    moves: usize,
    games: usize,
    half_moves: usize,
    header_file: Writer<File>,
    moves_file: Writer<File>,
    game_headers: GameHeaders,
    game_moves: GameMoves,
}

impl PgnExtractor {
    fn new() -> PgnExtractor {
        PgnExtractor {
            moves: 0,
            games: 0,
            half_moves: 0,
            header_file: Writer::from_path("header_results.csv").unwrap(),
            moves_file: Writer::from_path("moves_results.csv").unwrap(),
            game_headers: GameHeaders::new(),
            game_moves: GameMoves::new(),
        }
    }
}

impl Visitor for PgnExtractor {
    type Result = usize;

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        // agg headers
    }

    fn end_headers(&mut self) -> Skip {
        // write to file
        self.header_file.serialize(&self.game_headers).unwrap();
        self.game_headers = GameHeaders::new();
        Skip(false)
    }

    fn comment(&mut self, comment: RawComment<'_>) {
        // parse comment
    }

    fn begin_game(&mut self) {
        self.games += 1;
        self.half_moves = 0;
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

    // TODO: write csv headers before starting extraction

    let mut counter = PgnExtractor::new();
    reader.read_all(&mut counter)?;

    println!("{:?}", counter.moves);
    Ok(())
}