# Phone

The dialer is enabled. This is a local UI implementation; no modem or call backend is connected and no incoming calls, connected state, call audio or real dialing are simulated.

- On Home, digits, * or # open the dialer. F3 opens call history; the menu and configurable shortcuts also reach history.
- In the dialer, digits, * and # are literal. ASCII typing supports a leading +. Left/Right move the cursor, Backspace or the right softkey deletes, and the left softkey opens the local options menu. Choose Return to return to the previous screen. The right softkey returns when the field is empty. Home returns to Home.
- Enter or F3 attempts dialing. The current implementation shows “未连接通话服务，无法拨出。” and records the attempt as not dialed. No transport command is sent.
- Contact details and detected SMS numbers open a prefilled dialer. Returning keeps the existing conversation draft and selection.
- History keeps the latest 100 attempts in `organizer.json`, shared with desktop persistence and embedded runtime saves. Older profiles load with an empty history. Enter opens a record's number; the left softkey opens a blank dialer. Delete asks for confirmation.

The input strip and four contact rows follow the archived handset layout. Up/Down selects a local contact; typing returns focus to the entered number. Contact ordering and the two-item options menu are local policies, not verified handset matching logic.

Numbers are limited to 40 ASCII characters: digits, * and #, with + only at the beginning; an attempt requires at least one digit. Empty or invalid input is not recorded. Historical records do not imply a successful or real call. The shared backend API still reserves call commands/events for later implementation.
