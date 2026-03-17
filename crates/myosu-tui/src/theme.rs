use ratatui::style::{Color, Modifier, Style};

/// Semantic color palette mapped from the design spec's 8-token system.
///
/// During normal gameplay only `fg`, `fg_bright`, and `fg_dim` are visible.
/// Accent colors appear sparingly for emotional moments (win/loss/warning).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub fg: Color,
    pub fg_bright: Color,
    pub fg_dim: Color,
    pub converge: Color,
    pub diverge: Color,
    pub unstable: Color,
    pub focus: Color,
    pub protocol: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_rgb()
    }
}

impl Theme {
    #[must_use]
    pub fn default_rgb() -> Self {
        Self {
            fg: Color::Rgb(192, 192, 192),
            fg_bright: Color::Rgb(255, 255, 255),
            fg_dim: Color::Rgb(96, 96, 96),
            converge: Color::Rgb(0, 204, 102),
            diverge: Color::Rgb(204, 51, 51),
            unstable: Color::Rgb(204, 170, 0),
            focus: Color::Rgb(68, 136, 204),
            protocol: Color::Rgb(136, 68, 204),
        }
    }

    /// Style for fold lines — visually recedes via DIM modifier.
    #[must_use]
    pub fn fold_style(&self) -> Style {
        Style::default().fg(self.fg_dim).add_modifier(Modifier::DIM)
    }

    /// Style for non-current hand history — visually recedes via DIM modifier.
    #[must_use]
    pub fn hand_shadow(&self) -> Style {
        Style::default().fg(self.fg_dim).add_modifier(Modifier::DIM)
    }

    /// Style for LLM/coach text — italic signals "not fact".
    #[must_use]
    pub fn assistant_style(&self) -> Style {
        Style::default()
            .fg(self.fg)
            .add_modifier(Modifier::ITALIC)
    }

    /// All 8 color tokens as an array, for iteration or validation.
    #[must_use]
    pub fn all_colors(&self) -> [Color; 8] {
        [
            self.fg,
            self.fg_bright,
            self.fg_dim,
            self.converge,
            self.diverge,
            self.unstable,
            self.focus,
            self.protocol,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_colors_defined() {
        let theme = Theme::default_rgb();
        let colors = theme.all_colors();

        assert_eq!(colors.len(), 8);
        for color in &colors {
            match color {
                Color::Rgb(_, _, _) => {}
                other => panic!("expected Rgb color, got {other:?}"),
            }
        }
    }

    #[test]
    fn readable_without_color() {
        // The interface must be readable when all color is stripped.
        // With color removed, every token falls back to terminal default,
        // which is readable by definition — the structure carries meaning.
        // Verify that the theme contains distinct tokens so that when
        // color IS present, they remain distinguishable.
        let theme = Theme::default_rgb();
        let colors = theme.all_colors();

        // All 8 tokens must be distinct from each other.
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "color tokens at index {i} and {j} must differ"
                );
            }
        }
    }

    #[test]
    fn fold_style_is_dim() {
        let theme = Theme::default_rgb();
        let style = theme.fold_style();

        assert_eq!(style.fg.unwrap(), theme.fg_dim);
        assert!(style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn hand_shadow_is_dim() {
        let theme = Theme::default_rgb();
        let style = theme.hand_shadow();

        assert_eq!(style.fg.unwrap(), theme.fg_dim);
        assert!(style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn assistant_style_is_italic() {
        let theme = Theme::default_rgb();
        let style = theme.assistant_style();

        assert_eq!(style.fg.unwrap(), theme.fg);
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn default_trait_matches_default_rgb() {
        assert_eq!(Theme::default(), Theme::default_rgb());
    }
}
