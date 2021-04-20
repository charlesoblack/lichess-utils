extern crate csv;

use std::io;
use std::str;
use std::env;
use std::fs::File;
use std::string::String;

use csv::Writer;
use serde::Serialize;
use regex::Regex;

use pgn_reader::{Visitor, Skip, BufferedReader,
                 RawComment, RawHeader, SanPlus};
use shakmaty::{Chess, fen, Position};

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
struct GameComments {
    game_id: String,
    half_move: usize,
    clock: String,
    eval: String,
}

impl GameComments {
    fn new() -> GameComments {
        GameComments {
            game_id: String::from(""),
            half_move: 0,
            clock: String::from(""),
            eval: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct GamePositions {
    game_id: String,
    half_move: usize,
    fen: String,
}

impl GamePositions {
    fn new() -> GamePositions {
        GamePositions {
            game_id: String::from(""),
            half_move: 0,
            fen: String::from(""),
        }
    }
}

struct PgnExtractor {
    pos: Chess,
    moves: usize,
    games: usize,
    half_moves: usize,
    header_file: Writer<File>,
    positions_file: Writer<File>,
    comments_file: Writer<File>,
    game_headers: GameHeaders,
    game_positions: GamePositions,
    game_comments: GameComments,
}

impl PgnExtractor {
    fn new() -> PgnExtractor {
        PgnExtractor {
            pos: Chess::default(),
            moves: 0,
            games: 0,
            half_moves: 0,
            header_file: Writer::from_path("header_results.csv").unwrap(),
            positions_file: Writer::from_path("position_results.csv").unwrap(),
            comments_file: Writer::from_path("comments_results.csv").unwrap(),
            game_headers: GameHeaders::new(),
            game_positions: GamePositions::new(),
            game_comments: GameComments::new(),
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
        self.game_comments.game_id = self.game_headers.game_link.clone();
        self.game_positions.game_id = self.game_headers.game_link.clone();

        self.game_headers = GameHeaders::new();
        Skip(false)
    }

    fn comment(&mut self, comment: RawComment<'_>) {
        // parse comment
        let parsed_comment = str::from_utf8(comment.as_bytes()).unwrap();
        let re = Regex::new(r"\[%eval ([^\]]+)").unwrap();
        let captures = re.captures(parsed_comment).unwrap();
        self.game_comments.eval = captures.get(1).unwrap().as_str().to_string();

        let re = Regex::new(r"\[%clk ([^\]]+)").unwrap();
        let captures = re.captures(parsed_comment).unwrap();
        self.game_comments.clock = captures.get(1).unwrap().as_str().to_string();

        self.game_comments.half_move = self.half_moves;

        self.comments_file.serialize(&self.game_comments).unwrap();
    }

    fn begin_game(&mut self) {
        self.games += 1;
        self.half_moves = 0;
    }

    fn san(&mut self, san_plus: SanPlus) {
        self.moves += 1;
        self.half_moves += 1;

        // play move
        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            self.pos.play_unchecked(&m);
        };

        self.game_positions.fen = fen::fen(&self.pos);
        self.game_positions.half_move = self.half_moves;

        // write to file
        self.positions_file.serialize(&self.game_positions).unwrap();
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) -> Self::Result {
        self.pos = Chess::default();
        self.moves
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        // one argument passed
        2 => {
            let pgn_file = File::open(args[1].clone())?;
            let mut reader = BufferedReader::new(pgn_file);

            // TODO: write csv headers before starting extraction

            let mut parser = PgnExtractor::new();
            reader.read_all(&mut parser)?;

            println!("Games parsed: {:?}", parser.games);
            println!("Moves parsed: {:?}", parser.moves);
            Ok(())
        }
        _ => {
            //println!("Must pass only single file location as argument!");
            Err(io::Error::new(io::ErrorKind::Other, "Must pass only single file location as argument"))
        },
    }
}
