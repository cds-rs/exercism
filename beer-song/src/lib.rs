use std::borrow::Cow;

fn bottle_phrase(n: u32) -> Cow<'static, str> {
    match n {
        0 => Cow::Borrowed("no more bottles"),
        1 => Cow::Borrowed("1 bottle"),
        _ => Cow::Owned(format!("{n} bottles")),
    }
}

pub fn verse(n: u32) -> String {
    let (action, next) = match n {
        0 => ("Go to the store and buy some more", 99),
        1 => ("Take it down and pass it around", 0),
        _ => ("Take one down and pass it around", n - 1),
    };

    let bottles = bottle_phrase(n);
    let remaining = bottle_phrase(next);

    match n {
        0 => format!( "No more bottles of beer on the wall, {bottles} of beer.\n{action}, {remaining} of beer on the wall.\n"),
        _ => format!( "{bottles} of beer on the wall, {bottles} of beer.\n{action}, {remaining} of beer on the wall.\n"),
    }
}

pub fn sing(start: u32, end: u32) -> String {
    (end..=start)
        .rev()
        .map(verse)
        .collect::<Vec<_>>()
        .join("\n")
}
