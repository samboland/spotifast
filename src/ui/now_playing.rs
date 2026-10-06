//! Track details alongside the current page.

use super::widgets;
use crate::app::App;
use crate::i18n::gettext;
use crate::model::{Action, Loadable, Page};
use crate::theme::{self, Icon};

pub fn side_panel(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let width = (ui.available_width() - super::topbar::least_width(ui.ctx()))
        .clamp(theme::SIDE_PANEL_MIN_WIDTH, 340.0);
    egui::Panel::right("now-playing-panel")
        .exact_size(width)
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame::new().fill(palette.panel).inner_margin(16))
        .show(ui, |ui| {
            let space =
                super::window_controls_reservation(ui.ctx(), true, false, ui.available_width());
            ui.add_space(space.queue_top);
            theme::text(
                ui,
                gettext(app.locale, "Now playing"),
                theme::bold(18.0),
                palette.text,
            );
            ui.add_space(16.0);
            egui::ScrollArea::vertical()
                .id_salt("now-playing-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let Some(now) = app.now_playing() else {
                        theme::text(
                            ui,
                            gettext(app.locale, "Nothing playing"),
                            theme::regular(14.0),
                            palette.secondary,
                        );
                        return;
                    };
                    let size = ui.available_width();
                    let (rect, response) =
                        ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::click());
                    widgets::paint_cover(
                        ui,
                        &palette,
                        now.art_url.as_deref().or(now.art_small.as_deref()),
                        rect,
                        8.0,
                        Icon::Music,
                        Some(app.backend.art()),
                    );
                    let page = now
                        .album_id
                        .as_ref()
                        .map(|id| Page::Album(id.clone()))
                        .or_else(|| now.show_id.as_ref().map(|id| Page::Show(id.clone())));
                    if response.clicked()
                        && let Some(page) = &page
                    {
                        app.actions.push(Action::Open(page.clone()));
                    }
                    ui.add_space(16.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&now.title)
                                .font(theme::bold(24.0))
                                .color(palette.text),
                        )
                        .wrap(),
                    );
                    ui.horizontal_wrapped(|ui| {
                        if now.artists.is_empty() {
                            theme::text(ui, &now.subtitle, theme::regular(14.0), palette.secondary);
                        } else {
                            widgets::artist_links(
                                ui,
                                app,
                                &now.artists,
                                theme::regular(14.0),
                                palette.secondary,
                            );
                        }
                    });
                    if !now.is_episode {
                        let saved = app.is_saved(&now.uri).unwrap_or(false);
                        let (icon, color, label) = if saved {
                            (
                                Icon::HeartFilled,
                                palette.accent,
                                gettext(app.locale, "Remove from Liked Songs"),
                            )
                        } else {
                            (
                                Icon::Heart,
                                palette.secondary,
                                gettext(app.locale, "Save to Liked Songs"),
                            )
                        };
                        if theme::icon_button(ui, icon, 20.0, color, palette.text, &label).clicked()
                        {
                            app.actions.push(Action::ToggleSaved(now.uri.clone()));
                        }
                    }
                    if !now.album_name.is_empty() {
                        ui.add_space(16.0);
                        theme::text(
                            ui,
                            gettext(app.locale, "Album"),
                            theme::bold(14.0),
                            palette.text,
                        );
                        if theme::link(ui, &now.album_name, theme::regular(14.0), palette.secondary)
                            .clicked()
                            && let Some(page) = &page
                        {
                            app.actions.push(Action::Open(page.clone()));
                        }
                    }
                    ui.add_space(24.0);
                    if theme::link(
                        ui,
                        gettext(app.locale, "Next up"),
                        theme::bold(18.0),
                        palette.text,
                    )
                    .clicked()
                    {
                        app.actions.push(Action::ToggleQueuePanel);
                    }
                    ui.add_space(12.0);
                    match &app.queue {
                        Loadable::Loaded(queue) => {
                            if let Some(next) = queue.queue.first() {
                                ui.horizontal(|ui| {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::Vec2::splat(48.0),
                                        egui::Sense::hover(),
                                    );
                                    widgets::paint_cover(
                                        ui,
                                        &palette,
                                        next.image(96),
                                        rect,
                                        4.0,
                                        Icon::Music,
                                        Some(app.backend.art()),
                                    );
                                    ui.vertical(|ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(next.name())
                                                    .color(palette.text),
                                            )
                                            .wrap(),
                                        );
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(next.subtitle())
                                                    .color(palette.secondary),
                                            )
                                            .wrap(),
                                        );
                                    });
                                });
                            } else {
                                theme::text(
                                    ui,
                                    gettext(app.locale, "Nothing queued"),
                                    theme::regular(14.0),
                                    palette.secondary,
                                );
                            }
                        }
                        Loadable::Loading | Loadable::NotLoaded => {
                            widgets::loading_row(ui, &palette, app.locale)
                        }
                        _ => {
                            theme::text(
                                ui,
                                gettext(app.locale, "Queue"),
                                theme::regular(14.0),
                                palette.secondary,
                            );
                        }
                    }
                });
        });
}
