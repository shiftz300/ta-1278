//! One focus policy for drawing, keyboard input and host text routing.
use super::*;

enum Focus {
    Dial,
    Overlay,
    Recipient,
    Body,
    Query,
    Contact,
    Timer,
    Event,
    Alarm,
}
impl Phone {
    fn editor_focus(&self) -> Option<Focus> {
        if self.active_reminder().is_some() || self.messaging.ui.pending.is_some() {
            return None;
        }
        if let Some(overlay) = &self.overlay {
            return matches!(overlay, Overlay::Text { .. }).then_some(Focus::Overlay);
        }
        Some(match self.screen {
            Screen::Tools(tools::ToolPage::Dial) => Focus::Dial,
            Screen::Compose if self.messaging.compose.recipient => Focus::Recipient,
            Screen::Compose => Focus::Body,
            Screen::Conversation
                if self.messaging.chat.focus.is_none()
                    && self.messaging.chat.popup == conversation::Popup::None =>
            {
                Focus::Body
            }
            _ if self.contact_editor_active() => {
                if self.organizer.page == organizer::OrganizerPage::Contacts {
                    Focus::Query
                } else {
                    Focus::Contact
                }
            }
            _ if self.timer_name_active() => Focus::Timer,
            _ if self.event_editor_active() => Focus::Event,
            _ if self.alarm_name_active() => Focus::Alarm,
            _ => return None,
        })
    }
    pub fn editor(&self) -> Option<&Editor> {
        Some(match self.editor_focus()? {
            Focus::Overlay => match self.overlay.as_ref()? {
                Overlay::Text { editor, .. } => editor,
                _ => return None,
            },
            Focus::Dial => &self.calls.number,
            Focus::Recipient => &self.messaging.compose.to,
            Focus::Body => &self.messaging.compose.body,
            Focus::Query => &self.contacts.ui.query,
            Focus::Contact => &self.contacts.ui.input,
            Focus::Timer => &self.tools.timer_edit.name,
            Focus::Event => &self.organizer.event.input,
            Focus::Alarm => &self.organizer.alarm.name,
        })
    }
    pub fn editor_mut(&mut self) -> Option<&mut Editor> {
        let editor = match self.editor_focus()? {
            Focus::Overlay => match self.overlay.as_mut()? {
                Overlay::Text { editor, .. } => editor,
                _ => return None,
            },
            Focus::Dial => &mut self.calls.number,
            Focus::Recipient => &mut self.messaging.compose.to,
            Focus::Body => &mut self.messaging.compose.body,
            Focus::Query => &mut self.contacts.ui.query,
            Focus::Contact => &mut self.contacts.ui.input,
            Focus::Timer => &mut self.tools.timer_edit.name,
            Focus::Event => &mut self.organizer.event.input,
            Focus::Alarm => &mut self.organizer.alarm.name,
        };
        editor.prediction.set_engine(self.prediction_engine.clone());
        Some(editor)
    }
}
