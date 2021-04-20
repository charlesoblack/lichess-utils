extern crate csv;

use std::io;
use std::str;
use std::fs::File;
use std::string::String;

use csv::Writer;
use serde::Serialize;
use regex::Regex;

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
    clock: String,
    eval: String,
}

impl GameMoves {
    fn new() -> GameMoves {
        GameMoves {
            game_id: String::from(""),
            half_move: 0,
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
        let parsed_key = str::from_utf8(key).unwrap();
        let parsed_value = str::from_utf8(value.as_bytes())
                                .unwrap()
                                .to_string();

        let mut pointless = String::new();

        *match parsed_key {
            "Event" => &mut self.game_headers.event,
            "Site" => &mut self.game_headers.game_link,
            "White" => &mut self.game_headers.white_player,
            "Black" => &mut self.game_headers.black_player,
            "Result" => &mut self.game_headers.result,
            "UTCDate" => &mut self.game_headers.date_played,
            "UTCTime" => &mut self.game_headers.time_played,
            "WhiteElo" => &mut self.game_headers.white_elo,
            "BlackElo" => &mut self.game_headers.black_elo,
            "WhiteRatingDiff" => &mut self.game_headers.white_rating_diff,
            "BlackRatingDiff" => &mut self.game_headers.black_rating_diff,
            "ECO" => &mut self.game_headers.eco,
            "Opening" => &mut self.game_headers.opening_name,
            "TimeControl" => {let cloned_value = parsed_value.clone();
                              let time: Vec<&str> = cloned_value.split('+').collect();
                              self.game_headers.initial_time = time[0].parse().unwrap();
                              self.game_headers.increment = time[1].parse().unwrap();
                              &mut self.game_headers.time_control}, // special case
            "Termination" => &mut self.game_headers.termination,
            &_ => &mut pointless,
        } = parsed_value.clone();
    }

    fn end_headers(&mut self) -> Skip {
        // write to file
        self.header_file.serialize(&self.game_headers).unwrap();
        self.game_moves.game_id = self.game_headers.game_link.clone();

        self.game_headers = GameHeaders::new();
        Skip(false)
    }

    fn comment(&mut self, comment: RawComment<'_>) {
        // parse comment
        let parsed_comment = str::from_utf8(comment.as_bytes()).unwrap();
        let re = Regex::new(r"\[%eval ([^\]]+)").unwrap();
        let captures = re.captures(parsed_comment).unwrap();
        self.game_moves.eval = captures.get(1).unwrap().as_str().to_string();

        let re = Regex::new(r"\[%clk ([^\]]+)").unwrap();
        let captures = re.captures(parsed_comment).unwrap();
        self.game_moves.clock = captures.get(1).unwrap().as_str().to_string();

        self.game_moves.half_move = self.half_moves;

        self.moves_file.serialize(&self.game_moves).unwrap();
    }

    fn begin_game(&mut self) {
        self.games += 1;
        self.half_moves = 0;
    }

    fn san(&mut self, _san_plus: SanPlus) {
        self.moves += 1;
        self.half_moves += 1;
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

    let mut parser = PgnExtractor::new();
    reader.read_all(&mut parser)?;

    println!("Games parsed: {:?}", parser.games);
    println!("Moves parsed: {:?}", parser.moves);
    Ok(())
}
