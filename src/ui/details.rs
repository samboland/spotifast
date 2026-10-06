//! Read-only artist and contributor dialogs.
use super::widgets;
use crate::api::models::pick_image;
use crate::app::App;
use crate::i18n::{gettext, ngettext, pgettext};
use crate::model::{Action, Dialog, Loadable};
use crate::theme::{self, Icon};

fn heading(app: &mut App, ui: &mut egui::Ui, title: &str) {
    let palette = app.palette;
    ui.horizontal(|ui| {
        theme::text(ui, title, theme::bold(24.0), palette.text);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::icon_button(
                ui,
                Icon::X,
                18.0,
                palette.secondary,
                palette.text,
                &gettext(app.locale, "Close"),
            )
            .clicked()
            {
                app.actions.push(Action::CloseDialog);
            }
        });
    });
    ui.add_space(16.0);
}

fn follow(app: &mut App, ui: &mut egui::Ui, uri: &str) {
    let following = app.is_saved(uri).unwrap_or(false);
    let label = if following {
        pgettext(app.locale, "artist", "Following")
    } else {
        pgettext(app.locale, "artist", "Follow")
    };
    if theme::soft_button(ui, &app.palette, None, &label, following).clicked() {
        app.actions.push(Action::ToggleSaved(uri.to_string()));
    }
}

fn state(
    app: &mut App,
    ui: &mut egui::Ui,
    uri: &str,
    retry: Dialog,
) -> Option<crate::details::Details> {
    match app.details.get(uri).cloned().unwrap_or(Loadable::NotLoaded) {
        Loadable::Loaded(details) => Some(details),
        Loadable::Loading | Loadable::NotLoaded => {
            widgets::loading_row(ui, &app.palette, app.locale);
            None
        }
        Loadable::Failed(error) => {
            ui.add(
                egui::Label::new(egui::RichText::new(error).color(app.palette.secondary)).wrap(),
            );
            if theme::soft_button(ui, &app.palette, None, &gettext(app.locale, "Retry"), false)
                .clicked()
            {
                app.actions.push(Action::ShowDialog(retry));
            }
            None
        }
    }
}

pub fn about(app: &mut App, ui: &mut egui::Ui, id: &str) {
    let palette = app.palette;
    ui.set_width((ui.ctx().content_rect().width() - 96.0).clamp(240.0, 720.0));
    heading(app, ui, &gettext(app.locale, "About the artist"));
    egui::ScrollArea::vertical()
        .id_salt("artist-about")
        .min_scrolled_height((ui.ctx().content_rect().height() - 220.0).clamp(120.0, 500.0))
        .max_height((ui.ctx().content_rect().height() - 180.0).max(120.0))
        .show(ui, |ui| {
            let artist = app
                .artist_pages
                .get(id)
                .and_then(|page| page.artist.get())
                .cloned();
            let uri = format!("spotify:artist:{id}");
            if let Some(artist) = artist {
                if let Some(url) = pick_image(&artist.images, 640) {
                    let size = ui
                        .available_width()
                        .min((ui.ctx().content_rect().height() * 0.3).clamp(100.0, 300.0));
                    ui.vertical_centered(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
                        widgets::paint_cover_contained(
                            ui,
                            &palette,
                            Some(url),
                            rect,
                            8.0,
                            Some(app.backend.art()),
                        );
                    });
                }
                ui.add_space(12.0);
                theme::text(ui, &artist.name, theme::bold(22.0), palette.text);
                if let Some(followers) = artist.followers {
                    let label = ngettext(
                        app.locale,
                        "{count} follower",
                        "{count} followers",
                        u32::try_from(followers.total).unwrap_or(u32::MAX),
                    )
                    .replace("{count}", &crate::util::format_count(followers.total));
                    theme::text(ui, label, theme::regular(14.0), palette.secondary);
                }
                follow(app, ui, &uri);
                ui.add_space(16.0);
            }
            if let Some(details) = state(app, ui, &uri, Dialog::ArtistAbout { id: id.into() }) {
                let text = details.biography.unwrap_or_else(|| {
                    gettext(app.locale, "No biography is available.").into_owned()
                });
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(text)
                            .font(theme::regular(16.0))
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
            }
        });
}

pub fn credits(app: &mut App, ui: &mut egui::Ui, uri: &str, name: &str) {
    let palette = app.palette;
    heading(app, ui, &gettext(app.locale, "Credits"));
    ui.add(
        egui::Label::new(
            egui::RichText::new(name)
                .font(theme::bold(18.0))
                .color(palette.text),
        )
        .wrap(),
    );
    ui.add_space(16.0);
    egui::ScrollArea::vertical()
        .id_salt("track-credits")
        .max_height((ui.ctx().content_rect().height() - 220.0).max(120.0))
        .show(ui, |ui| {
            if let Some(details) = state(
                app,
                ui,
                uri,
                Dialog::TrackCredits {
                    uri: uri.into(),
                    name: name.into(),
                },
            ) {
                if details.credits.is_empty() {
                    theme::text(
                        ui,
                        gettext(app.locale, "No contributor credits are available."),
                        theme::regular(14.0),
                        palette.secondary,
                    );
                }
                for credit in details.credits {
                    theme::text(ui, &credit.name, theme::bold(16.0), palette.text);
                    theme::text(
                        ui,
                        crate::details::role_label(credit.role, app.locale),
                        theme::regular(14.0),
                        palette.secondary,
                    );
                    if let Some(uri) = credit.uri {
                        follow(app, ui, &uri);
                    }
                    ui.add_space(14.0);
                }
                if let Some(label) = details.label {
                    theme::text(
                        ui,
                        gettext(app.locale, "Label"),
                        theme::bold(16.0),
                        palette.text,
                    );
                    theme::text(ui, label, theme::regular(14.0), palette.secondary);
                }
            }
        });
}
