#![no_std]

use embedded_graphics::{
    draw_target::DrawTarget, geometry::Point, pixelcolor::BinaryColor, Drawable, Pixel,
};

macro_rules! pix {
    ($s:expr) => {{
        const fn to_bits(c: char) -> u32 {
            match c {
                'W' => 0b10,
                'B' => 0b01,
                '_' => 0b00,
                _ => panic!("Invalid character in input string"),
            }
        }

        const fn pix_inner(s: &str) -> u32 {
            let mut result = 0u32;
            let bytes = s.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                result <<= 2;
                result |= to_bits(bytes[i] as char);
                i += 1;
            }
            result
        }

        pix_inner($s)
    }};
}

#[rustfmt::skip]
pub const EIGHT_FLAG: (u32, &[u32]) = (2, &[
    pix!("WB_"),
    pix!("WB_"),
    pix!("BWB"),
    pix!("BWB"),
    pix!("BBB"),
]);

#[rustfmt::skip]
pub const SIXTEENTH_FLAG: (u32, &[u32]) = (2, &[
    pix!("WBB"),
    pix!("BWB"),
    pix!("BWB"),
    pix!("WBB"),
    pix!("BWB"),
    pix!("BWB"),
    pix!("BBB"),
]);

#[rustfmt::skip]
pub const WHOLE_REST: (u32, &[u32]) = (7, &[
    pix!("________"),
    pix!("________"),
    pix!("________"),
    pix!("_WWW____"),
    pix!("_WWW____"),
    pix!("________"),
    pix!("________"),
    pix!("________"),
]);

#[rustfmt::skip]
pub const HALF_REST: (u32, &[u32]) = (7, &[
    pix!("________"),
    pix!("________"),
    pix!("________"),
    pix!("________"),
    pix!("_WWW____"),
    pix!("_WWW____"),
    pix!("________"),
    pix!("________"),
]);

#[rustfmt::skip]
pub const DOTTED_HALF_REST: (u32, &[u32]) = (7, &[
    pix!("________"),
    pix!("________"),
    pix!("________"),
    pix!("________"),
    pix!("_WWW_W__"),
    pix!("_WWW____"),
    pix!("________"),
    pix!("________"),
]);

#[rustfmt::skip]
pub const QUARTER_REST: (u32, &[u32]) = (7, &[
    pix!("_W______"),
    pix!("__W_____"),
    pix!("BBWWB___"),
    pix!("_WWW____"),
    pix!("__W_____"),
    pix!("_WWW____"),
    pix!("BWBBB___"),
    pix!("__W_____"),
]);

#[rustfmt::skip]
pub const DOTTED_QUARTER_REST: (u32, &[u32]) = (7, &[
    pix!("_W______"),
    pix!("__W_____"),
    pix!("BBWWB___"),
    pix!("_WWW____"),
    pix!("__W__W__"),
    pix!("_WWW____"),
    pix!("BWBBB___"),
    pix!("__W_____"),
]);

#[rustfmt::skip]
pub const EIGHTH_REST: (u32, &[u32]) = (7, &[
    pix!("_______"),
    pix!("_BB____"), 
    pix!("BWWBBWB"), 
    pix!("BWWWWB_"), 
    pix!("_BBBWB_"), 
    pix!("__BWB__"), 
    pix!("__BWB__"),
    pix!("_BWB___"),
    pix!("_BWB___"),
]);

#[rustfmt::skip]
pub const DOTTED_EIGHTH_REST: (u32, &[u32]) = (7, &[
    pix!("_______"),
    pix!("_BB____"), 
    pix!("BWWBBWB"), 
    pix!("BWWWWB_"), 
    pix!("_BBBWBW"), 
    pix!("__BWB__"), 
    pix!("__BWB__"),
    pix!("_BWB___"),
    pix!("_BWB___"),
]);

#[rustfmt::skip]
pub const SIXTEENTH_REST: (u32, &[u32]) = (8, &[
pix!("________"),
pix!("________"),
pix!("_BWWBBWB"),
pix!("__WWWW__"),
pix!("_WW__W__"),
pix!("_WWWW___"),
pix!("BBBBWB__"),
pix!("___W____"),
pix!("___W____"),
]);

#[rustfmt::skip]
pub const BASS_CLEF: (u32, &[u32]) = (12, &[
    pix!("___BBBBB____"), 
    pix!("__BWWWWWB___"),
    pix!("_BWWWWWWWB_W"),
    pix!("BWWBBBBWWB__"),
    pix!("BWWWWBBBWWB_"),
    pix!("BWWWWWBBWWB_"),
    pix!("BWWWWWBBWWBW"),
    pix!("_BWWWBBBWWB_"),
    pix!("______BWWB__"),
    pix!("______BWWB__"),
    pix!("______BWB___"),
    pix!("_____BWWB___"),
    pix!("____BWWB____"),
    pix!("__BBWWB_____"),
    pix!("_BWWBB______"),
    pix!("__BB________"),
]);

#[rustfmt::skip]
pub const TIE: (u32, &[u32]) = (5, &[
    pix!("W___W"),
    pix!("_WWW_"),
]);

#[rustfmt::skip]
pub const FILLED_NOTEHEAD: (u32, &[u32]) = (6, &[
    pix!("_BBBB_"),
    pix!("__WW__"),
    pix!("BWWWWB"),
    pix!("__WW__"),
    pix!("_BBBB_"),
]);

macro_rules! art {
    ($name:ident, $size:expr, $($line:tt)*) => {
        #[rustfmt::skip]
        pub const $name: (u32, &[u32]) = ($size, &[
            $(pix!(stringify!($line))),*
        ]);
    };
}

// TODO: change all things to this definition
art!(EMPTY_NOTEHEAD, 6,
_BBBB_
__WW__
BW__WB
__WW__
_BBBB_
);

#[rustfmt::skip]
// pub const EMPTY_NOTEHEAD: (u32, &[u32]) = (6, &[
//     pix!("_BBBB_"),
//     pix!("__WW__"),
//     pix!("BW__WB"),
//     pix!("__WW__"),
//     pix!("_BBBB_"),
// ]);

#[rustfmt::skip]
pub const BEAT_ONE: (u32, &[u32]) = (3, &[
    pix!("_W_"),
    pix!("WW_"),
    pix!("_W_"),
    pix!("_W_"),
]);

#[rustfmt::skip]
pub const BEAT_TWO: (u32, &[u32]) = (3, &[
    pix!("WWW"),
    pix!("__W"),
    pix!("WW_"),
    pix!("WWW"),
]);

#[rustfmt::skip]
pub const BEAT_THREE: (u32, &[u32]) = (3, &[
    pix!("WWW"),
    pix!("_WW"),
    pix!("__W"),
    pix!("WWW"),
]);

#[rustfmt::skip]
pub const BEAT_FOUR: (u32, &[u32]) = (3, &[
    pix!("W_W"),
    pix!("W_W"),
    pix!("WWW"),
    pix!("__W"),
]);

#[rustfmt::skip]
pub const BEAT_E: (u32, &[u32]) = (3, &[
    pix!("___"),
    pix!("WW_"),
    pix!("W__"),
    pix!("WW_"),
]);

#[rustfmt::skip]
pub const BEAT_AND: (u32, &[u32]) = (3, &[
    pix!("___"),
    pix!("_W_"),
    pix!("WWW"),
    pix!("_W_"),
]);

#[rustfmt::skip]
pub const BEAT_A: (u32, &[u32]) = (3, &[
    pix!("___"),
    pix!("_W_"),
    pix!("W_W"),
    pix!("W_W"),
]);

art!(PAUSED, 6,
WWBBWW
WWBBWW
WWBBWW
WWBBWW
WWBBWW
WWBBWW
);

art!(STOPPED, 6,
WWWWWW
WWWWWW
WWWWWW
WWWWWW
WWWWWW
WWWWWW
);

art!(PLAYING, 6,
WWBBBB
WWWWBB
WWWWWW
WWWWWW
WWWWBB
WWBBBB
);

art!(LETTER_A, 5,
BWWWB
WBBBW
WBBBW
WWWWW
WBBBW
WBBBW
);

art!(LETTER_B, 5,
WWWWB
WBBBW
WWWWB
WBBBW
WBBBW
WWWWW
);

art!(LETTER_C, 5,
BWWWB
WBBBW
WBBBB
WBBBB
WBBBW
BWWWB
);

art!(METRONOME_LEFT, 5,
WBBBB
BWBBB
BWBBB
BBWBB
WWWWW
WWWWW
);

art!(METRONOME_CENTER, 5,
BBWBB
BBWBB
BBWBB
BBWBB
WWWWW
WWWWW
);

art!(METRONOME_RIGHT, 5,
BBBBW
BBBWB
BBBWB
BBWBB
WWWWW
WWWWW
);

pub fn draw_symbol<D>(
    target: &mut D,
    position: Point,
    symbol: (u32, &[u32]),
) -> Result<u32, D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    draw_symbol_with_direction(target, position, symbol, false)
}

pub fn draw_symbol_with_direction<D>(
    target: &mut D,
    position: Point,
    symbol: (u32, &[u32]),
    rotate: bool,
) -> Result<u32, D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let (width, symbol) = symbol;
    let height = symbol.len() as i32;
    let mut _max_x = 0;

    for (y, &line) in symbol.iter().enumerate() {
        let y_pos = if rotate {
            height - 1 - y as i32
        } else {
            y as i32
        };

        let mut x = -(16 - (width as i32));

        for i in (0..=15).rev() {
            let bits = (line >> (i * 2)) & 0b11;
            let new_x = if rotate {
                position.x - x
            } else {
                position.x + x
            };
            let draw_pos = Point::new(new_x, position.y + y_pos);

            match bits {
                0b10 => Pixel(draw_pos, BinaryColor::On).draw(target)?,
                0b01 => Pixel(draw_pos, BinaryColor::Off).draw(target)?,
                _ => {}
            }

            if bits == 0b10 || bits == 0b01 {
                _max_x = x;
            }

            x += 1;
        }
    }

    Ok(width)
}
