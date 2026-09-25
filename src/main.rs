use chesslib::Board;
use chesslib::IntMove;
use chesslib::piece::Piece;
use ggez::*;
use rand::Rng;

// ToDo: Possibly add support for different screen resolutions
static TOP_LEFT: [f32; 2] = [448.0, 28.0];
static GAME_SCALE: f32 = 4.0;
static PROMOTION_MARGIN: f32 = 30.0;

static BELL_SIZE: f32 = 4.0;
static BELL_TOP: f32 = 480.0;
static BELL_LEFT: f32 = 88.0;
static BELL_RIGHT: f32 = 1920.0 - BELL_LEFT - (64.0*BELL_SIZE);

struct Ember {
    x: f32,
    y: f32,
    brightness: f32,
    color: graphics::Color,
}

struct State {
    dt: std::time::Duration,
    time: f32,
    board: chesslib::Board,
    stage: i8,
    start: [i8; 2],
    promoting_move: Option<IntMove>,
    game_state: i8,
    image_background: graphics::Image,
    image_pieces: graphics::Image,
    image_bell: graphics::Image,
    image_hints: graphics::Image,
    image_promo: graphics::Image,
    image_eyes: graphics::Image,
    image_ember: graphics::Image,
    white_bell_frame: f32,
    black_bell_frame: f32,
    fog_mask: Vec<u8>,
    embers: Vec<Ember>,
    eye_brightness: u8,
}

pub fn upscale_image(ctx: &mut Context, image: &graphics::Image, scale: usize) -> GameResult<graphics::Image> {
    let width = image.width() as usize;
    let height = image.height() as usize;

    let original = image.to_pixels(ctx)?;

    let new_width = width * scale;
    let new_height = height * scale;

    let mut pixels = vec![0u8; new_width * new_height * 4];

    for y in 0..height {
        for x in 0..width {
            let src = (y * width + x) * 4;

            for dy in 0..scale {
                for dx in 0..scale {
                    let nx = x * scale + dx;
                    let ny = y * scale + dy;

                    let dst = (ny * new_width + nx) * 4;

                    pixels[dst..dst + 4]
                        .copy_from_slice(&original[src..src + 4]);
                }
            }
        }
    }

    Ok(graphics::Image::from_pixels(
        ctx,
        &pixels,
        graphics::ImageFormat::Rgba8UnormSrgb,
        new_width as u32,
        new_height as u32,
    ))
}

pub fn add_glow(ctx: &mut Context, image: &graphics::Image, radius: i32, strength: f32) -> GameResult<graphics::Image> {
    let width = image.width() as usize;
    let height = image.height() as usize;

    let original = image.to_pixels(ctx)?;
    let mut result = original.clone();

    for y in 0..height {
        for x in 0..width {
            let mut glow_r = 0.0;
            let mut glow_g = 0.0;
            let mut glow_b = 0.0;
            let mut glow_a = 0.0;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx < 0 || nx >= width as i32 ||
                       ny < 0 || ny >= height as i32 {
                        continue;
                    }

                    let distance = ((dx * dx + dy * dy) as f32).sqrt();

                    if distance > radius as f32 {
                        continue;
                    }

                    let i = (ny as usize * width + nx as usize) * 4;

                    let alpha = original[i + 3] as f32 / 255.0;

                    // Fade with distance
                    let falloff = 1.0 - distance / radius as f32;

                    let contribution = alpha * falloff * strength;

                    glow_r += original[i] as f32 * contribution;
                    glow_g += original[i + 1] as f32 * contribution;
                    glow_b += original[i + 2] as f32 * contribution;
                    glow_a += contribution;
                }
            }

            let i = (y * width + x) * 4;

            if original[i + 3] == 0 {
                result[i] = glow_r.clamp(0.0, 255.0) as u8;
                result[i + 1] = glow_g.clamp(0.0, 255.0) as u8;
                result[i + 2] = glow_b.clamp(0.0, 255.0) as u8;
                result[i + 3] = (glow_a * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    Ok(graphics::Image::from_pixels(
        ctx,
        &result,
        graphics::ImageFormat::Rgba8UnormSrgb,
        width as u32,
        height as u32,
    ))
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let dt = std::time::Duration::new(0, 0);
        let time = 0.0;
        // let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ -";
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = match Board::from_fen(fen) {
            Ok(val) => val,
            Err(e) => {
                println!("{e}");
                return Err(ggez::GameError::CustomError(e));
            }
        };

        let image_background = graphics::Image::from_path(ctx, "/GrassTmp.png")?;
        let image_pieces = graphics::Image::from_path(ctx, "/chess.png")?;
        let image_bell = graphics::Image::from_path(ctx, "/bell.png")?;
        let image_hints = graphics::Image::from_path(ctx, "/hint.png")?;
        let image_promo = graphics::Image::from_path(ctx, "/promotion.png")?;
        let image_eyes_no_glow = upscale_image(ctx, &graphics::Image::from_path(ctx, "/eyes.png")?, GAME_SCALE as usize)?;
        let image_eyes = add_glow(ctx, &image_eyes_no_glow, 7, 0.04)?;
        let pixels = (0..16 * 16).flat_map(|i| {
                let x = i % 16;
                let y = i / 16;

                if (6..10).contains(&x) && (6..10).contains(&y) {
                    [255, 255, 255, 255]
                } else {
                    [0, 0, 0, 0]
                }
            }).collect::<Vec<u8>>();

        let image_ember_no_glow = graphics::Image::from_pixels(ctx, &pixels, graphics::ImageFormat::Rgba8UnormSrgb, 16, 16);
        let image_ember = add_glow(ctx, &image_ember_no_glow, 7, 0.07)?;

        let stage = match fen.split_whitespace().collect::<Vec<&str>>()[1] {
            "b" => 3,
            "w" => 0,
            _ => return Err(ggez::GameError::CustomError("Bruh".to_string())), 
        };
        let white_bell_frame = 0.0;
        let black_bell_frame = 0.0;
        let start = [-1, -1];
        let promoting_move = None;
        let game_state = 0; // game_states: 0 -> in play, 1 -> White wins, 2 -> draw, 3 -> Black wins
        let fog_mask = vec![255 as u8; 64];
        let embers: Vec<Ember> = vec![];
        let eye_brightness: u8 = 0;

        Ok(State {dt, time, board, stage, start, promoting_move, game_state, image_background, image_pieces, image_bell, image_hints, image_promo, image_eyes, image_ember, white_bell_frame, black_bell_frame, fog_mask, embers, eye_brightness})
    }
}

fn smoothstep(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn hard_fog_pixel(x: u32, y: u32, fog_mask: Vec<u8>) -> (u8, u8, u8, u8) {
    let r = 120u8;
    let g = 120u8;
    let b = 120u8;
    let mut a = 255u8;
    
    let scale_x = ((x as f32) - (TOP_LEFT[0] / GAME_SCALE)) / 32.0 - 0.5;
    let scale_y = ((y as f32) - (TOP_LEFT[1] / GAME_SCALE)) / 32.0 - 0.5;

    if scale_x >= -0.5 && scale_x < 7.5 && scale_y >= -0.5 && scale_y < 7.5 {
        let mut op_tl = 255u8;
        let mut op_tr = 255u8;
        let mut op_bl = 255u8;
        let mut op_br = 255u8;
        if scale_x >= 0.0 && scale_y >= 0.0 {
            op_tl = fog_mask[(7 - (scale_y.floor() as usize)) * 8 + (scale_x.floor() as usize)];
        }
        if scale_x <= 7.0 && scale_y >= 0.0 {
            op_tr = fog_mask[(7 - (scale_y.floor() as usize)) * 8 + (scale_x.ceil() as usize)];
        }
        if scale_x >= 0.0 && scale_y <= 7.0 {
            op_bl = fog_mask[(7 - (scale_y.ceil() as usize)) * 8 + (scale_x.floor() as usize)];
        }
        if scale_x <= 7.0 && scale_y <= 7.0 {
            op_br = fog_mask[(7 - (scale_y.ceil() as usize)) * 8 + (scale_x.ceil() as usize)];
        }

        let dx = smoothstep(scale_x - scale_x.floor());
        let dy = smoothstep(scale_y - scale_y.floor());
        a = ((
            (op_tl as f32) * (1.0 - dx) * (1.0 - dy) +
            (op_tr as f32) * (dx) * (1.0 - dy) +
            (op_bl as f32) * (1.0 - dx) * (dy) +
            (op_br as f32) * (dx) * (dy)) * 2.0)
            .clamp(0.0, 255.0) as u8;
    }
    else {
        let a_dist_x = ((BELL_LEFT / GAME_SCALE) + (64.0/2.0) - 2.0) - (x as f32);
        let dist_y = ((BELL_TOP / GAME_SCALE) + (64.0/2.0) - 6.0) - (y as f32);
        let bell_a_dist = (a_dist_x.powf(2.0) + dist_y.powf(2.0)).sqrt();
        if bell_a_dist < 50.0 {
            a = (((bell_a_dist-20.0)/30.0)*255.0).clamp(0.0, 255.0) as u8;
        }
        else {
            let b_dist_x = ((BELL_RIGHT / GAME_SCALE) + (64.0/2.0) - 2.0) - (x as f32);
            let bell_b_dist = (b_dist_x.powf(2.0) + dist_y.powf(2.0)).sqrt();
            if bell_b_dist < 50.0 {
                a = (((bell_b_dist-20.0)/30.0)*255.0).clamp(0.0, 255.0) as u8;
            }
        }

    }

    (r, g, b, a)
}

fn soft_fog_pixel(x: u32, y: u32, fog_mask: Vec<u8>) -> (u8, u8, u8, u8) {
    let r = 120u8;
    let g = 120u8;
    let b = 120u8;
    let mut a = 255u8;
    
    let scale_x = ((x as f32) - (TOP_LEFT[0] / GAME_SCALE)) / 32.0 - 0.5;
    let scale_y = ((y as f32) - (TOP_LEFT[1] / GAME_SCALE)) / 32.0 - 0.5;

    if scale_x >= -0.5 && scale_x < 7.5 && scale_y >= -0.5 && scale_y < 7.5 {
        let mut op_tl = 255u8;
        let mut op_tr = 255u8;
        let mut op_bl = 255u8;
        let mut op_br = 255u8;
        if scale_x >= 0.0 && scale_y >= 0.0 {
            op_tl = fog_mask[(7 - (scale_y.floor() as usize)) * 8 + (scale_x.floor() as usize)];
        }
        if scale_x <= 7.0 && scale_y >= 0.0 {
            op_tr = fog_mask[(7 - (scale_y.floor() as usize)) * 8 + (scale_x.ceil() as usize)];
        }
        if scale_x >= 0.0 && scale_y <= 7.0 {
            op_bl = fog_mask[(7 - (scale_y.ceil() as usize)) * 8 + (scale_x.floor() as usize)];
        }
        if scale_x <= 7.0 && scale_y <= 7.0 {
            op_br = fog_mask[(7 - (scale_y.ceil() as usize)) * 8 + (scale_x.ceil() as usize)];
        }

        let dx = smoothstep(scale_x - scale_x.floor());
        let dy = smoothstep(scale_y - scale_y.floor());
        a = ((
            (op_tl as f32) * (1.0 - dx) * (1.0 - dy) +
            (op_tr as f32) * (dx) * (1.0 - dy) +
            (op_bl as f32) * (1.0 - dx) * (dy) +
            (op_br as f32) * (dx) * (dy)) * 2.0)
            .clamp(0.0, 255.0) as u8;
    }
    else {
        let a_dist_x = ((BELL_LEFT / GAME_SCALE) + (64.0/2.0) - 2.0) - (x as f32);
        let dist_y = ((BELL_TOP / GAME_SCALE) + (64.0/2.0) - 6.0) - (y as f32);
        let bell_a_dist = (a_dist_x.powf(2.0) + dist_y.powf(2.0)).sqrt();
        if bell_a_dist < 50.0 {
            a = (((bell_a_dist-20.0)/30.0)*255.0).clamp(0.0, 255.0) as u8;
        }
        else {
            let b_dist_x = ((BELL_RIGHT / GAME_SCALE) + (64.0/2.0) - 2.0) - (x as f32);
            let bell_b_dist = (b_dist_x.powf(2.0) + dist_y.powf(2.0)).sqrt();
            if bell_b_dist < 50.0 {
                a = (((bell_b_dist-20.0)/30.0)*255.0).clamp(0.0, 255.0) as u8;
            }
        }
    }
    // Temporary soft fog script that just kind of ignores it
    a = a.clamp(0, 0);

    (r, g, b, a)
}

impl ggez::event::EventHandler for State {
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: input::mouse::MouseButton, x: f32, y: f32) -> GameResult {
        if button == input::mouse::MouseButton::Left {
            // Clicked on board
            if TOP_LEFT[0] < x && x < TOP_LEFT[0] + (256.0*GAME_SCALE) && TOP_LEFT[1] < y && y < TOP_LEFT[1] + (256.0*GAME_SCALE) && (x - TOP_LEFT[0]) % (32.0*GAME_SCALE) > 1.0*GAME_SCALE  && (x - TOP_LEFT[0]) % (32.0*GAME_SCALE) < 31.0*GAME_SCALE  && (y - TOP_LEFT[1]) % (32.0*GAME_SCALE) > 1.0*GAME_SCALE  && (y - TOP_LEFT[1]) % (32.0*GAME_SCALE) < 31.0*GAME_SCALE {
                if self.stage == 1 || self.stage == 4 {
                    let mx = ((x - TOP_LEFT[0]) / (32.0*GAME_SCALE)).floor() as i8;
                    let my = (7.0 - ((y - TOP_LEFT[1]) / (32.0*GAME_SCALE)).floor()) as i8;
                    let mut is_start = false;
                    for m in self.board.valid_moves.clone() {
                        if m.sx == mx && m.sy == my {
                            self.start = [mx, my];
                            is_start = true;
                            break;
                        }
                    }
                    if !is_start {
                        if self.start != [-1, -1] {
                            for mut m in self.board.valid_moves.clone() {
                                if m.ex == mx && m.ey == my && m.sx == self.start[0] && m.sy == self.start[1] {
                                    if (self.board.get_int_board()[m.sy as usize][m.sx as usize] == 0 && m.ey == 7) || (self.board.get_int_board()[m.sy as usize][m.sx as usize] == 6 && m.ey == 0) {
                                        m.promotion_piece = None;
                                        self.promoting_move = Some(m);
                                    }
                                    else {
                                        let _ = self.board.make_and_validate_move(m);
                                    }
                                    self.stage += 1;
                                    if self.board.is_checkmate() {
                                        self.game_state = match self.stage {
                                            5 => 3,
                                            2 => 1,
                                            _ => 0,
                                        }
                                    }
                                    else if self.board.is_stalemate() {
                                        self.game_state = 2
                                    }
                                    break;
                                }
                            }
                            self.start = [-1, -1];
                        }
                    }
                }
            }
            // Clicked on white bell
            else if BELL_LEFT < x - (10.0*BELL_SIZE) && x + (12.0*BELL_SIZE) < BELL_LEFT + (BELL_SIZE * 64.0) && BELL_TOP < y - (11.0*BELL_SIZE) && y + (34.0*BELL_SIZE) < BELL_TOP + (BELL_SIZE * 64.0) {
                if self.stage == 0 {
                    self.stage += 1;
                }
                self.white_bell_frame = 0.0001;
                // ToDo: Play bell sound
            }
            // Clicked on black bell
            else if BELL_RIGHT < x - (10.0*BELL_SIZE) && x + (12.0*BELL_SIZE) < BELL_RIGHT + (BELL_SIZE * 64.0) && BELL_TOP < y - (11.0*BELL_SIZE) && y + (34.0*BELL_SIZE) < BELL_TOP + (BELL_SIZE * 64.0) {
                if self.stage == 3 {
                    self.stage += 1;
                }
                self.black_bell_frame = 0.0001;
                // ToDo: Play bell sound
            }
            // Clicked on promotion selector
            else if ((TOP_LEFT[0] - PROMOTION_MARGIN - (32.0*GAME_SCALE) < x && x < TOP_LEFT[0] - PROMOTION_MARGIN && self.stage == 2) || (TOP_LEFT[0] + PROMOTION_MARGIN + (256.0*GAME_SCALE) < x && x < TOP_LEFT[0] + PROMOTION_MARGIN + (256.0*GAME_SCALE) + (32.0*GAME_SCALE) && self.stage == 5)) && TOP_LEFT[1] < y && y < TOP_LEFT[1] + (128.0*GAME_SCALE) {
                let height = ((y - (TOP_LEFT[1])) / (32.0*GAME_SCALE)).floor();
                if self.promoting_move != None && self.promoting_move.unwrap().promotion_piece == None {
                    self.promoting_move = Some(IntMove {sx: self.promoting_move.unwrap().sx, sy: self.promoting_move.unwrap().sy, ex: self.promoting_move.unwrap().ex, ey: self.promoting_move.unwrap().ey, promotion_piece: match height{
                        1.0 => Some(Piece::Knight),
                        2.0 => Some(Piece::Bishop),
                        3.0 => Some(Piece::Rook),
                        _ => Some(Piece::Queen),
                    }});
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = ctx.time.delta();
        self.time += (self.dt.as_nanos() as f32) / 1000_000.0;
        if self.white_bell_frame != 0.0 {
            self.white_bell_frame += ((self.dt.as_nanos() as f32) / 1000_000_000.0)*18.0;
            if self.white_bell_frame >= 6.0 {
                self.white_bell_frame = 0.0;
            }
        }
        if self.black_bell_frame != 0.0 {
            self.black_bell_frame += ((self.dt.as_nanos() as f32) / 1000_000_000.0)*18.0;
            if self.black_bell_frame >= 6.0 {
                self.black_bell_frame = 0.0;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // println!("dt: {}.{}ms", self.dt.as_nanos() / 1000000, self.dt.as_nanos() / 10000 - (100 * (self.dt.as_nanos() / 1000000)));
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from_rgb(20, 35, 20));
        canvas.set_sampler(graphics::Sampler::nearest_clamp());
        let screen = ctx.gfx.drawable_size();

        // #region Bells, Board, and Pieces layer 1 and fog zones
        // Background
        canvas.draw(
            &self.image_background,
            graphics::DrawParam::new()
                .dest(TOP_LEFT)
                .scale([GAME_SCALE, GAME_SCALE])
        );

        // Bells
        let mut frame = (self.white_bell_frame / 2.0).ceil();
        canvas.draw(
            &self.image_bell,
            graphics::DrawParam::new()
                .src(graphics::Rect::new(
                    0.0 / 128.0,
                    (frame * 64.0) / 256.0,
                    64.0 / 128.0,
                    64.0 / 256.0,
                ))
                .dest([BELL_LEFT, BELL_TOP])
                .scale([BELL_SIZE, BELL_SIZE])
        );

        frame = (self.black_bell_frame / 2.0).ceil();
        canvas.draw(
            &self.image_bell,
            graphics::DrawParam::new()
                .src(graphics::Rect::new(
                    64.0 / 128.0,
                    (frame * 64.0) / 256.0,
                    64.0 / 128.0,
                    64.0 / 256.0,
                ))
                .dest([BELL_RIGHT, BELL_TOP])
                .scale([BELL_SIZE, BELL_SIZE])
        );

        // Pieces Layer 1
        let int_board = self.board.get_int_board();
        for r in (0..8).rev() {
            for c in 0usize..8 {
                if int_board[r][c] == 12 {
                    continue;
                }
                let sheet_coord = match int_board[r][c] % 6 {
                    0 => [0.0,3.0],
                    1 => [3.0,3.0],
                    2 => [1.0,3.0],
                    3 => [2.0,3.0],
                    4 => [4.0,3.0],
                    5 => [5.0,3.0],
                    _ => [0.0,0.0],
                };
                canvas.draw(
                    &self.image_pieces,
                    graphics::DrawParam::new()
                        .src(graphics::Rect::new(
                            (sheet_coord[0] * 32.0) / 192.0,
                            (sheet_coord[1] * 32.0) / 128.0,
                            32.0 / 192.0,
                            32.0 / 128.0,
                        ))
                        .dest([TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)), TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32))])
                        .scale([GAME_SCALE, GAME_SCALE])
                );
            }
        }

        for r in (0..8).rev() {
            for c in 0usize..8 {
                if int_board[r][c] == 12 || int_board[r][c] % 6 == 2 || int_board[r][c] % 6 == 4 {
                    continue;
                }
                if ((self.stage != 4 && int_board[r][c] >= 6) || (self.stage != 1 && int_board[r][c] <= 5)) && self.game_state == 0{
                    continue;
                }
                if int_board[r][c] % 6 == 0 {
                    let p = 1.0 - (28.0_f32 / 30.0).powf((self.dt.as_nanos() as f32 / 1000_000_000.0) * 30.0);
                    if rand::rng().random_bool(p as f64) {
                        self.embers.push(Ember {
                            x: TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)) + 5.0*GAME_SCALE,
                            y: TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32)) + 22.0*GAME_SCALE,
                            color: graphics::Color::new(rand::rng().random::<f32>() * 0.5 + 0.5, rand::rng().random::<f32>() * 0.2 + 0.25, 0.25, 1.0),
                            brightness: 1.0,
                        });
                    }
                }
                else if int_board[r][c] % 6 == 1 {
                    let p = 1.0 - (26.0_f32 / 30.0).powf((self.dt.as_nanos() as f32 / 1000_000_000.0) * 30.0);
                    if rand::rng().random_bool(p as f64) {
                        self.embers.push(Ember {
                            x: TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)) + 12.0*GAME_SCALE,
                            y: TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32)) + 20.0*GAME_SCALE,
                            color: graphics::Color::new(rand::rng().random::<f32>() * 0.5 + 0.5, rand::rng().random::<f32>() * 0.2 + 0.25, 0.25, 1.0),
                            brightness: 1.0,
                        });
                    }
                }
                else if int_board[r][c] % 6 == 3 {
                    let p = 1.0 - (26.0_f32 / 30.0).powf((self.dt.as_nanos() as f32 / 1000_000_000.0) * 30.0);
                    if rand::rng().random_bool(p as f64) {
                        self.embers.push(Ember {
                            x: TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)) + 13.0*GAME_SCALE,
                            y: TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32)) + 20.0*GAME_SCALE,
                            color: graphics::Color::new(rand::rng().random::<f32>() * 0.5 + 0.5, rand::rng().random::<f32>() * 0.2 + 0.25, 0.25, 1.0),
                            brightness: 1.0,
                        });
                    }
                }
                else if int_board[r][c] % 6 == 5 {
                    let p = 1.0 - (24.0_f32 / 30.0).powf((self.dt.as_nanos() as f32 / 1000_000_000.0) * 30.0);
                    if rand::rng().random_bool(p as f64) {
                        let mut c4 = graphics::Color::new(rand::rng().random::<f32>() * 0.4 + 0.6, rand::rng().random::<f32>() * 0.2 + 0.35, 0.35, 1.0);
                        if self.game_state == 1 {
                            if int_board[r][c] == 5 {
                                c4 = graphics::Color::new(rand::rng().random::<f32>() * 0.2 + 0.1, rand::rng().random::<f32>() * 0.2 + 0.75, 0.25, 1.0)
                            }
                            else {
                                c4 = graphics::Color::new(0.0, 0.0, 0.0, 1.0)
                            }
                        }
                        else if self.game_state == 2 {
                            c4 = graphics::Color::new(rand::rng().random::<f32>() * 0.2 + 0.65, rand::rng().random::<f32>() * 0.2 + 0.65, 0.35, 1.0)
                        }
                        else if self.game_state == 3 {
                            if int_board[r][c] == 11 {
                                c4 = graphics::Color::new(rand::rng().random::<f32>() * 0.2 + 0.1, rand::rng().random::<f32>() * 0.2 + 0.75, 0.25, 1.0)
                            }
                            else {
                                c4 = graphics::Color::new(0.0, 0.0, 0.0, 1.0)
                            }
                        }
                        self.embers.push(Ember {
                            x: TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)) + 11.5*GAME_SCALE,
                            y: TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32)) + 14.0*GAME_SCALE,
                            color: c4,
                            brightness: 1.0,
                        });
                    }
                }
            }
        }
        let mut highlights: Vec<i8> = vec![];
        if self.game_state != 0 {
            for i in 0..64usize {
                self.fog_mask[i] = 0.0_f32.max((self.fog_mask[i] as f32) * (0.3_f32.powf((self.dt.as_nanos() as f32) / 1000_000_000.0))).floor() as u8;
            }
        }
        else if self.stage == 1 || self.stage == 4 {
            let mut show = self.board.valid_moves.clone();
            let mut c = chesslib::utils::Colour::White;
            if self.stage == 4 {
                c = chesslib::utils::Colour::Black;
            }
            let b = self.board.get_int_board();
            let mut k_sq = 0i8;
            for r in 0..8usize {
                for c in 0..8usize {
                    if (b[r][c] == 5 && self.stage == 1) || (b[r][c] == 11 && self.stage == 4) {
                        k_sq = (r * 8 + c) as i8;
                    }
                    if (self.stage == 1 && b[r][c] <= 5) || (self.stage == 4 && b[r][c] >= 6 && b[r][c] != 12) {
                        show.push(IntMove {sx: c as i8, sy: r as i8, ex: c as i8, ey: r as i8, promotion_piece: None});
                    }
                }
            }
            let mut dir_y: Vec<i8> = vec![0i8; 1];
            if k_sq / 8 != 0 {
                dir_y.push(-1i8);
            }
            if k_sq / 8 != 7 {
                dir_y.push(1i8);
            }
            let mut dir = vec![[0i8, 0i8]; 0];
            for d in dir_y {
                dir.push([0, d]);
                if k_sq % 8 != 0 {
                    dir.push([-1, d]);
                }
                if k_sq % 8 != 7 {
                    dir.push([1, d]);
                }
            }

            for ds in dir {
                let p = b[(k_sq / 8 + ds[1]) as usize][(k_sq % 8 + ds[0]) as usize];
                if ((p > 5 && self.stage == 1) || ((p < 6 || p == 12) && self.stage == 4)) && !self.board.valid_moves.contains(&IntMove {sx: k_sq % 8, sy: k_sq / 8, ex: k_sq % 8 + ds[0], ey: k_sq / 8 + ds[1], promotion_piece: None}) {
                    show.push(IntMove {sx: k_sq % 8 + ds[0], sy: (k_sq / 8) + ds[1], ex: k_sq % 8 + ds[0], ey: (k_sq / 8) + ds[1], promotion_piece: None});
                    highlights.push(k_sq + (ds[1] * 8) + ds[0]);
                }
                else if self.board.is_attacked(&c, k_sq % 8 + ds[0], (k_sq / 8) + ds[1]) {
                    show.push(IntMove {sx: k_sq % 8 + ds[0], sy: (k_sq / 8) + ds[1], ex: k_sq % 8 + ds[0], ey: (k_sq / 8) + ds[1], promotion_piece: None});
                    highlights.push(k_sq + (ds[1] * 8) + ds[0]);
                }
            }

            if self.board.in_check(&c) {
                let checkers = self.board.get_checker(&c);
                for checker in checkers {
                    show.push(IntMove {sx: checker.0 as i8, sy: checker.1 as i8, ex: checker.0 as i8, ey: checker.1 as i8, promotion_piece: None});
                    highlights.push((checker.1 * 8 + checker.0) as i8);
                }
                show.push(IntMove {sx: k_sq % 8, sy: k_sq / 8 as i8, ex: k_sq % 8 as i8, ey: k_sq / 8 as i8, promotion_piece: None});
                highlights.push((k_sq) as i8);
            }

            for m in show {
                self.fog_mask[m.ey as usize * 8 + m.ex as usize] = 0.0_f32.max((self.fog_mask[m.ey as usize * 8 + m.ex as usize] as f32) * (0.3_f32.powf((self.dt.as_nanos() as f32) / 1000_000_000.0))).floor() as u8;
            }
        }
        else {
            for i in 0..64usize {
                self.fog_mask[i] = 255.0_f32.min(((self.fog_mask[i] as f32).max(1.0)) * (3.33_f32.powf((self.dt.as_nanos() as f32) / 1000_000_000.0))).ceil() as u8;
            }
        }
        // #endregion
        
        for h in highlights {
            let p = 1.0 - (20.0_f32 / 30.0).powf((self.dt.as_nanos() as f32 / 1000_000_000.0) * 30.0);
            if rand::rng().random_bool(p as f64) {
                self.embers.push(Ember {
                    x: TOP_LEFT[0] + (32.0*GAME_SCALE*((h % 8) as f32)) + rand::rng().random::<f32>()*32.0*GAME_SCALE,
                    y: TOP_LEFT[1] + (32.0*GAME_SCALE*((7-(h/8)) as f32)) + rand::rng().random::<f32>()*32.0*GAME_SCALE,
                    color: graphics::Color::new(rand::rng().random::<f32>() * 0.7 + 0.3, rand::rng().random::<f32>() * 0.2 + 0.25, 0.25, 1.0),
                    brightness: 1.0,
                });
            }
        }

        // Update the position and brightness of all embers and draw them
        for e in &mut self.embers {
            // Update
            e.x += (rand::rng().random::<f32>() - 0.5) * 40.0 * (((self.dt.as_nanos()) as f32) / 1000_000_000.0);
            e.y += -1.4 * 6.0 * (((self.dt.as_nanos()) as f32) / 1000_000_000.0);
            e.brightness *= 0.7_f32.powf((self.dt.as_nanos() as f32) / 1000_000_000.0);

            // Draw
            let c3 = graphics::Color::new((e.color.r - 0.2) * e.brightness + 0.2, e.color.g * e.brightness, (e.color.b + 0.3) * e.brightness - 0.3, e.color.a * e.brightness);
            canvas.draw(
                &self.image_ember,
                graphics::DrawParam::new()
                    .color(c3)
                    .dest([e.x, e.y])
                    .scale([2.0, 2.0])
            );
        }

        // Destroy
        self.embers.retain(|e| e.brightness > 0.01);

        // #region Pieces Layer 2 and move hints
        for r in (0..8).rev() {
            for c in 0usize..8 {
                if int_board[r][c] == 12 {
                    continue;
                }
                let mut sheet_coord = match int_board[r][c] % 6 {
                    0 => [0.0,0.0],
                    1 => [3.0,0.0],
                    2 => [1.0,0.0],
                    3 => [2.0,0.0],
                    4 => [4.0,0.0],
                    5 => [5.0,0.0],
                    _ => [0.0,0.0],
                };
                let colour = match int_board[r][c] {
                    x if x >= 6 => true,
                    _ => false,
                };
                if colour {
                    sheet_coord = [sheet_coord[0], sheet_coord[1] + 1.0];
                }
                canvas.draw(
                    &self.image_pieces,
                    graphics::DrawParam::new()
                        .src(graphics::Rect::new(
                            (sheet_coord[0] * 32.0) / 192.0,
                            (sheet_coord[1] * 32.0) / 128.0,
                            32.0 / 192.0,
                            32.0 / 128.0,
                        ))
                        .dest([TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)), TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32))])
                        .scale([GAME_SCALE, GAME_SCALE])
                );
            }
        }

        // Move hints
        if self.start != [-1, -1] {
            for m in self.board.valid_moves.clone() {
                if m.sx == self.start[0] && m.sy == self.start[1] && (m.promotion_piece == None || m.promotion_piece == Some(Piece::Queen)) {
                    
                    canvas.draw(
                        &self.image_hints,
                        graphics::DrawParam::new()
                            .dest([TOP_LEFT[0] + (32.0*GAME_SCALE*(m.ex as f32)), TOP_LEFT[1] + (32.0*GAME_SCALE*((7-m.ey) as f32))])
                            .color(graphics::Color::new(1.0, 1.0, 1.0, 0.5))
                            .scale([GAME_SCALE, GAME_SCALE])
                    );
                }
            }
        }
        // #endregion

        // #region Fog Layer 1 (Hard)

        let width = (screen.0 as u32) / (GAME_SCALE as u32);
        let height = (screen.1 as u32) / (GAME_SCALE as u32);

        // RGBA8 data: 4 bytes per generated pixel.
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                // Your arbitrary function goes here.
                let (r, g, b, a) = hard_fog_pixel(x, y, self.fog_mask.clone());

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);
            }
        }

        let image = graphics::Image::from_pixels(
            ctx,
            &pixels,
            graphics::ImageFormat::Rgba8UnormSrgb,
            width,
            height,
        );

        canvas.draw(
            &image,
            graphics::DrawParam::default().scale([GAME_SCALE, GAME_SCALE]),
        );
        // #endregion

        // Embers
        // Only make new embers if in stage 1 or 4

        // Ensure kings embers are brighter and attacker is shown when in check
        // Also highlight all adjacant squares than can be attacked by an enemy in embers, but don't highlight the enemies responsible for those
        // Highlighting means having a high change of generating a random ember at a random coordinate within that square every frame, and dispells hard fog

        // For each piece, if it's their turn, generate embers at an assigned coordinate relative to them every fram with a probability.
        // The queen and knight have no embers, and so no coordinate either.


        // #region Wrap up. Add eyes, fog layer 2, transition to stage 0 or 3, and draw promotion selection to canvas if applicable
        // Eyes
        if self.stage == 1 || self.stage == 4 || self.game_state != 0 {
            self.eye_brightness = 255.0_f32.min(((self.eye_brightness as f32).max(1.0)) * (5_f32.powf((self.dt.as_nanos() as f32) / 1000_000_000.0))).ceil() as u8;
        }
        else {
            self.eye_brightness = 0.0_f32.max(((self.eye_brightness as f32)) * (0.2_f32.powf((self.dt.as_nanos() as f32) / 1000_000_000.0))).floor() as u8;
        }
        for r in (0..8).rev() {
            for c in 0usize..8 {
                if int_board[r][c] == 4 {
                    let mut c2 = graphics::Color::new(1.0, 1.0, 1.0, (self.eye_brightness as f32)/255.0);
                    if self.stage >= 4 && self.game_state == 0 {
                        c2 = graphics::Color::new(1.0, 0.0, 0.0, (self.eye_brightness as f32)/255.0);
                    }
                    canvas.draw(
                        &self.image_eyes,
                        graphics::DrawParam::new()
                            .color(c2)
                            .dest([TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)), TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32))])
                    );
                }
                else if int_board[r][c] == 10 {
                    let mut c2 = graphics::Color::new(1.0, 1.0, 1.0, (self.eye_brightness as f32)/255.0);
                    if self.stage <= 3 && self.game_state == 0 {
                        c2 = graphics::Color::new(1.0, 0.0, 0.0, (self.eye_brightness as f32)/255.0);
                    }
                    canvas.draw(
                        &self.image_eyes,
                        graphics::DrawParam::new()
                            .color(c2)
                            .dest([TOP_LEFT[0] + (32.0*GAME_SCALE*(c as f32)), TOP_LEFT[1] + (32.0*GAME_SCALE*((7-r) as f32))])
                    );
                }
            }
        }

        if self.stage == 2 || self.stage == 5 {
            if self.promoting_move != None {
                // Draw Choices
                if self.promoting_move.unwrap().promotion_piece == None {
                    let left = match self.stage {
                        2 => TOP_LEFT[0] - PROMOTION_MARGIN - (32.0*GAME_SCALE),
                        _ => TOP_LEFT[0] + (256.0*GAME_SCALE) + PROMOTION_MARGIN,
                    };
                    canvas.draw(
                        &self.image_promo,
                        graphics::DrawParam::new()
                            .dest([left, TOP_LEFT[1]])
                            .scale([GAME_SCALE, GAME_SCALE])
                    );
                    for y in match self.stage {
                        2 => [3.0, 0.0],
                        _ => [3.0, 1.0],
                    } {
                        canvas.draw(
                            &self.image_pieces,
                            graphics::DrawParam::new()
                                .src(graphics::Rect::new(
                                    (4.0 * 32.0) / 192.0,
                                    (y * 32.0) / 128.0,
                                    32.0 / 192.0,
                                    32.0 / 128.0,
                                ))
                                .dest([left, TOP_LEFT[1] + (0.0*32.0*GAME_SCALE)])
                                .scale([GAME_SCALE, GAME_SCALE])
                        );
                        canvas.draw(
                            &self.image_pieces,
                            graphics::DrawParam::new()
                                .src(graphics::Rect::new(
                                    (1.0 * 32.0) / 192.0,
                                    (y * 32.0) / 128.0,
                                    32.0 / 192.0,
                                    32.0 / 128.0,
                                ))
                                .dest([left, TOP_LEFT[1] + (1.0*32.0*GAME_SCALE)])
                                .scale([GAME_SCALE, GAME_SCALE])
                        );
                        canvas.draw(
                            &self.image_pieces,
                            graphics::DrawParam::new()
                                .src(graphics::Rect::new(
                                    (2.0 * 32.0) / 192.0,
                                    (y * 32.0) / 128.0,
                                    32.0 / 192.0,
                                    32.0 / 128.0,
                                ))
                                .dest([left, TOP_LEFT[1] + (2.0*32.0*GAME_SCALE)])
                                .scale([GAME_SCALE, GAME_SCALE])
                        );
                        canvas.draw(
                            &self.image_pieces,
                            graphics::DrawParam::new()
                                .src(graphics::Rect::new(
                                    (3.0 * 32.0) / 192.0,
                                    (y * 32.0) / 128.0,
                                    32.0 / 192.0,
                                    32.0 / 128.0,
                                ))
                                .dest([left, TOP_LEFT[1] + (3.0*32.0*GAME_SCALE)])
                                .scale([GAME_SCALE, GAME_SCALE])
                        );
                    };
                }
                else {
                    let _ = self.board.make_and_validate_move(self.promoting_move.unwrap());
                    if self.board.is_checkmate() {
                        self.game_state = match self.stage {
                            5 => 3,
                            2 => 1,
                            _ => 0,
                        }
                    }
                    else if self.board.is_stalemate() {
                        self.game_state = 2
                    }
                    self.promoting_move = None;
                }
            }
            else {
                let mut fog_all = true;
                for fog in &self.fog_mask {
                    if *fog <= 248 {
                        fog_all = false;
                        break;
                    }
                }
                if fog_all {
                    self.stage = (self.stage + 1) % 6;
                }
            }
        }

        // Fog Layer 2 (Soft)
        // The opacity applied to this by the fog mask is only reduced to a certain minimum value, never all the way to 0 like hard fog regardless of the mask value.
        // Its procedurally generated monochrome texture is also modulated to be orange slightly for all the points near an ember for each ember.
        // There is not step to change the opacity of the soft fog around the bells
        // Its modulating texture includes opacity as well, so the opacity itself is variable and large parts are fairly transparent
        // Finally, the whole underlying texture is shifted by a constant x and y coordinate every frame, simulating wind

        if false {
            let width = (screen.0 as u32) / (GAME_SCALE as u32);
            let height = (screen.1 as u32) / (GAME_SCALE as u32);

            // RGBA8 data: 4 bytes per generated pixel.
            let mut pixels = Vec::with_capacity((width * height * 4) as usize);

            for y in 0..height {
                for x in 0..width {
                    // Your arbitrary function goes here.
                    let (r, g, b, a) = soft_fog_pixel(x, y, self.fog_mask.clone());

                    pixels.push(r);
                    pixels.push(g);
                    pixels.push(b);
                    pixels.push(a);
                }
            }

            let image = graphics::Image::from_pixels(
                ctx,
                &pixels,
                graphics::ImageFormat::Rgba8UnormSrgb,
                width,
                height,
            );

            canvas.draw(
                &image,
                graphics::DrawParam::default().scale([GAME_SCALE, GAME_SCALE]),
            );
        }
        // #endregion

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("Chess", "Curpas")
        .default_conf(c)
        .add_resource_path("./assets")
        .window_setup(
            conf::WindowSetup::default()
                .title("The Fog Is Coming")
        )
        .window_mode(
            conf::WindowMode::default()
                .fullscreen_type(conf::FullscreenType::Desktop)
        )
        .build()
        .unwrap();
    let state = State::new(&mut ctx)?;
    let _ = event::run(ctx, event_loop, state);
    return Ok(())
}