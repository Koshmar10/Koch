fn translate_fen_for_model(fen: &str) {
    let board_layout = {
        let fen_parts = fen.split(' ').collect::<Vec<&str>>();
        let board = fen_parts[0];
        board
    };
    let board_rows = {
        let board_layout = board_layout.split('/').collect::<Vec<&str>>();
        let mut board_rows = Vec::new();
        let mut i = 1;
        for row in board_layout {
            let mut board_row = vec![format!("{}", 9 - i)];
            for ch in row.chars() {
                if ch.is_digit(10) {
                    board_row.extend(make_empty_cols(ch.to_digit(10).unwrap() as usize));
                } else {
                    board_row.push(format!("{}", ch));
                }
            }
            board_rows.push(board_row.join(" "));
            i += 1;
        }
        board_rows.push("  a b c d e f g h ".into());
        board_rows.join("\n")
    };
    println!("{}", board_rows);
}
fn make_empty_cols(count: usize) -> Vec<String> {
    let white_spaces = vec![".".to_string(); count];
    white_spaces
}

fn main() {
    let fen = "r1bq1rk1/pp3ppp/2n1pn2/2bp4/3P4/2NBPN2/PP3PPP/R1BQ1RK1 b - d3 4 12";
    translate_fen_for_model(fen);
}
