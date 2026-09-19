
#[derive(Copy,Clone,PartialEq)]
pub enum Colour {
    White,
    Black,
}
impl Colour{
    pub fn opposite(&self) -> Colour{
        match self{
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}
pub enum CastleRights{
    WQ,
    WK,
    BQ,
    BK,
}
impl CastleRights{
    pub fn getbit(castle_right: &CastleRights)->u8{
        match castle_right{
            CastleRights::WQ => 1,
            CastleRights::WK => 2,
            CastleRights::BQ => 4,
            CastleRights::BK => 8,
        }
    }
}