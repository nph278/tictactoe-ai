use std::io::stdout;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};

const ALL_POSITIONS: [(usize, usize); 9] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (1, 0),
    (1, 1),
    (1, 2),
    (2, 0),
    (2, 1),
    (2, 2),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameResult {
    X,
    O,
    Tie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiMode {
    Win,
    Lose,
    Tie,
    NoTie,
}

impl From<Player> for GameResult {
    fn from(p: Player) -> GameResult {
        match p {
            Player::X => GameResult::X,
            Player::O => GameResult::O,
        }
    }
}

impl Player {
    fn display(self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }

    fn display2(x: Option<Player>) -> &'static str {
        x.map_or(" ", Player::display)
    }

    fn opposite(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Game([[Option<Player>; 3]; 3], Player);

impl Game {
    fn new() -> Game {
        Game([[None; 3]; 3], Player::X)
    }

    fn render(&self) {
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        println!("   a b c");
        execute!(stdout(), MoveTo(0, 1)).unwrap();
        println!(
            "1 |{}|{}|{}|",
            Player::display2(self.0[0][0]),
            Player::display2(self.0[0][1]),
            Player::display2(self.0[0][2])
        );
        execute!(stdout(), MoveTo(0, 2)).unwrap();
        println!(
            "2 |{}|{}|{}|",
            Player::display2(self.0[1][0]),
            Player::display2(self.0[1][1]),
            Player::display2(self.0[1][2])
        );
        execute!(stdout(), MoveTo(0, 3)).unwrap();
        println!(
            "3 |{}|{}|{}|",
            Player::display2(self.0[2][0]),
            Player::display2(self.0[2][1]),
            Player::display2(self.0[2][2])
        );
    }

    fn winner(self) -> Option<GameResult> {
        if self.0[0][0].is_some() && self.0[0][0] == self.0[0][1] && self.0[0][0] == self.0[0][2] {
            Some(self.0[0][0].unwrap().into())
        } else if self.0[1][0].is_some()
            && self.0[1][0] == self.0[1][1]
            && self.0[1][0] == self.0[1][2]
        {
            Some(self.0[1][0].unwrap().into())
        } else if self.0[2][0].is_some()
            && self.0[2][0] == self.0[2][1]
            && self.0[2][0] == self.0[2][2]
        {
            Some(self.0[2][0].unwrap().into())
        } else if self.0[0][0].is_some()
            && self.0[0][0] == self.0[1][0]
            && self.0[0][0] == self.0[2][0]
        {
            Some(self.0[0][0].unwrap().into())
        } else if self.0[0][1].is_some()
            && self.0[0][1] == self.0[1][1]
            && self.0[0][1] == self.0[2][1]
        {
            Some(self.0[0][1].unwrap().into())
        } else if self.0[0][2].is_some()
            && self.0[0][2] == self.0[1][2]
            && self.0[0][2] == self.0[2][2]
        {
            Some(self.0[0][2].unwrap().into())
        } else if self.0[0][0].is_some()
            && self.0[0][0] == self.0[1][1]
            && self.0[0][0] == self.0[2][2]
        {
            Some(self.0[0][0].unwrap().into())
        } else if self.0[0][2].is_some()
            && self.0[0][2] == self.0[1][1]
            && self.0[0][2] == self.0[2][0]
        {
            Some(self.0[0][2].unwrap().into())
        } else if self.0.iter().any(|x| x.iter().any(|x| x.is_none())) {
            None
        } else {
            Some(GameResult::Tie)
        }
    }

    fn can_move(self, (i, j): (usize, usize)) -> bool {
        self.0[i][j].is_none()
    }

    fn force_move(&mut self, (i, j): (usize, usize)) {
        self.0[i][j] = Some(self.1);
        self.1 = self.1.opposite();
    }

    fn try_move(&mut self, (i, j): (usize, usize)) -> bool {
        if self.can_move((i, j)) {
            self.force_move((i, j));
            true
        } else {
            false
        }
    }

    fn win_probability(self, p: Player, m: AiMode) -> f32 {
        if let Some(w) = self.winner() {
            if w == p.into() {
                if m == AiMode::Win || m == AiMode::NoTie {
                    1.0
                } else {
                    0.0
                }
            } else if w == GameResult::Tie {
                if m == AiMode::Tie {
                    1.0
                } else if m == AiMode::NoTie {
                    0.0
                } else {
                    0.5
                }
            } else {
                if m == AiMode::Lose || m == AiMode::NoTie {
                    1.0
                } else {
                    0.0
                }
            }
        } else if self.1 == p {
            self.optimal_move(p, m).1
        } else {
            let ps: Vec<f32> = ALL_POSITIONS
                .into_iter()
                .filter(|x| self.can_move(*x))
                .map(|x| {
                    let mut game = self.clone();
                    game.force_move(x);
                    game.win_probability(p, m)
                })
                .collect();
            ps.iter().sum::<f32>() / ps.len() as f32
        }
    }

    fn optimal_move(self, p: Player, m: AiMode) -> ((usize, usize), f32) {
        ALL_POSITIONS
            .into_iter()
            .filter(|x| self.can_move(*x))
            .map(|x| {
                let mut game = self.clone();
                game.force_move(x);
                (x, game.win_probability(p, m))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap()
    }

    fn step(&mut self, x_ai: bool, o_ai: bool, m: AiMode) -> bool {
        let ai = match self.1 {
            Player::X => x_ai,
            Player::O => o_ai,
        };

        let space = if ai {
            self.optimal_move(self.1, m).0
        } else {
            let mut i = None;
            let mut j = None;
            loop {
                match read().unwrap() {
                    Event::Key(KeyEvent { code, .. }) => match code {
                        KeyCode::Char('q') | KeyCode::Esc => return false,
                        KeyCode::Char('1') => i = Some(0),
                        KeyCode::Char('2') => i = Some(1),
                        KeyCode::Char('3') => i = Some(2),
                        KeyCode::Char('a') => j = Some(0),
                        KeyCode::Char('b') => j = Some(1),
                        KeyCode::Char('c') => j = Some(2),
                        _ => {}
                    },
                    Event::Mouse(MouseEvent {
                        kind: MouseEventKind::Up(_),
                        column,
                        row,
                        ..
                    }) => {
                        let i = row - 1;
                        let j = (column - 2) / 2;
                        if i < 3 && j < 3 {
                            break (i as usize, j as usize);
                        }
                    }
                    _ => {}
                };
                if let Some(i) = i {
                    if let Some(j) = j {
                        break (i, j);
                    }
                }
            }
        };

        self.try_move(space);

        true
    }

    fn play(&mut self, x_ai: bool, o_ai: bool, m: AiMode) -> Option<GameResult> {
        self.render();
        while self.step(x_ai, o_ai, m) {
            self.render();
            if let Some(p) = self.winner() {
                return Some(p);
            }
        }
        None
    }
}

fn main() {
    let mut x_ai = false;
    let mut o_ai = false;
    let mut m = AiMode::Win;

    let mut args = std::env::args().skip(1);
    if let Some(a) = args.next() {
        if a == "1" {
            x_ai = true;
        }
        if let Some(a) = args.next() {
            if a == "1" {
                o_ai = true;
            }
            if let Some(a) = args.next() {
                if a == "l" {
                    m = AiMode::Lose;
                } else if a == "t" {
                    m = AiMode::Tie;
                } else if a == "n" {
                    m = AiMode::NoTie;
                }
            }
        }
    }

    enable_raw_mode().unwrap();
    execute!(stdout(), Hide).unwrap();
    execute!(stdout(), EnableMouseCapture).unwrap();
    execute!(stdout(), Clear(ClearType::All)).unwrap();

    if let Some(x) = Game::new().play(x_ai, o_ai, m) {
        execute!(stdout(), MoveTo(0, 4)).unwrap();
        match x {
            GameResult::X => println!("X wins"),
            GameResult::O => println!("O wins"),
            GameResult::Tie => println!("Tie"),
        }
    };

    execute!(stdout(), MoveTo(0, 5)).unwrap();
    execute!(stdout(), DisableMouseCapture).unwrap();
    execute!(stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
}
