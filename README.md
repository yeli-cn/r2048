
# rust 2048
A command line game of 2048 implemented in rust.


## Quick Start
Install Rust and Cargo, open the terminal and run:

1. `cargo update`
2. `cargo run`


## Mainly Game Logic
```rust
pub fn run() {
    init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("Welcome to Rust 2048 ~");

    let core = Core::new();
    let mut board = Board::new(4, None, 0);

    while !core.is_game_over(&board) {
        board.generate(1, 1..3);
        info!("{}", board);

        loop {
            info!("Input direction(w,a,s,d): ");
            let mut direction_str = String::new();
            stdin().read_line(&mut direction_str).unwrap();

            let traces = core.shift(
                &mut board,
                match direction_str.trim().to_lowercase().as_str() {
                    "w" => &Direction::Up,
                    "a" => &Direction::Left,
                    "s" => &Direction::Down,
                    "d" => &Direction::Right,
                    _ => {
                        warn!("Invalid input!");
                        continue;
                    }
                },
            );
            info!("Traces: {:?}", traces);
            if traces.is_empty() {
                warn!("Invalid moved!");
                continue;
            }
            break;
        }
    }

    info!("{}", board);
}
```
