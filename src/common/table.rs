use std::fmt::Display;
use std::io;
use std::io::Write;
use tabwriter::TabWriter;

pub struct Table<H: Display, V: Display, const COLUMNS: usize> {
    pub header: [H; COLUMNS],
    pub values: Vec<[V; COLUMNS]>,
}

#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub skip_header: bool,
    pub separator: String,
    pub tsv: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            skip_header: false,
            separator: String::from("\t"),
            tsv: false,
        }
    }
}

pub fn write<W: Write, H: Display, V: Display, const COLUMNS: usize>(
    mut writer: W,
    table: Table<H, V, COLUMNS>,
    config: &DisplayConfig,
) -> Result<(), io::Error> {
    if config.tsv {
        if !config.skip_header {
            writeln!(&mut writer, "{}", to_row(config, table.header))?;
        }

        for value in table.values {
            writeln!(&mut writer, "{}", to_row(config, value))?;
        }

        writer.flush()
    } else {
        let mut tw = TabWriter::new(writer).padding(3);

        if !config.skip_header {
            writeln!(&mut tw, "{}", to_row(config, table.header))?;
        }

        for value in table.values {
            writeln!(&mut tw, "{}", to_row(config, value))?;
        }

        tw.flush()
    }

}

fn to_row<T: Display, const COLUMNS: usize>(
    config: &DisplayConfig,
    columns: [T; COLUMNS],
) -> String {
    columns.map(|c| c.to_string()).join(&config.separator)
}
