//! Shared controls, following the native title/list/edit/three-softkey structure.
//! Geometry is fitted to this handset; this does not execute the MMK framework.
pub mod art;
pub mod candidates;
pub mod chrome;
pub mod editor;
pub mod form;
pub mod layout;
pub mod list;
pub mod menu;
pub mod navigation;
pub mod popup;
pub mod softkeys;
pub mod text;
pub mod tones;
pub const TITLE_HEIGHT: i32 = 24;
pub const SOFTKEY_TOP: i32 = 284;
pub const SOFTKEY_TEXT_Y: i32 = 289;
