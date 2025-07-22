use clap::Parser;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use config::{load_config, ColorScheme};
use regex::Regex;
use std::fs;
use std::io::{self, Read};
use std::sync::OnceLock;

mod config;

#[derive(Debug, Clone)]
enum DataType {
    Text,
    IntNumber,
    FloatNumber,
    Boolean,
    Date,
    Empty,
}

impl ColorScheme {
    fn hex_to_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb { r, g, b } // Changed from Color::Rgb(r, g, b)
    }

    fn cell_color(&self, ty: &DataType) -> Color {
        let hex = match ty {
            DataType::IntNumber => &self.data_types.int_number,
            DataType::FloatNumber => &self.data_types.float_number,
            DataType::Boolean => &self.data_types.boolean,
            DataType::Date => &self.data_types.date,
            DataType::Empty => &self.data_types.empty,
            DataType::Text => &self.data_types.text,
        };
        Self::hex_to_color(hex)
    }

    fn header_color(&self) -> Color {
        Self::hex_to_color(&self.header)
    }
}

#[derive(Parser)]
#[command(name = "csv-viewer")]
#[command(about = "A colorful CSV viewer")]
struct Args {
    input: String,

    #[arg(short, long)]
    show_row_numbers: bool,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    max_rows: Option<usize>,
}

static DATA_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn init_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(), // YYYY-MM-DD
        Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap(), // MM/DD/YYYY
        Regex::new(r"^\d{2}-\d{2}-\d{4}$").unwrap(), // MM-DD-YYYY
        Regex::new(r"^\d{4}/\d{2}/\d{2}$").unwrap(), // YYYY/MM/DD
        Regex::new(r"^\d{1,2}/\d{1,2}/\d{4}$").unwrap(), // M/D/YYYY
        Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap(), // YYYY-MM-DD HH:MM:SS
    ]
}

fn detect_data_type_cached(val: &str) -> DataType {
    let patterns = DATA_PATTERNS.get_or_init(|| init_patterns());
    for pattern in patterns {
        if pattern.is_match(val) {
            return DataType::Date;
        }
    }

    if val.trim().is_empty() {
        return DataType::Empty;
    }

    match val.to_lowercase().as_str() {
        "true" | "false" | "yes" | "no" | "y" | "n" => DataType::Boolean,
        _ => {
            if let Ok(_num) = val.parse::<f64>() {
                if val.contains('.') || val.to_lowercase().contains('e') {
                    DataType::FloatNumber
                } else if val.parse::<i64>().is_ok() {
                    DataType::IntNumber
                } else {
                    DataType::FloatNumber
                }
            } else {
                DataType::Text
            }
        }
    }
}

fn read_csv_data(
    input: &str,
) -> Result<(Option<Vec<String>>, Vec<Vec<String>>), Box<dyn std::error::Error>> {
    let content = if input == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(input)?
    };

    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let headers = if rdr.has_headers() {
        Some(rdr.headers()?.iter().map(|s| s.to_string()).collect())
    } else {
        None
    };

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(record.iter().map(|s| s.to_string()).collect());
    }

    Ok((headers, records))
}

fn create_table(
    headers: Option<Vec<String>>,
    records: Vec<Vec<String>>,
    scheme: &ColorScheme,
    args: &Args,
) -> Table {
    let mut table = Table::new();

    table.load_preset(UTF8_FULL);
    // Set headers with colors
    if let Some(h) = headers {
        let header_cells: Vec<Cell> = if args.show_row_numbers {
            std::iter::once(Cell::new("#").fg(scheme.header_color()))
                .chain(
                    h.iter()
                        .map(|name| Cell::new(name).fg(scheme.header_color())),
                )
                .collect()
        } else {
            h.iter()
                .map(|name| Cell::new(name).fg(scheme.header_color()))
                .collect()
        };
        table.set_header(header_cells);
    }

    let limited_records = if let Some(max) = args.max_rows {
        records.into_iter().take(max).collect::<Vec<_>>()
    } else {
        records
    };

    for (row_idx, record) in limited_records.iter().enumerate() {
        let mut row_cells = Vec::new();

        if args.show_row_numbers {
            row_cells.push(Cell::new(&format!("{}", row_idx + 1)).fg(scheme.header_color()));
        }

        for value in record {
            let data_type = detect_data_type_cached(value);
            let color = scheme.cell_color(&data_type);
            row_cells.push(Cell::new(value).fg(color));
        }

        table.add_row(row_cells);
    }

    table
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let scheme = load_config(args.config.as_deref());
    let (headers, records) = read_csv_data(&args.input)?;
    let table = create_table(headers, records, &scheme, &args);

    println!("{}", table);
    Ok(())
}
