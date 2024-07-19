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
    pix!("__BB__"),
    pix!("__WW__"),
    pix!("BWWWWB"),
    pix!("__WW__"),
    pix!("__BB__"),
]);
#[rustfmt::skip]
pub const EMPTY_NOTEHEAD: (u32, &[u32]) = (6, &[
    pix!("__BB__"),
    pix!("__WW__"),
    pix!("BW__WB"),
    pix!("__WW__"),
    pix!("__BB__"),
]);

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
    flip_y: bool,
) -> Result<u32, D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let (width, symbol) = symbol;
    let height = symbol.len() as i32;
    let mut _max_x = 0;

    for (y, &line) in symbol.iter().enumerate() {
        let y_pos = if flip_y {
            height - 1 - y as i32
        } else {
            y as i32
        };

        let mut x = -(16 - (width as i32));

        for i in (0..=15).rev() {
            let bits = (line >> (i * 2)) & 0b11;
            let draw_pos = Point::new(position.x + x, position.y + y_pos);

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
