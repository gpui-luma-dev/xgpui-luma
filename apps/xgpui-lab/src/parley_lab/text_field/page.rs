//! Lab page: Parley text fields and text areas, on five screens. In the
//! standalone Parley window each screen is one of the window's own tabs; in
//! XGPUI Lab's "Text Field" tab a tab bar on the page chooses them.
use std::sync::Arc;

use gpui::{Context, Entity, Font, FontFallbacks, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    button as radix_button,
    tabs::{Tabs, TabsItem},
    textfield::TextFieldLook,
};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma_look_radix::{self as radix, Look};

use super::{ParleyTextField, ParleyTextFieldEvent};
use crate::screen::{Hero, LabScreen, PanelShell, TextRole, properties, text};

/// Index of the field that starts disabled.
const DISABLED: usize = 1;
/// Index of the field with a digits-only validator.
const DIGITS: usize = 2;
/// Index of the compact search field.
const SEARCH: usize = 3;
/// Index of the long text area, filled with [`long_text`].
const LONG_AREA: usize = 11;

/// A screen of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Screen {
    Fields,
    RightToLeft,
    TextAreas,
    Scrolling,
    States,
}

impl Screen {
    /// (screen, id, label); the label is also the screen's tab in the standalone window.
    pub(crate) const ALL: [(Self, &'static str, &'static str); 5] = [
        (Self::Fields, "fields", "Text Fields"),
        (Self::RightToLeft, "rtl", "Text Right-to-left"),
        (Self::TextAreas, "areas", "Text Areas"),
        (Self::Scrolling, "scrolling", "Text Scrolling"),
        (Self::States, "states", "Text States"),
    ];

    fn from_id(id: &str) -> Self {
        Self::ALL.iter().find(|(_, key, _)| *key == id).map_or(Self::Fields, |(screen, ..)| *screen)
    }

    fn title(self) -> &'static str {
        Self::ALL.iter().find(|(screen, ..)| *screen == self).map_or("", |(_, _, label)| label)
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Fields => {
                "Left/Right move visually; Alt moves by word; Home/End go to the ends. Backspace removes whole \
                 characters. Double-click selects a word, triple-click all. Enter submits. Cmd/Ctrl-Z undoes \
                 a run of typing or deleting; Cmd/Ctrl-Shift-Z redoes."
            }
            Self::RightToLeft => {
                "Right-to-left text starts at the right edge; Left/Right still move visually. In the text area \
                 each line has its own direction. Selection is logical, so across a change of direction it can \
                 show in pieces."
            }
            Self::TextAreas => {
                "Enter adds a line break; Up/Down move by line and keep the column; Home/End go to the line's \
                 ends; Cmd/Ctrl-Up/Down go to the start or end."
            }
            Self::Scrolling => {
                "Long field: the caret scrolls into view; press inside and drag past the right edge, then hold \
                 still: it keeps scrolling and selecting. Long area: the wheel or trackpad scrolls; press inside \
                 and drag below the bottom edge, then hold still: it keeps scrolling, faster the further out."
            }
            Self::States => {
                "The button toggles the disabled field; its size must not change. Type a letter in the digits \
                 field for the invalid state. The search field is compact with an icon; Escape clears it."
            }
        }
    }
}

/// (label, initial value, placeholder, text area rows, screen)
pub(crate) const FIELDS: [(&str, &str, &str, Option<usize>, Screen); 12] = [
    ("Empty", "", "Type here; try an IME, emoji or Hebrew", None, Screen::Fields),
    ("Disabled (toggle above)", "Not editable while disabled", "", None, Screen::States),
    ("Digits only (validator; type a letter for the invalid state)", "12345", "", None, Screen::States),
    ("Search: compact, prefix icon, Escape clears", "", "Search", None, Screen::States),
    ("Latin, combining marks and emoji", "office café e\u{301} 👍🏽 🇮🇹 done", "", None, Screen::Fields),
    ("Hebrew (right-to-left base)", "שלום עולם 123 office", "", None, Screen::RightToLeft),
    ("Arabic (right-to-left base)", "مرحبا بالعالم 456", "", None, Screen::RightToLeft),
    (
        "Long text (scrolls to follow the caret)",
        "The quick brown fox jumps over the lazy dog, then keeps going past the end of the field so the caret has to scroll.",
        "",
        None,
        Screen::Scrolling,
    ),
    ("Text area: empty", "", "Several lines; Enter adds a line break", Some(4), Screen::TextAreas),
    (
        "Text area: wrapping and line breaks",
        "The quick brown fox jumps over the lazy dog. Long lines wrap at the area's width, \
         and Up/Down keep the caret's column.\nA hard line break starts this line.\n\n\
         An empty line came before this one; keep typing to scroll the area.",
        "",
        Some(4),
        Screen::TextAreas,
    ),
    (
        "Text area: Hebrew, Arabic and English",
        "שלום עולם, this line mixes directions 123.\nمرحبا بالعالم and some English words that wrap across lines.\n\
         office café 👍🏽",
        "",
        Some(4),
        Screen::RightToLeft,
    ),
    ("Text area: long (60 lines)", "", "", Some(6), Screen::Scrolling),
];

/// Sixty numbered lines, some long enough to wrap, for the long text area.
fn long_text() -> String {
    (1..=60)
        .map(|line| {
            if line % 5 == 0 {
                format!("Line {line}: a longer line that wraps at the area's width, to mix soft and hard breaks.")
            } else {
                format!("Line {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Testing aid: light text on the look's text color, for contrast while testing.
fn inverted(mut look: TextFieldLook) -> TextFieldLook {
    std::mem::swap(&mut look.background, &mut look.foreground);
    look.placeholder = look.foreground.opacity(0.6);
    look.caret = look.foreground;
    look
}

pub(crate) fn font() -> Font {
    let mut font = gpui::font("IBM Plex Sans");
    font.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Sans Hebrew".into(), "Noto Sans Arabic".into()]));
    font
}

pub struct TextFieldPage {
    look: Arc<Look>,
    pub(crate) fields: Vec<Entity<ParleyTextField>>,
    /// Toggles the "Disabled" field.
    toggle: Entity<radix_button::Button>,
    /// Chooses the screen in XGPUI Lab; the standalone window uses its own tabs.
    tabs: Option<Entity<Tabs>>,
    pub(crate) screen: Screen,
    last_event: String,
}

impl TextFieldPage {
    /// Fields draw with `look`, the application's active look. `invert_colors`
    /// swaps their text and background for contrast (the standalone window), and
    /// `tab_bar` adds a bar to choose screens (XGPUI Lab).
    pub fn new(look: Arc<Look>, invert_colors: bool, tab_bar: bool, cx: &mut Context<Self>) -> Self {
        let fields: Vec<_> = FIELDS
            .iter()
            .enumerate()
            .map(|(index, (label, value, placeholder, rows, _))| {
                let label = *label;
                let builder = ParleyTextField::new(format!("parley-text-field-{index}"))
                    .look(&look)
                    .surface()
                    .font(font())
                    .value(*value)
                    .placeholder(*placeholder)
                    .aria_label(label)
                    .enabled(index != DISABLED)
                    .look_override(move |look| if invert_colors { inverted(look) } else { look })
                    .on_event(cx, move |this: &mut Self, event, cx| {
                        this.last_event = match event {
                            ParleyTextFieldEvent::Change { value } => format!("{label}: changed to {value:?}"),
                            ParleyTextFieldEvent::Submit { value } => format!("{label}: submitted {value:?}"),
                            ParleyTextFieldEvent::FocusChanged { focused } => format!("{label}: focused {focused}"),
                            ParleyTextFieldEvent::EnabledChanged { enabled } => format!("{label}: enabled {enabled}"),
                        };
                        cx.notify();
                    });
                let builder = match index {
                    LONG_AREA => builder.value(long_text()),
                    DIGITS => builder.validator(Arc::new(|text: &str| text.chars().all(|ch| ch.is_ascii_digit()))),
                    SEARCH => builder.compact().prefix_icon(lucide_svg_static::Icon::Search).clean_on_escape(true),
                    _ => builder,
                };
                match rows {
                    Some(rows) => builder.rows(*rows),
                    None => builder,
                }
                .spawn(cx)
            })
            .collect();
        let toggle = radix::Button::new("parley-text-field-toggle-enabled")
            .look(look.as_ref())
            .outline()
            .label("Enable / disable")
            .on_click(cx, |this: &mut Self, cx| {
                let field = this.fields[DISABLED].clone();
                field.update(cx, |field, cx| field.set_enabled(!field.is_enabled(), cx));
            })
            .spawn(cx);
        let tabs = tab_bar.then(|| {
            radix::Tabs::new("parley-text-field-screens")
                .look(look.as_ref())
                .line()
                .items(Screen::ALL.iter().map(|(_, id, label)| TabsItem::new(*id).label(*label)))
                .active(Screen::ALL[0].1)
                .on_value_changed(cx, |this: &mut Self, id, _, cx| {
                    this.screen = Screen::from_id(id);
                    cx.notify();
                })
                .spawn(cx)
        });
        Self { look, fields, toggle, tabs, screen: Screen::Fields, last_event: "No events yet".into() }
    }

    pub(crate) fn set_screen(&mut self, screen: Screen, cx: &mut Context<Self>) {
        self.screen = screen;
        cx.notify();
    }
}

impl Render for TextFieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !cx.has_global::<crate::parley_lab::ParleyTextSystemActive>() {
            return LabScreen::new("parley-fields", self.look.clone()).crumb("xgpui-parley", "Text Field").title("Text Field")
                .lede("Explore editable text fields, text areas and mixed-direction input.")
                .hero(Hero::full(div().flex().flex_col().gap_2().child(text(&self.look,TextRole::Name,"Parley backend required")).child(text(&self.look,TextRole::Caption,
                    "These controls use GPUI’s paragraph API. Relaunch the application with the Parley text system to edit the samples."))))
                .caption("Current application backend: native")
                .section_panel(PanelShell::new("Launch command").child(text(&self.look,TextRole::Value,"cargo run -p xgpui-lab -- --text-system=parley")))
                .note("The shaping, line layout and paragraph previews use an isolated Parley engine and remain available on the native backend.")
                .into_any_element();
        }
        let screen = self.screen;
        let rows = self.fields.iter().zip(FIELDS).filter(|(_, field)| field.4 == screen).map(|(field, (label, ..))| {
            let state = field.read(cx);
            let (anchor, caret) = state.selection();
            let status = match state.layout_error() {
                Some(error) => format!("Layout error: {error}"),
                None => format!(
                    "{}{} bytes · anchor {} {:?} · caret {} {:?} · scroll {:.0}px",
                    if state.is_invalid() { "invalid · " } else { "" },
                    state.value().len(),
                    anchor.byte_index,
                    anchor.affinity,
                    caret.byte_index,
                    caret.affinity,
                    f32::from(state.scroll()),
                ),
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(text(&self.look, TextRole::Label, label))
                .child(div().w_full().max_w(px(600.)).child(field.clone()))
                .child(text(&self.look, TextRole::Value, status))
        });
        let mut page = LabScreen::new("parley-fields", self.look.clone())
            .crumb("xgpui-parley", "Text Field")
            .title("Text Field")
            .lede("Explore editable text fields, text areas and mixed-direction input.")
            .hero(Hero::full(div().id("parley-text-field-page").flex().flex_col().gap_4().children(rows)))
            .caption(format!("{} · Parley application backend", screen.title()))
            .section_panel(
                PanelShell::new("Events / live")
                    .child(properties(&self.look, [("Last event", self.last_event.clone())])),
            )
            .note(screen.hint());
        if let Some(tabs) = &self.tabs {
            page = page.control(tabs.read(cx).tab_list());
        }
        if screen == Screen::States {
            page = page.action(self.toggle.clone());
        }
        page.into_any_element()
    }
}
