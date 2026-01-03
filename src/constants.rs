//! Constants for the `st` application.

use nu_ansi_term::Color;

/// Name of the `.git` directory.
pub const GIT_DIR: &str = ".git";

/// Name of the global config file.
pub const ST_CFG_FILE_NAME: &str = ".st.toml";

/// Name of the store file, within `.git`.
pub const ST_CTX_FILE_NAME: &str = ".st_store.toml";

/// Array of colors used for displaying stacks in the terminal.
pub const COLORS: [Color; 6] = [
    Color::Blue,
    Color::Cyan,
    Color::Green,
    Color::Purple,
    Color::Yellow,
    Color::Red,
];

pub const QUOTE_CHAR: char = '▌';
pub const FILLED_CIRCLE: char = '●';
pub const EMPTY_CIRCLE: char = '○';
pub const BOTTOM_LEFT_BOX: char = '└';
pub const LEFT_FORK_BOX: char = '├';
pub const VERTICAL_BOX: char = '│';
pub const HORIZONTAL_BOX: char = '─';
