use std::fmt;

use tabled::{builder::Builder as TableBuilder, settings::Style};

pub trait TableFormattable<C> {
    fn get_cell_value_by_column(&self, column: &C) -> String;
}

pub trait PrettyFormatter<I, C>
where
    I: TableFormattable<C>,
    for<'a> &'a C: Into<String>,
{
    fn to_pretty(self, columns: &[C]) -> impl fmt::Display
    where
        Self: Iterator<Item = I> + Sized,
    {
        let listing = self.map(|i| {
            columns
                .iter()
                .map(|c| i.get_cell_value_by_column(c))
                .collect::<Vec<String>>()
        });

        let mut builder = TableBuilder::new();

        builder.push_record(columns);

        for row in listing {
            builder.push_record(row);
        }

        let mut table = builder.build();
        table.with(Style::blank());

        table
    }
}

impl<I, T, C> PrettyFormatter<I, C> for T
where
    I: TableFormattable<C>,
    T: Iterator<Item = I>,
    for<'a> &'a C: Into<String>,
{
}

pub trait TerseFormatter<I, C>
where
    I: TableFormattable<C>,
{
    fn to_terse(self, columns: &[C]) -> impl fmt::Display
    where
        Self: Iterator<Item = I> + Sized,
    {
        let output = self
            .map(|i| {
                let mut values = columns
                    .iter()
                    .map(|c| i.get_cell_value_by_column(c))
                    .collect::<Vec<String>>()
                    .join("/");
                values.push('\n');
                values
            })
            .collect::<String>();

        output
    }
}

impl<I, T, C> TerseFormatter<I, C> for T
where
    I: TableFormattable<C>,
    T: Iterator<Item = I>,
    for<'a> &'a C: Into<String>,
{
}
