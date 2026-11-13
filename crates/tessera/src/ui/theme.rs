//! Theme palettes, system fonts and the egui style.

use egui::epaint::text::Blob;
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Visuals};
use std::sync::Arc;
use tessera_core::disk::{Category, Kind};

use crate::settings::ThemePreset;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub preset: ThemePreset,
    pub dark: bool,
    pub bg: Color32,
    pub bg_glow_a: Color32,
    pub bg_glow_b: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub popup: Color32,
    pub border: Color32,
    pub highlight: Color32,
    pub shadow: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub text3: Color32,
    pub track: Color32,
    pub grid: Color32,
    pub tile_text: Color32,

    pub blue: Color32,
    pub violet: Color32,
    pub teal: Color32,
    pub cyan: Color32,
    pub pink: Color32,
    pub orange: Color32,
    pub amber: Color32,
    pub green: Color32,
    pub red: Color32,
    pub indigo: Color32,
}

const fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            preset: ThemePreset::Default,
            dark: true,
            bg: hex(0x0A0C12),
            bg_glow_a: Color32::from_rgba_unmultiplied(110, 90, 255, 30),
            bg_glow_b: Color32::from_rgba_unmultiplied(20, 200, 180, 18),
            card: Color32::from_rgba_unmultiplied(255, 255, 255, 9),
            card_hover: Color32::from_rgba_unmultiplied(255, 255, 255, 14),
            popup: hex(0x161A22),
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 17),
            highlight: Color32::from_rgba_unmultiplied(255, 255, 255, 10),
            shadow: Color32::from_rgba_unmultiplied(0, 0, 0, 90),
            text: hex(0xEEF1F7),
            text2: hex(0xA5AEBD),
            text3: hex(0x6C7584),
            track: Color32::from_rgba_unmultiplied(255, 255, 255, 16),
            grid: Color32::from_rgba_unmultiplied(255, 255, 255, 9),
            tile_text: Color32::WHITE,
            blue: hex(0x5B8CFF),
            violet: hex(0x9B7BFF),
            teal: hex(0x2DD4BF),
            cyan: hex(0x38BDF8),
            pink: hex(0xF472B6),
            orange: hex(0xFB923C),
            amber: hex(0xFBBF24),
            green: hex(0x34D399),
            red: hex(0xF87171),
            indigo: hex(0x818CF8),
        }
    }

    pub fn light() -> Self {
        Self {
            preset: ThemePreset::Default,
            dark: false,
            bg: hex(0xF3F5F9),
            bg_glow_a: Color32::from_rgba_unmultiplied(120, 100, 255, 26),
            bg_glow_b: Color32::from_rgba_unmultiplied(20, 190, 170, 20),
            card: Color32::from_rgba_unmultiplied(255, 255, 255, 215),
            card_hover: Color32::from_rgba_unmultiplied(255, 255, 255, 245),
            popup: Color32::WHITE,
            border: Color32::from_rgba_unmultiplied(15, 23, 42, 16),
            highlight: Color32::from_rgba_unmultiplied(255, 255, 255, 120),
            shadow: Color32::from_rgba_unmultiplied(15, 23, 42, 22),
            text: hex(0x0F172A),
            text2: hex(0x4B5565),
            text3: hex(0x8A93A3),
            track: Color32::from_rgba_unmultiplied(15, 23, 42, 14),
            grid: Color32::from_rgba_unmultiplied(15, 23, 42, 10),
            tile_text: Color32::WHITE,
            blue: hex(0x3B6FF0),
            violet: hex(0x7C5CF0),
            teal: hex(0x0FB5A3),
            cyan: hex(0x0EA5E9),
            pink: hex(0xE6519A),
            orange: hex(0xF07B2C),
            amber: hex(0xE8A10C),
            green: hex(0x16B07A),
            red: hex(0xE5484D),
            indigo: hex(0x5B63E8),
        }
    }

    pub fn resolve(preset: ThemePreset, system_dark: bool) -> Self {
        match preset {
            ThemePreset::Default if system_dark => Self::dark(),
            ThemePreset::Default => Self::light(),
            ThemePreset::Mars => Self::mars(),
            ThemePreset::LiquidGlass => Self::liquid_glass(system_dark),
            ThemePreset::Minimal => Self::minimal(system_dark),
        }
    }

    fn mars() -> Self {
        Self {
            preset: ThemePreset::Mars,
            dark: true,
            bg: hex(0x110F0F),
            bg_glow_a: Color32::from_rgba_unmultiplied(195, 69, 37, 54),
            bg_glow_b: Color32::from_rgba_unmultiplied(132, 83, 61, 29),
            card: hex(0x262322),
            card_hover: hex(0x39312E),
            popup: hex(0x302A28),
            border: hex(0x51413B),
            highlight: hex(0x735244),
            shadow: Color32::from_black_alpha(48),
            text: hex(0xF5E8D9),
            text2: hex(0xD3BDAA),
            text3: hex(0xB09A8C),
            track: hex(0x473C37),
            grid: hex(0x38302D),
            tile_text: Color32::WHITE,
            blue: hex(0xF18461),   // ember
            violet: hex(0xD4A083), // clay
            teal: hex(0xB8ACA0),   // ash
            cyan: hex(0xE6B890),   // sandstone
            pink: hex(0xE7A1A0),
            orange: hex(0xF1A25B),
            amber: hex(0xEAC583),
            green: hex(0xB9BB9B), // dusted sage
            red: hex(0xFF7468),
            indigo: hex(0xB98F94),
        }
    }

    fn liquid_glass(dark: bool) -> Self {
        let color = |smoked, pearl| hex(if dark { smoked } else { pearl });
        Self {
            preset: ThemePreset::LiquidGlass,
            dark,
            bg: color(0x101C30, 0xDCE5F0),
            bg_glow_a: hex(0x2DE9DF).gamma_multiply(if dark { 0.31 } else { 0.37 }),
            bg_glow_b: hex(0x9E83F6).gamma_multiply(if dark { 0.44 } else { 0.48 }),
            card: color(0x314254, 0xFFFFFF).gamma_multiply(if dark { 0.46 } else { 0.62 }),
            card_hover: color(0x416078, 0xDBE9F4),
            popup: color(0x203344, 0xF2F8FC),
            border: color(0xACDDE6, 0x537C9A).gamma_multiply(0.38),
            highlight: Color32::from_white_alpha(if dark { 110 } else { 216 }),
            shadow: color(0x020C1A, 0x364B70).gamma_multiply(if dark { 0.29 } else { 0.17 }),
            text: color(0xF1FBFF, 0x162A3E),
            text2: color(0xCAE0ED, 0x344F66),
            text3: color(0xB0C9DB, 0x4B6076),
            track: color(0xADDEEE, 0x345E7E).gamma_multiply(0.17),
            grid: color(0xC3E8F6, 0x345E7E).gamma_multiply(0.10),
            tile_text: Color32::WHITE,
            blue: color(0x66E7E3, 0x006E79),
            violet: color(0xC1B0FF, 0x7050B6),
            teal: color(0x8CF4C3, 0x087453),
            cyan: color(0x89CFFF, 0x206996),
            pink: color(0xEEA5DE, 0x983F80),
            orange: color(0xFFB784, 0x994922),
            amber: color(0xEDCD91, 0x845B1C),
            green: color(0x9ACDAE, 0x3C6B53),
            red: color(0xFF9B9F, 0xA43D4F),
            indigo: color(0x98ACFC, 0x525F9F),
        }
    }

    fn minimal(dark: bool) -> Self {
        let gray = |ink, paper| Color32::from_gray(if dark { ink } else { paper });
        Self {
            preset: ThemePreset::Minimal,
            dark,
            bg: gray(16, 246),
            bg_glow_a: Color32::TRANSPARENT,
            bg_glow_b: Color32::TRANSPARENT,
            card: gray(23, 253),
            card_hover: gray(48, 230),
            popup: gray(31, 255),
            border: gray(60, 202),
            highlight: Color32::TRANSPARENT,
            shadow: Color32::TRANSPARENT,
            text: gray(242, 24),
            text2: gray(199, 69),
            text3: gray(158, 100),
            track: gray(55, 225),
            grid: gray(43, 233),
            tile_text: Color32::WHITE,
            blue: gray(231, 32),
            violet: gray(140, 115),
            teal: gray(158, 105),
            cyan: gray(202, 64),
            pink: gray(173, 90),
            orange: gray(190, 89),
            amber: gray(150, 117),
            green: gray(135, 113),
            red: gray(249, 16),
            indigo: gray(119, 146),
        }
    }

    /// Use quieter versions of metric accents across large storage tiles.
    fn storage_color(&self, default: u32, accent: Color32) -> Color32 {
        if self.preset == ThemePreset::Default {
            hex(default)
        } else {
            accent.lerp_to_gamma(self.bg, if self.dark { 0.26 } else { 0.18 })
        }
    }

    pub fn category(&self, c: Category) -> Color32 {
        match c {
            Category::Apps => self.storage_color(0x3B82F6, self.blue),
            Category::Projects => self.storage_color(0x8B5CF6, self.violet),
            Category::Developer => self.storage_color(0x14B8A6, self.teal),
            Category::AiModels => self.storage_color(0xEC4899, self.pink),
            Category::Media => self.storage_color(0xF97316, self.orange),
            Category::Documents => self.storage_color(0xEAB308, self.amber),
            Category::AppData => self.storage_color(0x6366F1, self.indigo),
            Category::Caches => self.storage_color(0xF43F5E, self.red),
            Category::Trash => self.storage_color(0xFB7185, self.cyan),
            Category::System => self.storage_color(0x64748B, self.text3),
            Category::Other => self.storage_color(0x8B95A7, self.text2),
        }
    }

    /// Tile colour for a node of `kind` inside a block of base colour `base`.
    pub fn kind_color(&self, kind: Kind, base: Color32) -> Color32 {
        match kind {
            Kind::Artifacts | Kind::Cache => self.category(Category::Caches),
            Kind::Git => self.storage_color(0xF59E0B, self.amber),
            Kind::App => self.category(Category::Apps),
            Kind::Project => self.category(Category::Projects),
            Kind::VirtualMachine | Kind::VmDisk => self.category(Category::Developer),
            Kind::Video => self.category(Category::Media),
            Kind::Image | Kind::MediaLibrary => self.storage_color(0xFB923C, self.cyan),
            Kind::Audio => self.category(Category::AiModels),
            Kind::Archive | Kind::DiskImage => self.category(Category::Documents),
            Kind::Model => self.storage_color(0xD946EF, self.pink),
            Kind::Database => self.category(Category::AppData),
            _ => base,
        }
    }

    /// Pick the higher contrast ink after compositing translucent tiles.
    pub fn tile_ink(&self, fill: Color32) -> Color32 {
        let surface = self.bg.blend(self.card).blend(fill);
        let linear = egui::Rgba::from(surface);
        let luminance = 0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b();
        // Black and white have equal contrast at relative luminance 0.179.
        if luminance > 0.179 {
            Color32::BLACK
        } else {
            self.tile_text
        }
    }

    pub fn severity(&self, level: u8) -> Color32 {
        match level {
            0 => self.green,
            1 => self.amber,
            2 => self.orange,
            _ => self.red,
        }
    }

    pub fn apply(&self, ctx: &egui::Context, slot: egui::Theme) {
        let mut visuals = if self.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.panel_fill = self.bg;
        visuals.window_fill = self.popup;
        visuals.window_stroke = egui::Stroke::new(1.0, self.border);
        visuals.window_corner_radius = egui::CornerRadius::same(12);
        visuals.menu_corner_radius = egui::CornerRadius::same(10);
        visuals.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: self.shadow,
        };
        visuals.popup_shadow = egui::Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: self.shadow,
        };
        visuals.selection.bg_fill = self.blue.gamma_multiply(0.5);
        visuals.hyperlink_color = self.blue;
        visuals.override_text_color = Some(self.text);
        for w in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            w.corner_radius = egui::CornerRadius::same(7);
        }
        visuals.widgets.inactive.weak_bg_fill = self.track;
        visuals.widgets.inactive.bg_fill = self.track;
        visuals.widgets.hovered.weak_bg_fill = self.card_hover;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        if self.preset != ThemePreset::Default {
            visuals.weak_text_color = Some(self.text3);
            visuals.selection.bg_fill = self.blue.gamma_multiply(0.22);
            visuals.selection.stroke.color = self.text;
            visuals.faint_bg_color = self.grid;
            visuals.extreme_bg_color = self.bg;
            visuals.warn_fg_color = self.amber;
            visuals.error_fg_color = self.red;
            for widget in [
                &mut visuals.widgets.hovered,
                &mut visuals.widgets.active,
                &mut visuals.widgets.open,
            ] {
                widget.weak_bg_fill = self.card_hover;
                widget.bg_fill = self.card_hover;
                widget.fg_stroke.color = self.text;
                widget.bg_stroke.color = self.border;
            }
        }
        ctx.set_visuals_of(slot, visuals);
    }
}

// ---------------------------------------------------------------------------
// Fonts: San Francisco straight from the system, at several weights.

pub const REGULAR: &str = "sf-regular";
pub const MEDIUM: &str = "sf-medium";
pub const SEMIBOLD: &str = "sf-semibold";
pub const BOLD: &str = "sf-bold";
pub const ROUNDED: &str = "sf-rounded";

/// Memory-map a system font. Its pages stay clean and shared with every
/// other app using the font, instead of a private copy per weight (SF Pro
/// alone is 8 MB).
fn map_font(path: &str) -> Option<Blob> {
    use std::os::fd::AsRawFd;
    let file = std::fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len() as usize;
    if len == 0 {
        return None;
    }
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        return None;
    }
    // Never unmapped: the fonts are used for the life of the app. The files
    // live on the sealed system volume, so they cannot change underneath.
    let bytes: &'static [u8] = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    Some(Arc::new(bytes))
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    let sf = map_font("/System/Library/Fonts/SFNS.ttf");
    let rounded = map_font("/System/Library/Fonts/SFNSRounded.ttf");
    let mono = map_font("/System/Library/Fonts/SFNSMono.ttf");

    let fallback: Vec<String> = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mut add = |name: &str, blob: &Option<Blob>, weight: f32| {
        let family = FontFamily::Name(name.into());
        let mut chain = Vec::new();
        if let Some(blob) = blob {
            let mut data = FontData::from_blob(blob.clone(), 0);
            data.tweak.coords = egui::epaint::text::VariationCoords::new([(b"wght", weight)]);
            defs.font_data.insert(name.to_string(), Arc::new(data));
            chain.push(name.to_string());
        }
        chain.extend(fallback.iter().cloned());
        defs.families.insert(family, chain);
    };
    add(REGULAR, &sf, 400.0);
    add(MEDIUM, &sf, 510.0);
    add(SEMIBOLD, &sf, 590.0);
    add(BOLD, &sf, 700.0);
    add(ROUNDED, &rounded, 650.0);

    // Default families.
    if sf.is_some() {
        defs.families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, REGULAR.into());
    }
    if let Some(blob) = mono {
        defs.font_data
            .insert("sf-mono".into(), Arc::new(FontData::from_blob(blob, 0)));
        defs.families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "sf-mono".into());
    }
    ctx.set_fonts(defs);
}

pub fn font(size: f32, family: &'static str) -> FontId {
    FontId::new(size, FontFamily::Name(family.into()))
}
