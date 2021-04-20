use crate::array_iter;
use crossterm::style::{Print, StyledContent};
use std::{fmt::Display, io::Write};

pub fn draw_table<W, D, const N: usize>(
    mut w: W,
    headers: [&str; N],
    rows: &[[StyledContent<D>; N]],
) -> anyhow::Result<()>
where
    W: Write + Sized,
    D: Display + Clone + Length + AsRef<str>,
{
    fn calculate_hints<D, const N: usize>(
        headers: &[&str; N],
        rows: &[[D; N]],
    ) -> ([usize; N], Vec<usize>)
    where
        D: Length,
    {
        let headers = headers
            .iter()
            .enumerate()
            .fold([0; N], |mut sizes, (i, c)| {
                sizes[i] = sizes[i].max(c.len());
                sizes
            });

        let rows = rows.iter().flat_map(|row| row.iter().enumerate()).fold(
            vec![0; rows.len() + 1], // what
            |mut sizes, (i, cell)| {
                sizes[i] = sizes[i].max(cell.len());
                sizes
            },
        );

        (headers, rows)
    }

    let (header_hint, row_hint) = calculate_hints(&headers, rows);

    let hints = array_iter(header_hint)
        .zip(row_hint.iter())
        .map(|(l, r)| l.max(*r));

    let min_width = hints.clone().sum::<usize>() + header_hint.len() * 2;

    for (i, (header, hint)) in headers.iter().zip(hints).enumerate() {
        if i > 0 {
            crossterm::queue!(w, Print(" | "))?;
        }
        crossterm::queue!(w, Print(format!("{: <hint$}", header, hint = hint)))?;
    }

    crossterm::queue!(
        w,
        Print("\n"),
        Print(format!("{:->width$}", "", width = min_width)),
        Print("\n")
    )?;

    for row in rows {
        for (i, (cell, hint)) in row.iter().zip(row_hint.iter()).enumerate() {
            if i > 0 {
                crossterm::queue!(w, Print(" | "))?;
            }
            crossterm::queue!(
                w,
                Print(format!(
                    "{: <hint$}",
                    cell,
                    hint = (*hint).max(header_hint[i])
                ))
            )?;
        }
        crossterm::queue!(w, Print("\n"))?;
    }

    Ok(())
}

pub trait Length {
    fn len(&self) -> usize;
}

impl<D> Length for StyledContent<D>
where
    D: Display + Clone + AsRef<str>,
{
    fn len(&self) -> usize {
        self.content().as_ref().len()
    }
}

impl Length for String {
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl Length for &str {
    fn len(&self) -> usize {
        str::len(self)
    }
}
