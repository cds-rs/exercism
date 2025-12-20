#![allow(dead_code)]
#[derive(Debug)]
struct Sentiment<'a> {
    phrase: &'a str,
    is_question: bool,
    is_yelling: bool,
    is_silent: bool,
}

impl<'a> Sentiment<'a> {
    pub fn new(phrase: &'a str) -> Self {
        Self {
            phrase,
            is_silent: phrase.chars().all(|c| c.is_whitespace()),
            is_question: phrase.trim().ends_with('?'),
            is_yelling: {
                let has_letters = phrase.chars().any(|c| c.is_ascii_alphabetic());
                let all_uppercase = phrase.chars().filter(|c| c.is_alphabetic())
                                          .all(|c| c.is_ascii_uppercase());
                has_letters && all_uppercase
            }
        }
    }
}



pub fn reply(message: &str) -> &str {
    let s = Sentiment::new(message);
    dbg!(&s);
    match (s.is_silent, s.is_question, s.is_yelling) {
        (false, true, false) => "Sure.",
        (false, false, true) => "Whoa, chill out!",
        (false, true, true) => "Calm down, I know what I'm doing!",
        (false, false, false) => "Whatever.",
        (_, _, _) => "Fine. Be that way!",
    }
}
