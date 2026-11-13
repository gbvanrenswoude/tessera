use std::time::{SystemTime, UNIX_EPOCH};

use egui::cache::{ComputerMut, FrameCache};
use egui::{Align2, CornerRadius, CursorIcon, Rect, Sense, Ui, UiBuilder, pos2, vec2};
use objc2_foundation::{NSDate, NSDateFormatter, NSDateFormatterStyle};
use tessera_core::disk::changes::DiskChanges;
use tessera_core::disk::{Category, DiskIndex, NodeId};
use tessera_core::format;

use crate::ui::paint;
use crate::ui::theme::{self, Theme};

#[derive(Default)]
struct ScanDate;

impl ComputerMut<SystemTime, String> for ScanDate {
    fn compute(&mut self, time: SystemTime) -> String {
        let seconds = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let date = NSDate::dateWithTimeIntervalSince1970(seconds);
        NSDateFormatter::localizedStringFromDate_dateStyle_timeStyle(
            &date,
            NSDateFormatterStyle::ShortStyle,
            NSDateFormatterStyle::MediumStyle,
        )
        .to_string()
    }
}

pub(super) fn growth_rows(
    ui: &mut Ui,
    rect: Rect,
    changes: &DiskChanges,
    index: &DiskIndex,
    t: &Theme,
) -> Option<(Category, NodeId)> {
    let p = ui.painter_at(rect);
    let period = ui.ctx().memory_mut(|memory| {
        let dates = memory.caches.cache::<FrameCache<String, ScanDate>>();
        let start = dates.get(changes.previous_at).clone();
        format!("{start}  →  {}", dates.get(changes.current_at))
    });
    p.text(
        rect.min,
        Align2::LEFT_TOP,
        period,
        theme::font(12.0, theme::MEDIUM),
        t.text2,
    );
    if changes.rows.is_empty() {
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No measurable growth in the indexed folders.",
            theme::font(14.0, theme::MEDIUM),
            t.text2,
        );
        return None;
    }

    let heading_y = rect.top() + 32.0;
    paint::section(&p, pos2(rect.left() + 10.0, heading_y), "Folder", t);
    let size_x = rect.right() - 175.0;
    p.text(
        pos2(size_x, heading_y),
        Align2::RIGHT_TOP,
        "Current size",
        theme::font(12.0, theme::MEDIUM),
        t.text3,
    );
    p.text(
        pos2(rect.right() - 20.0, heading_y),
        Align2::RIGHT_TOP,
        "Growth",
        theme::font(12.0, theme::SEMIBOLD),
        t.text2,
    );

    let list = Rect::from_min_max(pos2(rect.left(), heading_y + 23.0), rect.max);
    let mut selected = None;
    let mut list_ui = ui.new_child(UiBuilder::new().max_rect(list));
    egui::ScrollArea::vertical()
        .id_salt("growth-list")
        .auto_shrink([false, false])
        .show_rows(&mut list_ui, 44.0, changes.rows.len(), |ui, range| {
                for row in &changes.rows[range] {
                    let (r, response) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
                    let p = ui.painter();
                    if response.hovered() || response.has_focus() {
                        p.rect_filled(r, CornerRadius::same(7), t.track);
                    }
                    p.circle_filled(pos2(r.left() + 10.0, r.top() + 13.0), 4.0, t.category(row.category));
                    let name_x = r.left() + 23.0;
                    let width = size_x - name_x - 110.0;
                    let name = paint::elided(p, &row.label, theme::font(14.0, theme::SEMIBOLD), t.text, width);
                    let path = paint::elided(p, &row.display_path, theme::font(11.5, theme::REGULAR), t.text3, width);
                    p.galley(pos2(name_x, r.top() + 3.0), name, t.text);
                    p.galley(pos2(name_x, r.top() + 23.0), path, t.text3);
                    p.text(pos2(size_x, r.center().y), Align2::RIGHT_CENTER, format::bytes_short(index.node(row.id).size), theme::font(13.0, theme::REGULAR), t.text2);
                    let prefix = if row.at_least { "at least +" } else { "+" };
                    p.text(pos2(rect.right() - 20.0, r.center().y), Align2::RIGHT_CENTER,
                        format!("{prefix}{}", format::bytes_short(row.growth)), theme::font(14.0, theme::SEMIBOLD), t.text);
                    let response = response.on_hover_cursor(CursorIcon::PointingHand);
                    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &row.label));
                    if response.clicked() {
                        selected = Some((row.category, row.id));
                    }
                    response.on_hover_ui(|ui| {
                        ui.label(&row.display_path);
                        if row.at_least {
                            ui.label("The earlier scan included this folder among smaller items. Its growth may be larger.");
                        }
                    });
                }
        });
    selected
}
