use st::constants::*;

#[test]
fn test_git_dir_constant() {
    assert_eq!(GIT_DIR, ".git");
}

#[test]
fn test_config_file_name() {
    assert_eq!(ST_CFG_FILE_NAME, ".st.toml");
}

#[test]
fn test_context_file_name() {
    assert_eq!(ST_CTX_FILE_NAME, ".st_store.toml");
}

#[test]
fn test_colors_array_length() {
    assert_eq!(COLORS.len(), 6);
}

#[test]
fn test_box_drawing_chars() {
    assert_eq!(VERTICAL_BOX, '│');
    assert_eq!(HORIZONTAL_BOX, '─');
    assert_eq!(BOTTOM_LEFT_BOX, '└');
    assert_eq!(LEFT_FORK_BOX, '├');
}

#[test]
fn test_circle_chars() {
    assert_eq!(FILLED_CIRCLE, '●');
    assert_eq!(EMPTY_CIRCLE, '○');
}

#[test]
fn test_quote_char() {
    assert_eq!(QUOTE_CHAR, '▌');
}

#[test]
fn test_box_chars_are_different() {
    assert_ne!(VERTICAL_BOX, HORIZONTAL_BOX);
    assert_ne!(VERTICAL_BOX, BOTTOM_LEFT_BOX);
    assert_ne!(VERTICAL_BOX, LEFT_FORK_BOX);
    assert_ne!(HORIZONTAL_BOX, BOTTOM_LEFT_BOX);
    assert_ne!(HORIZONTAL_BOX, LEFT_FORK_BOX);
    assert_ne!(BOTTOM_LEFT_BOX, LEFT_FORK_BOX);
}

#[test]
fn test_circle_chars_are_different() {
    assert_ne!(FILLED_CIRCLE, EMPTY_CIRCLE);
}

#[test]
fn test_all_constants_are_valid_unicode() {
    // Ensure all character constants are valid Unicode
    assert!(VERTICAL_BOX.is_ascii_graphic() || !VERTICAL_BOX.is_ascii());
    assert!(HORIZONTAL_BOX.is_ascii_graphic() || !HORIZONTAL_BOX.is_ascii());
    assert!(BOTTOM_LEFT_BOX.is_ascii_graphic() || !BOTTOM_LEFT_BOX.is_ascii());
    assert!(LEFT_FORK_BOX.is_ascii_graphic() || !LEFT_FORK_BOX.is_ascii());
    assert!(FILLED_CIRCLE.is_ascii_graphic() || !FILLED_CIRCLE.is_ascii());
    assert!(EMPTY_CIRCLE.is_ascii_graphic() || !EMPTY_CIRCLE.is_ascii());
    assert!(QUOTE_CHAR.is_ascii_graphic() || !QUOTE_CHAR.is_ascii());
}
