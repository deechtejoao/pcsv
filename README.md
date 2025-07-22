# Pcsv – Pretty CSV Viewer


![image](images/1.jpg)

A Rust-based command-line CSV viewer that automatically detects data types and applies intelligent color coding for enhanced data visualization. PCSV transforms plain CSV files into beautifully formatted, color-coded tables that make data patterns instantly recognizable.

## Installation
```bash
git clone https://github.com/deechtejoao/pcsv
cd pcsv
cargo install --path .
# or just run in place
cargo run -- examples/data.csv
```


## Usage

```bash
# View a CSV file
pcsv file.csv

# Read from stdin
pcsv file.csv 

# Show row numbers
pcsv -s file.csv

# Limit to first 50 rows
pcsv -m 50 large_file.csv

# Use custom configuration
pcsv -c /path/to/config.toml data.csv
```
![image2](images/2.jpg) 

## Colorschemes
```bash
# Create a new directory for your custom Colorscheme 
mkdir ~/.config/pcsv/
# Create your config.toml file
touch ~/.config/pcsv/pcsv.toml
```
>  Edit file with your editor and add your color scheme, for example:
```toml
header = "#83A598"

[data_types]
text = "#EBDBB2"    
date = "#FE8019"         
float_number = "#B8BB26"   
int_number = "#83A598"     
boolean = "#FABD2F"        
empty = "#504945"          

```
---
![image3](images/3.jpg)
