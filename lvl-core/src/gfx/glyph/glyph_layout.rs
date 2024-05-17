use crate::gfx::elements::Font;
use fontdue::layout::{GlyphRasterConfig, HorizontalAlign, VerticalAlign, WrapStyle};
use lvl_math::Vec2;

#[derive(Clone, Copy, PartialEq)]
pub struct GlyphLayoutConfig {
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub wrap_style: WrapStyle,
    pub wrap_hard_breaks: bool,
}

impl GlyphLayoutConfig {
    pub fn new(
        horizontal_align: HorizontalAlign,
        vertical_align: VerticalAlign,
        wrap_style: WrapStyle,
        wrap_hard_breaks: bool,
    ) -> Self {
        Self {
            horizontal_align,
            vertical_align,
            wrap_style,
            wrap_hard_breaks,
        }
    }
}

impl Default for GlyphLayoutConfig {
    fn default() -> Self {
        Self {
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap_style: WrapStyle::Word,
            wrap_hard_breaks: true,
        }
    }
}

pub struct GlyphLayoutElement {
    pub size: Vec2,
    pub offset: Vec2,
    pub key: GlyphRasterConfig,
}

struct GlyphLineLayout {
    pub width: f32,
    pub elements: Vec<GlyphLayoutElement>,
}

fn compute_glyph_line_layout(
    font: &Font,
    font_size: f32,
    inset: f32,
    chars: &mut impl Iterator<Item = char>,
) -> GlyphLineLayout {
    let mut prev = None;
    let mut acc_width = 0.0f32;
    let mut acc_horizontal_offset = 0f32;
    let mut elements = Vec::new();

    for c in chars {
        if c == '\n' {
            break;
        }

        let metrics = font.font().metrics(c, font_size);
        let kern = prev
            .and_then(|prev| font.font().horizontal_kern(prev, c, font_size))
            .unwrap_or(0.0f32);

        let offset = Vec2::new(
            -inset + metrics.xmin as f32 + kern + acc_horizontal_offset,
            -inset + metrics.ymin as f32,
        );
        let size = Vec2::new(
            metrics.width as f32 + inset * 2f32,
            metrics.height as f32 + inset * 2f32,
        );
        elements.push(GlyphLayoutElement {
            size,
            offset,
            key: GlyphRasterConfig {
                glyph_index: font.font().lookup_glyph_index(c),
                px: font_size,
                font_hash: font.font().file_hash(),
            },
        });

        acc_width += kern + metrics.advance_width;
        acc_horizontal_offset += kern + metrics.advance_width;

        prev = Some(c);
    }

    GlyphLineLayout {
        width: acc_width,
        elements,
    }
}
