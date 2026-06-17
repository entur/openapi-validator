pub use oav_lib::config::{Config, Jobs, Linter, Mode};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{KeyAction, KeyInput, Keymap};
    use crossterm::event::{KeyCode, KeyModifiers};

    fn parse_config(yaml: &str) -> Config {
        yaml_serde::from_str(yaml).expect("should parse")
    }

    #[test]
    fn keys_scalar_round_trips_through_keymap() {
        let cfg = parse_config("keys:\n  scroll_down: \"x\"\n");
        let (km, warnings) = Keymap::from_config(&cfg.keys);
        assert!(warnings.is_empty());

        let x = KeyInput {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
        };
        assert!(km.has_action(&x, KeyAction::ScrollDown));

        let j = KeyInput {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
        };
        assert!(!km.has_action(&j, KeyAction::ScrollDown));
    }
}
