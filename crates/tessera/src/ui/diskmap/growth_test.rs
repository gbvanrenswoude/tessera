use super::*;
use egui::{Event, Modifiers, RawInput, Shape};
use std::time::{Duration, UNIX_EPOCH};
use tessera_core::disk::changes::{DiskChanges, GrowthRow};
use tessera_core::disk::{ScanConfig, ScanProgress, scan};

#[test]
fn growth_row_opens_a_folder_with_only_unindexed_contents() {
    let root = std::env::temp_dir().join(format!("tessera-growth-map-{}", std::process::id()));
    let folder = root.join("Leaf project");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("Cargo.toml"), "[package]").unwrap();
    let config = ScanConfig {
        min_dir: 0,
        threads: 1,
        ..ScanConfig::for_path(&root)
    };
    let index = scan(&config, &ScanProgress::default()).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let id = index.find(&folder).unwrap();
    assert_eq!(index.node(id).child_count, 0);
    let changes = DiskChanges {
        previous_at: UNIX_EPOCH,
        current_at: UNIX_EPOCH + Duration::from_secs(60),
        rows: vec![GrowthRow {
            id,
            category: Category::Projects,
            label: "Leaf project".into(),
            display_path: index.display_path(id),
            growth: index.node(id).size,
            at_least: false,
        }],
    };
    let shared = Shared::new(crate::settings::Settings::default());
    shared.disk.set_index(index);
    let index = shared.disk.view().index.unwrap();
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx);
    let theme = Theme::dark();
    let area = Rect::from_min_size(pos2(20.0, 20.0), vec2(1000.0, 500.0));
    let mut map = DiskMap {
        show_growth: true,
        ..Default::default()
    };
    let frame = |map: &mut DiskMap, events| {
        let input = RawInput {
            screen_rect: Some(area.expand(20.0)),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            if map.show_growth {
                if let Some((category, id)) = growth_rows(ui, area, &changes, &index, &theme) {
                    map.open_growth(&index, category, id);
                }
            } else {
                map.ui(
                    ui,
                    area,
                    &shared,
                    &Snapshot::default(),
                    &theme,
                    TextureId::default(),
                );
            }
        });
        output.textures_delta.clear();
        output.shapes
    };
    let pos = frame(&mut map, vec![])
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(text) if text.galley.text() == "Leaf project" => {
                Some(text.visual_bounding_rect().center())
            }
            _ => None,
        })
        .expect("visible growth result");
    frame(&mut map, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            &mut map,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert!(!map.show_growth);
    frame(&mut map, vec![]);
    assert!(matches!(map.stack.last(), Some(Level::Node { path, .. }) if *path == folder));
    let layout = map.layout.as_ref().unwrap();
    assert_eq!(layout.title, "Leaf project");
    assert_eq!(layout.total, index.node(id).size);
}
