use askama::Template;
use crate::models::DirectoryListing;

#[derive(Template)]
#[template(path = "gallery.html")]
pub struct GalleryTemplate {
    pub listing: DirectoryListing,
    pub emoji_prefix: &'static str,
}

#[derive(Template)]
#[template(path = "music_player.html")]
pub struct MusicPlayerTemplate {
    pub listing: DirectoryListing,
}
