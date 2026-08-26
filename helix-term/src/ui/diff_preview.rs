use crate::compositor::{Component, Context};
use crate::ui::markdown::highlighted_code_block;
use helix_core::syntax;
use helix_view::{graphics::Rect, Theme};
use tui::{
    buffer::Buffer as Surface,
    text::{Span, Spans, Text},
    widgets::{Paragraph, Widget, Wrap},
};

pub struct DiffPreview {
    contents: Text<'static>,
    size: (u16, u16),
    viewport: (u16, u16),
}

impl DiffPreview {
    pub fn new(
        before: &str,
        after: &str,
        language: &str,
        theme: &Theme,
        loader: &syntax::Loader,
    ) -> Self {
        let mut lines = vec![Spans::from(Span::styled(
            "hunk preview",
            theme.get("ui.text.focus"),
        ))];

        append_side(
            &mut lines,
            before,
            "- ",
            theme.get("diff.minus"),
            language,
            theme,
            loader,
        );
        append_side(
            &mut lines,
            after,
            "+ ",
            theme.get("diff.plus"),
            language,
            theme,
            loader,
        );

        Self {
            contents: Text::from(lines),
            size: (0, 0),
            viewport: (0, 0),
        }
    }
}

fn append_side(
    lines: &mut Vec<Spans<'static>>,
    text: &str,
    prefix: &str,
    diff_style: helix_view::graphics::Style,
    language: &str,
    theme: &Theme,
    loader: &syntax::Loader,
) {
    let highlighted = highlighted_code_block(text, language, Some(theme), loader, None);

    for line in highlighted.lines {
        let mut spans = vec![Span::styled(prefix.to_owned(), diff_style)];
        spans.extend(line.0.into_iter().map(|span| Span {
            content: span.content.into_owned().into(),
            style: diff_style.patch(span.style),
        }));
        lines.push(Spans::from(spans));
    }
}

impl Component for DiffPreview {
    fn render(&mut self, area: Rect, surface: &mut Surface, cx: &mut Context) {
        Paragraph::new(&self.contents)
            .wrap(Wrap { trim: false })
            .scroll((cx.scroll.unwrap_or_default() as u16, 0))
            .render(area, surface);
    }

    fn required_size(&mut self, viewport: (u16, u16)) -> Option<(u16, u16)> {
        if viewport != self.viewport {
            let width = self.contents.width() as u16;
            let height = self.contents.height() as u16;
            self.size = (width.min(viewport.0), height.min(viewport.1));
            self.viewport = viewport;
        }
        Some(self.size)
    }
}
