use crossterm::{cursor, event, queue, style, terminal};
use event::{Event, KeyCode, KeyEvent, KeyEventKind};
use rand::Rng;
use std::collections::VecDeque;
use std::io::{self, stdout, Stdout, Write};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
struct Position {
	x: i32,
	y: i32,
}

enum Direction {
	None,
	Right,
	Left,
	Up,
	Down,
}

fn random_position(window_size: &terminal::WindowSize) -> Position {
	let mut rng: rand::rngs::ThreadRng = rand::thread_rng();
	return Position {
		x: rng.gen_range(0..window_size.columns as i32),
		y: rng.gen_range(0..window_size.rows as i32),
	};
}

fn move_wrap(position: &mut Position, window_size: &terminal::WindowSize, (x, y): (i32, i32)) {
	position.x = (position.x + x + window_size.columns as i32) % (window_size.columns as i32);
	position.y = (position.y + y + window_size.rows as i32) % (window_size.rows as i32);
}

// fn wrap(position: &mut Position, window_size: &terminal::WindowSize) {
//	 move_wrap(position, window_size, (0, 0));
// }

fn main() -> io::Result<()> {
	let mut window_size: terminal::WindowSize = terminal::window_size()?;
	let mut args = std::env::args().skip(1).map(|arg: String| arg.parse().expect("Failed to parse integer argument"));
	match [args.next(), args.next()] {
		[None, None] => (),
		[Some(x), Some(y)] => window_size = terminal::WindowSize {
			columns: x,
			rows: y,
			height: x,
			width: y,
		},
		_ => panic!("Missing second argument"),
	}
	assert!(args.next() == None, "Too many arguments");
	window_size.columns /= 2;
	let window_size: terminal::WindowSize = window_size;

	let mut apple: Position = random_position(&window_size);
	let mut body: VecDeque<Position> = VecDeque::from_iter([random_position(&window_size)]);
	let mut direction: Direction = Direction::None;
	let mut score: u32 = 0;

	terminal::enable_raw_mode()?;
	let mut stdout: Stdout = stdout();
	queue!(stdout, cursor::Hide, terminal::EnterAlternateScreen)?;
	stdout.flush()?;

	'game: loop {
		use Direction::*;

		let mut head: Position = body[0];
		let (x, y) = match direction {
			None => (0, 0),
			Right => (1, 0),
			Left => (-1, 0),
			Up => (0, -1),
			Down => (0, 1),
		};
		move_wrap(&mut head, &window_size, (x, y));
		// wrap(&mut apple, &window_size);

		if head == apple {
			apple = random_position(&window_size);
			score += 1;
		} else {
			body.pop_back();
		}

		queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
		for &part in &body {
			if head == part {
				break 'game;
			}
			queue!(stdout, cursor::MoveTo((part.x * 2) as u16, part.y as u16), style::SetBackgroundColor(style::Color::Green), style::Print("  "), style::ResetColor)?;
		}
		body.push_front(head);
		queue!(stdout, cursor::MoveTo((head.x * 2) as u16, head.y as u16), style::SetBackgroundColor(style::Color::Blue), style::Print("  "), style::ResetColor, cursor::MoveTo((apple.x * 2) as u16, apple.y as u16), style::SetBackgroundColor(style::Color::Red), style::Print("  "), style::ResetColor)?;
		stdout.flush()?;

		if event::poll(Duration::from_millis(100))? {
			match event::read()? {
				Event::Key(KeyEvent {
					code,
					kind: KeyEventKind::Press,
					..
				}) => {
					use KeyCode::*;
					direction = match code {
						Right => Direction::Right,
						Left => Direction::Left,
						Up => Direction::Up,
						Down => Direction::Down,
						Esc | Char('q') => break 'game,
						_ => direction,
					};
				}
				// Event::Resize(x, y) => {
				//	 window_size = terminal::WindowSize {
				//		 columns: x / 2,
				//		 rows: y,
				//		 width: x,
				//		 height: y,
				//	 };
				//	 body.iter_mut().for_each(|part| wrap(part, &window_size));
				// }
				_ => (),
			}
		}
	}

	let text: String = format!("Score: {}", score);
	queue!(stdout, cursor::MoveTo(window_size.columns.checked_sub((text.len() / 2) as u16).unwrap_or(0), (window_size.rows / 2) as u16), style::Print(text))?;
	stdout.flush()?;
	thread::sleep(Duration::from_secs(1));
	terminal::disable_raw_mode()?;
	queue!(stdout, cursor::Show, terminal::Clear(terminal::ClearType::All), terminal::LeaveAlternateScreen)?;
	stdout.flush()?;

	return Ok(());
}
