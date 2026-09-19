User Guide:


The most important features that you will probably use will be in the board and int_move class


Int_move is a class that stores a move as a start coordinate end coordinate and promotion piece.


With an intmove you can perform moves on the chess board with make_and_validate_move.


You can also get all the valid intmoves from the field Board.valid_moves.


To get a nice representation of the board you may use Board.get_int_board and that will return a 2d array of ints representing the pieces
