use alloy::primitives::{U256, utils::parse_units};
use anyhow::{Ok, Result};
use num_format::{Locale, ToFormattedString};

#[derive(Clone, Copy)]
pub enum TableAlignment {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy)]
pub enum TableBorderChar {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Horizontal,
    Vertical,
    Middle,
    TopMiddle,
    BottomMiddle,
    HorizontalLeft,
    HorizontalRight,
}

impl TableBorderChar {
    pub const fn as_char(self) -> char {
        match self {
            Self::TopLeft => '╭',
            Self::TopRight => '╮',
            Self::BottomLeft => '╰',
            Self::BottomRight => '╯',
            Self::Horizontal => '─',
            Self::Vertical => '│',
            Self::Middle => '┼',
            Self::TopMiddle => '┬',
            Self::BottomMiddle => '┴',
            Self::HorizontalLeft => '├',
            Self::HorizontalRight => '┤',
        }
    }
}

pub struct TableColumn {
    content: String,
    width: usize,
    align: TableAlignment,
}

impl TableColumn {
    pub fn new(content: impl Into<String>, width: usize) -> Self {
        Self {
            content: content.into(),
            width,
            align: TableAlignment::Left,
        }
    }

    pub fn with_alignment(mut self, align: TableAlignment) -> Self {
        self.align = align;
        self
    }
}

pub fn format_table_column(column: &TableColumn) -> String {
    let length = column.content.chars().count();

    if column.width <= length {
        return column.content.to_owned();
    }

    let padding = column.width - length;

    match column.align {
        TableAlignment::Left => format!("{}{}", column.content, " ".repeat(padding)),
        TableAlignment::Right => format!("{}{}", " ".repeat(padding), column.content),
        TableAlignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!(
                "{}{}{}",
                " ".repeat(left),
                column.content,
                " ".repeat(right)
            )
        }
    }
}

pub fn format_table_row(columns: &[TableColumn]) -> String {
    let row = columns
        .iter()
        .map(|c| format!(" {} ", format_table_column(c)))
        .collect::<Vec<String>>()
        .join(&TableBorderChar::Vertical.as_char().to_string());
    format!(
        "{}{}{}",
        TableBorderChar::Vertical.as_char(),
        row,
        TableBorderChar::Vertical.as_char(),
    )
}

pub fn format_table_header(columns: &[TableColumn]) -> String {
    let top_border = columns
        .iter()
        .map(|c| {
            TableBorderChar::Horizontal
                .as_char()
                .to_string()
                .repeat(c.width + 2)
        })
        .collect::<Vec<String>>()
        .join(&TableBorderChar::TopMiddle.as_char().to_string());

    let header = format_table_row(columns);

    let separator = columns
        .iter()
        .map(|c| {
            TableBorderChar::Horizontal
                .as_char()
                .to_string()
                .repeat(c.width + 2)
        })
        .collect::<Vec<String>>()
        .join(&TableBorderChar::Middle.as_char().to_string());

    format!(
        "{}{}{}\n{}\n{}{}{}",
        TableBorderChar::TopLeft.as_char(),
        top_border,
        TableBorderChar::TopRight.as_char(),
        header,
        TableBorderChar::HorizontalLeft.as_char(),
        separator,
        TableBorderChar::HorizontalRight.as_char()
    )
}

pub fn format_table_footer(columns: &[TableColumn]) -> String {
    let bottom_border = columns
        .iter()
        .map(|c| {
            TableBorderChar::Horizontal
                .as_char()
                .to_string()
                .repeat(c.width + 2)
        })
        .collect::<Vec<String>>()
        .join(&TableBorderChar::BottomMiddle.as_char().to_string());
    format!(
        "{}{}{}",
        TableBorderChar::BottomLeft.as_char(),
        bottom_border,
        TableBorderChar::BottomRight.as_char()
    )
}

pub struct TableColumnWidths {
    pub id: usize,
    pub name: usize,
    pub public_key: usize,
    pub collateral_balance: usize,
    pub usd_balance: usize,
    pub token_balance: usize,
    // pub symbol: usize,
    // pub allocation: usize,
    // pub parameter: usize,
    // pub entry_mcap: usize,
}

pub fn to_wei(amount: &str) -> Result<U256> {
    Ok(parse_units(amount, 18)?.into())
}

pub fn to_token_units(amount: &str, decimals: u8) -> Result<U256> {
    Ok(parse_units(amount, decimals)?.into())
}

pub fn format_currency(value: f64) -> String {
    let int_part = value.trunc() as i64;
    let frac_part = (value.fract().abs() * 100.0).round() as u64;

    format!(
        "{}.{:02}",
        int_part.to_formatted_string(&Locale::en),
        frac_part
    )
}
