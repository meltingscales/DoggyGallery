use askama::Template;
use crate::models::DirectoryListing;

#[derive(Template)]
#[template(path = "gallery.html")]
pub struct GalleryTemplate {
    pub listing: DirectoryListing,
}
