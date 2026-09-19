use crate::{board::Board, piece::Piece,utils::Colour,utils::CastleRights};

//BitMove is used to make a move inside the Board struct. Should ignore for gui.
pub struct BitMove {
    pub white_flips: [u64; 7],
    pub black_flips: [u64; 7],
    pub enpasant_flip: u64,
    pub castle_flip: u8,
}

impl BitMove {
    #[allow(dead_code)]
    pub fn new() -> BitMove{
        return BitMove { white_flips:[0;7], black_flips: [0;7], enpasant_flip: 0,castle_flip: 0};
    }
    pub fn from_int_move(int_move: IntMove,board:&Board,colour:&Colour)->Result<BitMove,String>{
        assert!(Board::is_inside_board(int_move.ex, int_move.ey));
        let board_start_i = Board::getpos(int_move.sx,int_move.sy);
        let board_end_i = Board::getpos(int_move.ex,int_move.ey);
        let movepiece = board.get_piece(board_start_i, &colour);
        let capturepiece = board.get_piece(board_end_i, &colour.opposite());
        let movei = match movepiece{
            Some(val) => Piece::get_index(val),
            None => {
                return Err(String::from("No piece at starting position"))
            },
        };


        let mut friendlyarr = [0;7];
        let mut enemyarr = [0;7];
        friendlyarr[6] |= 1u64 << board_start_i;
        friendlyarr[movei] |= 1u64 << board_start_i;
        friendlyarr[6] |= 1u64 << board_end_i;


        match int_move.promotion_piece{
            None => friendlyarr[movei] |= 1u64 << board_end_i,
            Some(pc) => friendlyarr[Piece::get_index(pc)] |= 1u64 << board_end_i,
        }
        
        match capturepiece{
            Some(val) => {
                enemyarr[6] |= 1u64 << board_end_i;
                enemyarr[Piece::get_index(val)] |= 1u64 << board_end_i;
            },
            None => {},
        };

        let mut enpasant_flips = board.enpasant;
        if movepiece == Some(Piece::Pawn){
            if (int_move.sy - int_move.ey).abs() == 2{
                let cy = (int_move.sy + int_move.ey)/2;
                enpasant_flips ^= 1u64 << Board::getpos(int_move.sx, cy);
            }
            if board.can_enpasant(board_end_i){
                enemyarr[Piece::get_index(Piece::Pawn)] ^= 1u64 << Board::getpos(int_move.ex, int_move.sy);
                enemyarr[6] ^= 1u64 << Board::getpos(int_move.ex, int_move.sy);
            }
        }

        if movepiece == Some(Piece::King){
            if int_move.sx - int_move.ex == 2{
                let rooki = Piece::get_index(Piece::Rook);
                friendlyarr[rooki] ^= Board::getbit(0, int_move.sy);
                friendlyarr[6]^= Board::getbit(0, int_move.sy);
                friendlyarr[rooki] ^= Board::getbit(3, int_move.sy);
                friendlyarr[6] ^= Board::getbit(3, int_move.sy);

            }
            if int_move.sx - int_move.ex == -2{
                let rooki = Piece::get_index(Piece::Rook);
                friendlyarr[rooki] ^= Board::getbit(7, int_move.sy);
                friendlyarr[rooki] ^= Board::getbit(5, int_move.sy);
                friendlyarr[6] ^= Board::getbit(7, int_move.sy);
                friendlyarr[6] ^= Board::getbit(5, int_move.sy);
            }
        }
        //looks if a move ends or starts at the same place a starting rook would be and updates castling rights
        let mut castle_flips = 0u8;
        if (board_start_i == Board::getpos(0, 0)) || (board_end_i == Board::getpos(0, 0)){
            castle_flips |= board.castle_rights & CastleRights::getbit(&CastleRights::WQ);
        }
        if (board_start_i == Board::getpos(7, 0)) || (board_end_i == Board::getpos(7, 0)){
            castle_flips |= board.castle_rights & CastleRights::getbit(&CastleRights::WK);
        }
        if (board_start_i == Board::getpos(0, 7)) || (board_end_i == Board::getpos(0, 7)){
            castle_flips |= board.castle_rights & CastleRights::getbit(&CastleRights::BQ);
        }
        if (board_start_i == Board::getpos(7, 7)) || (board_end_i == Board::getpos(7, 7)){
            castle_flips |= board.castle_rights & CastleRights::getbit(&CastleRights::BK);
        }
        if (board_start_i == Board::getpos(4, 0)) || (board_end_i == Board::getpos(4, 0)){
            castle_flips |= board.castle_rights & 3;
        }
        if (board_start_i == Board::getpos(4, 7)) || (board_end_i == Board::getpos(4, 7)){
            castle_flips |= board.castle_rights & 12;
        }


        match colour{
            Colour::White => Ok(BitMove { white_flips: friendlyarr, black_flips: enemyarr,enpasant_flip:enpasant_flips,castle_flip: castle_flips}),
            Colour::Black => Ok(BitMove { white_flips: enemyarr, black_flips: friendlyarr, enpasant_flip:enpasant_flips,castle_flip: castle_flips}),
        }
    }
    #[allow(dead_code)]
    pub fn from_string(inp:&str,board:&Board,colour:&Colour)->Result<BitMove,String>{
        let int_move = match IntMove::parse(inp){
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        return BitMove::from_int_move(int_move, board, colour); 
    }
}

//IntMove is a move stored as a start and end coordinate and a promotion piece. It is used as a more userfriendly format then bitmoves.
#[derive(PartialEq,Copy,Clone)]
pub struct IntMove {
    pub sx: i8,
    pub sy: i8,
    pub ex: i8,
    pub ey: i8,
    pub promotion_piece: Option<Piece>,
}
impl IntMove {
    //Parses user input into an intmove. The user input should be of the format e2 e4 if it is a normal move
    //if it is a promotion the format should be e7 e8 q. where q can be replaced by the character for the piece.
    pub fn parse(input: &str) -> Result<IntMove, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let sx: i8;
        let sy: i8;
        let ex: i8;
        let ey: i8;
        if parts.len() < 2 {
            return Err(format!("'{input}' has too few arguments"));
        }
        if parts.len() > 3{
            return Err(format!("'{input}' has too many arguments"));
        }
        (sx, sy) = match parse_cords(parts[0]) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        (ex, ey) = match parse_cords(parts[1]) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        let piece:Option<Piece>;
        if parts.len() == 3{
                piece = match Piece::get_type(&parts[2].chars().next().unwrap()){
                Ok(val) => Some(val),
                Err(e) => return Err(e),
            };
        }
        else{
            piece = None;
        }
        return Ok(IntMove{sx:sx,sy:sy,ex:ex,ey:ey,promotion_piece:piece});
    }
}
//turns chess cordinates ex e4 into board coordinates (4, 3) it will return (x,y) 
pub fn parse_cords(inp: &str) -> Result<(i8, i8), String> {
    if inp.len() != 2 {
        return Err(format!(
            "length of {inp} is not 2 unable to parse coordinate"
        ));
    }
    let bytes = inp.as_bytes();
    let x: i8 = match bytes[0] {
        b'a'..=b'h' => (bytes[0] - b'a') as i8,
        _ => {
            return Err(format!(
                "in {inp} first character {} is not in range a-h",
                bytes[0]
            ));
        }
    };
    let y: i8 = match bytes[1] {
        b'1'..=b'8' => (bytes[1] - b'1') as i8,
        _ => {
            return Err(format!(
                "in {inp} second character {} is not in range 1-8",
                bytes[1]
            ));
        }
    };
    return Ok((x, y));
}


