pub mod board;
pub use board::Board;
pub use chessmove::IntMove;
pub use utils::Colour;
pub mod piece;
pub mod utils;
pub mod chessmove;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn working_king_castle(){
        let mut board = Board::from_fen("rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq -").unwrap();
        board.make_and_validate_move(IntMove::parse("e1 g1").unwrap()).unwrap();
        board.make_and_validate_move(IntMove::parse("e8 g8").unwrap()).unwrap();
    }
    #[test]
    fn working_queen_castle(){
        let mut board = Board::from_fen("r3kbnr/ppp1pppp/2nq4/3p1b2/3P1B2/2NQ4/PPP1PPPP/R3KBNR w KQkq -").unwrap();
        board.make_and_validate_move(IntMove::parse("e1 c1").unwrap()).unwrap();
        board.make_and_validate_move(IntMove::parse("e8 c8").unwrap()).unwrap();
        
    }
    #[test]
    fn blocked_queen_castle(){
        let mut board = Board::from_fen("r3kbnr/ppp1pppp/2n5/3p1Q2/3P1q2/2N5/PPP1PPPP/R3KBNR w KQkq -").unwrap();
        match board.make_and_validate_move(IntMove::parse("e1 c1").unwrap()){
            Err(_) => {},
            Ok(()) => panic!("Invalid castle was allowed"),
        }
        board.make_and_validate_move(IntMove::parse("g1 f3").unwrap()).unwrap();
        match board.make_and_validate_move(IntMove::parse("e8 c8").unwrap()){
            Err(_) => {},
            Ok(()) => panic!("Invalid castle was allowed"),
        }
        board.make_and_validate_move(IntMove::parse("f4 d4").unwrap()).unwrap(); 
        match board.make_and_validate_move(IntMove::parse("e1 c1").unwrap()){
            Err(_) => {},
            Ok(()) => panic!("Invalid castle was allowed"),
        }   
    }
    #[test]
    fn enpasant(){
        let mut board = Board::from_fen("rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq -").unwrap();
        board.make_and_validate_move(IntMove::parse("e4 e5").unwrap()).unwrap();
        board.make_and_validate_move(IntMove::parse("d7 d5").unwrap()).unwrap();
        board.make_and_validate_move(IntMove::parse("e5 d6").unwrap()).unwrap();
    }
    #[test]
    fn anti_castle_position(){
        let mut board = Board::from_fen("r3k2r/8/8/8/8/4n3/8/R3K2R w KQkq -").unwrap();
        assert_eq!(board.valid_moves.len(), 22);
        board.make_and_validate_move(IntMove::parse("h1 h8").unwrap()).unwrap();
        assert_eq!(board.valid_moves.len(),3);
    }
    #[test]
    fn init_position(){
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -").unwrap();
        assert_eq!(board.valid_moves.len(),20);
    }
    #[test]
    fn promotion(){
        let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQkq -").unwrap();
        board.make_and_validate_move(IntMove::parse("d7 c8 n").unwrap()).unwrap();
    }
    #[test]
    fn perfstarting(){
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -").unwrap();
        assert_eq!(board.perft(1),20);
        assert_eq!(board.perft(2),400);
        assert_eq!(board.perft(3),8902);
    }
    #[test]
    fn perf1(){
        let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap();
        assert_eq!(board.valid_moves.len(),48);
        assert_eq!(board.perft(2),2039);
        assert_eq!(board.perft(3), 97862);
        
    }
    #[test]
    fn perf2(){
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();
        assert_eq!(board.valid_moves.len(),14);
        assert_eq!(board.perft(2),191);
        assert_eq!(board.perft(3), 2812);
        assert_eq!(board.perft(4),43238);
    }
    #[test]
    fn perf3(){
        let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq -").unwrap();
        assert_eq!(board.valid_moves.len(),6);
        assert_eq!(board.perft(2),264);
        assert_eq!(board.perft(3), 9467);
        board = Board::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ -").unwrap();
        assert_eq!(board.valid_moves.len(),6);
        assert_eq!(board.perft(2),264);
        assert_eq!(board.perft(3), 9467);
    }
    #[test]
    fn perf4(){
        let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -").unwrap();
        assert_eq!(board.valid_moves.len(),44);
        assert_eq!(board.perft(2),1486);
        assert_eq!(board.perft(3), 62379);
    }
    #[test]
    fn perf5(){
        let mut board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - -    ").unwrap();
        assert_eq!(board.valid_moves.len(),46);
        assert_eq!(board.perft(2),2079);
        assert_eq!(board.perft(3), 89890);
    }
}
