//! What a notebook can be given to tell it apart in the sidebar: [`book_icons`]
//! has the icons to choose from and finds one by what was typed, and
//! [`book_colors`] has the colors, as swatches and as the `#rrggbb` a
//! notebook keeps. The apps draw them; which are offered, in what order, and
//! what a notebook keeps is decided here.

pub mod book_colors;
pub mod book_icons;
