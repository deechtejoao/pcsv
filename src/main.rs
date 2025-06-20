use anyhow::Result;
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use crossterm::style::{Color as CrosstermColor, Stylize};
use csv::ReaderBuilder;
use regex::Regex;
use serde::Deserialize;
use std::{fs, fs::File};

#[derive(Debug, Deserialize)]
struct RGB(u8, u8, u8);

#[derive(Debug, Deserialize)]
struct ColorScheme {
    dark0: RGB,
    dark1: RGB,
    dark2: RGB,
    dark3: RGB,
    light0: RGB,
    light1: RGB,
    light2: RGB,
    bright_green: RGB,
    bright_aqua: RGB,
    bright_blue: RGB,
    neutral_blue: RGB,
    bright_red: RGB,
    bright_orange: RGB,
    bright_yellow: RGB,
    bright_purple: RGB,
    neutral_purple: RGB,
}

impl ColorScheme {
    fn rgb(rgb: &RGB) -> Color {
        Color::Rgb {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
        }
    }
    fn cell_color(&self, ty: &DataType) -> Color {
        match ty {
            DataType::Number => Self::rgb(&self.bright_green),
            DataType::Boolean => Self::rgb(&self.bright_yellow),
            DataType::Date => Self::rgb(&self.bright_orange),
            DataType::Empty => Self::rgb(&self.dark3),
            DataType::Text => Self::rgb(&self.light1),
        }
    }
}

fn load_scheme() -> Result<ColorScheme> {
    Ok(ColorScheme {
        dark0: RGB(40, 40, 40),
        dark1: RGB(60, 60, 60),
        dark2: RGB(80, 80, 80),
        dark3: RGB(100, 100, 100),
        light0: RGB(200, 200, 200),
        light1: RGB(180, 180, 180),
        light2: RGB(160, 160, 160),
        bright_green: RGB(0, 255, 0),
        bright_aqua: RGB(0, 255, 255),
        bright_blue: RGB(0, 0, 255),
        neutral_blue: RGB(100, 100, 200),
        bright_red: RGB(255, 0, 0),
        bright_orange: RGB(255, 165, 0),
        bright_yellow: RGB(255, 255, 0),
        bright_purple: RGB(128, 0, 128),
        neutral_purple: RGB(147, 112, 219),
    })
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
            cells.push(Cell::new("#").fg(ColorScheme::rgb(&scheme.light0)));
        }
        for name in h {
            cells.push(Cell::new(name).fg(ColorScheme::rgb(&scheme.light0)));
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
            cells.push(Cell::new((idx + 1).to_string()).fg(ColorScheme::rgb(&scheme.bright_aqua)));
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
    let scheme = load_scheme()?;

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
