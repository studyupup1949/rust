use std::io;


pub fn x_o_game() {
    let mut board = vec![' '; 9];
    let mut current_player = 'X';

    loop {
        println!(
            "\n{} | {} | {}\n--|---|--\n{} | {} | {}\n--|---|--\n{} | {} | {}",
            board[0], board[1], board[2], board[3], board[4], board[5], board[6], board[7], board[8]
        );

        println!("Player {}, enter your move (1-9):", current_player);
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Ok(pos) = input.trim().parse::<usize>() {
            if pos >= 1 && pos <= 9 && board[pos - 1] == ' ' {
                board[pos - 1] = current_player;

                if check_winner(&board, current_player) {
                    println!("Player {} wins!", current_player);
                    break;
                }

                if !board.contains(&' ') {
                    println!("It's a draw!");
                    break;
                }

                current_player = if current_player == 'X' { 'O' } else { 'X' };
            } else {
                println!("Invalid move. Try again.");
            }
        } else {
            println!("Invalid input. Enter a number between 1-9.");
        }
    }
}

fn check_winner(board: &[char], player: char) -> bool {
    let win_patterns = [
        (0, 1, 2),
        (3, 4, 5),
        (6, 7, 8),
        (0, 3, 6),
        (1, 4, 7),
        (2, 5, 8),
        (0, 4, 8),
        (2, 4, 6),
    ];
    win_patterns.iter().any(|&(a, b, c)| {
        board[a] == player && board[b] == player && board[c] == player
    })
}

