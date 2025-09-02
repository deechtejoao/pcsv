use serde::Deserialize;
use std::fs;
use std::path::Path;

type HexColor = String;

#[derive(Debug, Deserialize)]
pub struct ColorScheme {
    pub data_types: DataTypeColors,
    pub header: HexColor,
    pub pager: Option<PagerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DataTypeColors {
    pub text: HexColor,
    pub date: HexColor,
    pub float_number: HexColor,
    pub int_number: HexColor,
    pub boolean: HexColor,
    pub empty: HexColor,
}

#[derive(Debug, Deserialize)]
pub struct PagerConfig {
    pub scroll_single_line: usize,
    pub scroll_multi_line: usize,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            data_types: DataTypeColors {
                text: "#BACEDF".to_string(),
                date: "#FAB387".to_string(),
                float_number: "#89B4FA".to_string(),
                int_number: "#A6E3A1".to_string(),
                boolean: "#F9E2AF".to_string(),
                empty: "#585B70".to_string(),
            },
            header: "#CBB6F7".to_string(),
            pager: Some(PagerConfig {
                scroll_single_line: 1,
                scroll_multi_line: 10,
            }),
        }
    }
}

pub fn load_config(config_path: Option<&str>) -> ColorScheme {
    let paths = match config_path {
        Some(path) => vec![path],
        None => vec!["~/.config/pcsv/config.toml"],
    };

    for path in paths {
        let expanded_path = expand_home(path);
        if Path::new(&expanded_path).exists() {
            if let Ok(content) = fs::read_to_string(&expanded_path) {
                if let Ok(scheme) = toml::from_str::<ColorScheme>(&content) {
                    return scheme;
                }
            }
        }
    }

    ColorScheme::default()
}

fn expand_home(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return format!("{}/{}", home.to_string_lossy(), &path[2..]);
        }
    }
    path.to_string()
}
