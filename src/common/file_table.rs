use std::fmt::Display;
use std::io;
use std::io::Write;
use tabwriter::TabWriter;

pub struct FileTable<H: Display, V: Display, const COLUMNS: usize> {
    pub header: [H; COLUMNS],
    pub values: Vec<[V; COLUMNS]>,
}

#[derive(Debug, Clone, Default)]
pub struct DisplayConfig {
    skip_header: bool,
}

pub fn write<W: Write, H: Display, V: Display, const COLUMNS: usize>(
    writer: W,
    table: FileTable<H, V, COLUMNS>,
    config: &DisplayConfig,
) -> Result<(), io::Error> {
    let mut tw = TabWriter::new(writer).padding(3);

    if !config.skip_header {
        writeln!(&mut tw, "{}", to_row(table.header))?;
    }

    for value in table.values {
        writeln!(&mut tw, "{}", to_row(value))?;
    }

    tw.flush()
}

fn to_row<T: Display, const COLUMNS: usize>(columns: [T; COLUMNS]) -> String {
    string_columns(columns).join("\t")
}

fn string_columns<T: Display, const COLUMNS: usize>(columns: [T; COLUMNS]) -> [String; COLUMNS] {
    columns.map(|item| item.to_string())
}
