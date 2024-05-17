use fontdue::Font as FontDueFont;

#[derive(Debug)]
pub struct Font {
    font: FontDueFont,
    sdf_font_size: f32,
    sdf_inset: usize,
    sdf_radius: usize,
    sdf_cutoff: f32,
}

impl Font {
    pub fn new(
        font: FontDueFont,
        sdf_font_size: f32,
        sdf_inset: usize,
        sdf_radius: usize,
        sdf_cutoff: f32,
    ) -> Self {
        Self {
            font,
            sdf_font_size,
            sdf_inset,
            sdf_radius,
            sdf_cutoff,
        }
    }

    pub fn font(&self) -> &FontDueFont {
        &self.font
    }

    pub fn sdf_font_size(&self) -> f32 {
        self.sdf_font_size
    }

    pub fn sdf_inset(&self) -> usize {
        self.sdf_inset
    }

    pub fn sdf_radius(&self) -> usize {
        self.sdf_radius
    }

    pub fn sdf_cutoff(&self) -> f32 {
        self.sdf_cutoff
    }
}
