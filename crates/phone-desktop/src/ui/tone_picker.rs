//! Shared built-in/file ringtone choice for settings and organizer forms.
use crate::{
    canvas::Canvas,
    ui::{chrome, list, softkeys::Softkeys},
};
pub fn draw(c: &mut Canvas, title: &str, index: usize, current: &str, default: &str) {
    chrome::title(c, title);
    let labels = crate::native_audio::tone_labels(default);
    list::draw(c, labels.len(), index, list::ListStyle::STANDARD, |i| {
        list::Row {
            label: &labels[i],
            enabled: true,
            marker: if i == crate::native_audio::file_index() {
                list::Marker::None
            } else {
                list::Marker::Radio(labels[i] == current)
            },
        }
    });
    Softkeys::back("选择").draw(c);
}
