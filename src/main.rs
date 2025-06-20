use anyhow::Result;
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use crossterm::style::{Color as CrosstermColor, Stylize};
use csv::ReaderBuilder;
use regex::Regex;
use serde::Deserialize;
use std::{fs, fs::File};

#[derive(Debug, Deserialize)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Deserialize)]
struct ColorScheme {
    background: BackgroundColors,
    text: TextColors,
    bright: BrightColors,
    neutral: NeutralColors,
}

#[derive(Debug, Deserialize)]
struct BackgroundColors {
    dark0: RGB,
    dark1: RGB,
    dark2: RGB,
    dark3: RGB,
}

#[derive(Debug, Deserialize)]
struct TextColors {
    light0: RGB,
    light1: RGB,
    light2: RGB,
}

#[derive(Debug, Deserialize)]
struct BrightColors {
    green: RGB,
    aqua: RGB,
    blue: RGB,
    red: RGB,
    orange: RGB,
    yellow: RGB,
    purple: RGB,
}

#[derive(Debug, Deserialize)]
struct NeutralColors {
    blue: RGB,
    purple: RGB,
}

impl ColorScheme {
    fn rgb(rgb: &RGB) -> Color {
        Color::Rgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }

    fn cell_color(&self, ty: &DataType) -> Color {
        match ty {
            DataType::Number => Self::rgb(&self.bright.green),
            DataType::Boolean => Self::rgb(&self.bright.yellow),
            DataType::Date => Self::rgb(&self.bright.orange),
            DataType::Empty => Self::rgb(&self.background.dark3),
            DataType::Text => Self::rgb(&self.text.light1),
        }
    }
}

fn load_scheme(path: Option<&str>) -> Result<ColorScheme> {
    if let Some(p) = path {
        let toml_str = fs::read_to_string(p)?;
        let scheme: ColorScheme = toml::from_str(&toml_str)?;
        Ok(scheme)
    } else {
        Ok(ColorScheme {
            background: BackgroundColors {
                dark0: RGB { r: 30, g: 30, b: 46 },
                dark1: RGB { r: 49, g: 50, b: 68 },
                dark2: RGB { r: 69, g: 71, b: 90 },
                dark3: RGB { r: 88, g: 91, b: 112 },
            },
            text: TextColors {
                light0: RGB { r: 205, g: 214, b: 244 },
                light1: RGB { r: 186, g: 194, b: 222 },
                light2: RGB { r: 166, g: 173, b: 200 },
            },
            bright: BrightColors {
                green: RGB { r: 166, g: 227, b: 161 },
                aqua: RGB { r: 148, g: 226, b: 213 },
                blue: RGB { r: 137, g: 180, b: 250 },
                red: RGB { r: 243, g: 139, b: 168 },
                orange: RGB { r: 250, g: 179, b: 135 },
                yellow: RGB { r: 249, g: 226, b: 175 },
                purple: RGB { r: 203, g: 166, b: 247 },
            },
            neutral: NeutralColors {
                blue: RGB { r: 116, g: 199, b: 236 },
                purple: RGB { r: 180, g: 190, b: 254 },
            },
        })
    }
}

#[derive(Parser, Debug)]
#[clap(name = "csv-viewer", version, about)]
struct Args {
    file: String,
    #[clap(short = 'n', long = "rows", default_value = "50")]
    max_rows: usize,
    #[clap(short = 'r', long = "row-numbers")]
    show_row_numbers: bool,
    #[clap(short = 'w', long = "width", default_value = "40")]
    max_width: usize,
    #[clap(short = 'd', long = "delimiter", default_value = ",")]
    delimiter: String,
    #[clap(long = "no-header")]
    no_header: bool,
    #[clap(short = 'c', long = "colorscheme")]
    colorscheme: Option<String>,
}

#[derive(Debug, Clone)]
enum DataType {
    Text,
    Number,
    Boolean,
    Date,
    Empty,
}

fn detect_data_type(val: &str) -> DataType {
    let s = val.trim();
    if s.is_empty() {
        return DataType::Empty;
    }
    match s.to_lowercase().as_str() {
        "true" | "false" | "yes" | "no" | "y" | "n" => return DataType::Boolean,
        _ => {}
    }
    if s.parse::<f64>().is_ok() {
        return DataType::Number;
    }
    if is_date_like(s) {
        return DataType::Date;
    }
    DataType::Text
}

fn is_date_like(s: &str) -> bool {
    const PATS: [&str; 3] = [
        r"^\d{4}-\d{2}-\d{2}$",
        r"^\d{2}/\d{2}/\d{4}$",
        r"^\d{2}-\d{2}-\d{4}$",
    ];
    PATS.iter()
        .any(|p| Regex::new(p).map(|re| re.is_match(s)).unwrap_or(false))
}

fn create_table(
    records: &[Vec<String>],
    headers: Option<&[String]>,
    args: &Args,
    scheme: &ColorScheme,
) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(120);

    if let Some(h) = headers {
        let mut cells = Vec::new();
        if args.show_row_numbers {
            cells.push(Cell::new("#").fg(ColorScheme::rgb(&scheme.text.light0)));
        }
        for name in h {
            cells.push(Cell::new(name).fg(ColorScheme::rgb(&scheme.text.light0)));
        }
        table.set_header(cells);
    }

    let max = if args.max_rows == 0 {
        records.len()
    } else {
        args.max_rows.min(records.len())
    };

    for (idx, row) in records.iter().take(max).enumerate() {
        let mut cells = Vec::new();
        if args.show_row_numbers {
            cells.push(Cell::new((idx + 1).to_string()).fg(ColorScheme::rgb(&scheme.bright.aqua)));
        }

        for field in row {
            let mut txt = field.clone();
            if txt.len() > args.max_width {
                txt.truncate(args.max_width.saturating_sub(3));
                txt.push_str("...");
            }
            let ty = detect_data_type(&txt);
            cells.push(Cell::new(txt).fg(scheme.cell_color(&ty)));
        }

        table.add_row(cells);
    }
    table
}

fn print_file_info(path: &str, rows: usize, cols: usize) {
    println!(
        "{}",
        "CSV File Information".with(CrosstermColor::Cyan).bold()
    );
    println!(
        "{} {}",
        "File:".with(CrosstermColor::Blue),
        path.with(CrosstermColor::White)
    );
    println!(
        "{} {}",
        "Rows:".with(CrosstermColor::Blue),
        rows.to_string().with(CrosstermColor::Green)
    );
    println!(
        "{} {}",
        "Columns:".with(CrosstermColor::Blue),
        cols.to_string().with(CrosstermColor::Green)
    );
    println!();
}

fn print_footer(displayed: usize, total: usize) {
    if displayed < total {
        println!();
        println!(
            "{} {} {} {} {} {}",
            "Showing".with(CrosstermColor::Yellow),
            displayed.to_string().with(CrosstermColor::White),
            "of".with(CrosstermColor::Yellow),
            total.to_string().with(CrosstermColor::White),
            "rows. Use".with(CrosstermColor::Yellow),
            "-n 0".with(CrosstermColor::Green)
        );
        println!("{}", "to show all rows.".with(CrosstermColor::Yellow));
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let scheme = load_scheme(args.colorscheme.as_deref())?;

    let delim = if args.delimiter.len() == 1 {
        args.delimiter.chars().next().unwrap()
    } else {
        return Err(anyhow::anyhow!("Delimiter must be a single character"));
    };

    let file = File::open(&args.file)
        .map_err(|e| anyhow::anyhow!("Cannot open '{}': {}", &args.file, e))?;

    let mut rdr = ReaderBuilder::new()
        .delimiter(delim as u8)
        .has_headers(!args.no_header)
        .from_reader(file);

    let headers = if args.no_header {
        None
    } else {
        Some(
            rdr.headers()?
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        rows.push(rec.iter().map(|s| s.to_string()).collect());
    }

    let total_rows = rows.len();
    let total_cols = headers
        .as_ref()
        .map(|h| h.len())
        .or_else(|| rows.first().map(|r| r.len()))
        .unwrap_or(0);

    print_file_info(&args.file, total_rows, total_cols);

    let table = create_table(&rows, headers.as_deref(), &args, &scheme);
    println!("{table}");

    let shown = if args.max_rows == 0 {
        total_rows
    } else {
        args.max_rows.min(total_rows)
    };
    print_footer(shown, total_rows);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datatype_detection() {
        assert!(matches!(detect_data_type(""), DataType::Empty));
        assert!(matches!(detect_data_type("false"), DataType::Boolean));
        assert!(matches!(detect_data_type("123"), DataType::Number));
        assert!(matches!(detect_data_type("2023-06-20"), DataType::Date));
        assert!(matches!(detect_data_type("hello"), DataType::Text));
    }
}
