use std::borrow::Cow;

fn bottle_phrase(n: u32, capitalize: bool) -> Cow<'static, str> {
    match (n, capitalize) {
        (0, true) => Cow::Borrowed("No more bottles"),
        (0, false) => Cow::Borrowed("no more bottles"),
        (1, _) => Cow::Borrowed("1 bottle"),
        _ => Cow::Owned(format!("{n} bottles")),
    }
}

pub fn verse(n: u32) -> String {
    let (action, next) = match n {
        0 => ("Go to the store and buy some more", 99),
        1 => ("Take it down and pass it around", 0),
        _ => ("Take one down and pass it around", n - 1),
    };

    format!(
        "{} of beer on the wall, {} of beer.\n{}, {} of beer on the wall.\n",
        bottle_phrase(n, true),
        bottle_phrase(n, false),
        action,
        bottle_phrase(next, false)
    )
}

pub fn sing(start: u32, end: u32) -> String {
    (end..=start).rev().map(verse).collect::<Vec<_>>().join("\n")
}
