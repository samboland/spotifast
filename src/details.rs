//! Optional catalogue details exposed by librespot's metadata reader.
use librespot_core::{Session, SpotifyUri};
use librespot_metadata::{Metadata, artist::Artist, track::Track};

#[derive(Clone, Debug, Default)]
pub struct Details {
    pub biography: Option<String>,
    pub credits: Vec<Credit>,
    pub label: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Credit {
    pub name: String,
    pub uri: Option<String>,
    pub role: i32,
}

pub async fn fetch(session: &Session, uri: &str) -> anyhow::Result<Details> {
    let id = SpotifyUri::from_uri(uri)?;
    match &id {
        SpotifyUri::Artist { .. } => {
            let artist = Artist::get(session, &id).await?;
            let text = artist
                .biographies
                .iter()
                .map(|bio| crate::util::strip_html(&bio.text))
                .filter(|text| !text.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            Ok(Details {
                biography: (!text.is_empty()).then_some(text),
                ..Default::default()
            })
        }
        SpotifyUri::Track { .. } => {
            let track = Track::get(session, &id).await?;
            let mut credits: Vec<_> = track
                .artists_with_role
                .iter()
                .map(|artist| Credit {
                    name: artist.name.clone(),
                    uri: artist.id.to_uri().ok(),
                    role: artist.role as i32,
                })
                .collect();
            // Older metadata may list performers without contributor roles.
            // Keep those names without assigning composition or production credits.
            if credits.is_empty() {
                credits = track
                    .artists
                    .iter()
                    .map(|artist| Credit {
                        name: artist.name.clone(),
                        uri: artist.id.to_uri().ok(),
                        role: 0,
                    })
                    .collect();
            }
            Ok(Details {
                credits,
                label: (!track.album.label.is_empty()).then_some(track.album.label),
                ..Default::default()
            })
        }
        _ => anyhow::bail!("Details are available for artists and tracks."),
    }
}

pub fn role_label(role: i32, locale: crate::i18n::Locale) -> std::borrow::Cow<'static, str> {
    use crate::i18n::gettext;
    match role {
        1 => gettext(locale, "Main Artist"),
        2 => gettext(locale, "Featured Artist"),
        3 => gettext(locale, "Remixer"),
        4 => gettext(locale, "Actor"),
        5 => gettext(locale, "Composer"),
        6 => gettext(locale, "Conductor"),
        7 => gettext(locale, "Orchestra"),
        _ => gettext(locale, "Artist"),
    }
}
