use chesslib::Board;
use chesslib::IntMove;

fn main() {
    let mut board = match Board::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ"){
        Ok(val) => val,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    println!("{}",board.perft(3));
    //println!("{}",board.perft(4));
    loop{
        //board.print_bitmasks();
        board.print_board();
        if board.is_checkmate(){
            println!("CHECKMATE!");
            break;
        }
        if board.is_stalemate(){
            println!("DRAW :(");
            break;
        }
        let mut inp:String = String::new();
        std::io::stdin().read_line(&mut inp).unwrap();
        let int_move = match IntMove::parse(&inp){
            Ok(val) => val,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        match board.make_and_validate_move(int_move){
            Err(e) => println!("{e}"),
            Ok(()) => {},
        }
    }
}
