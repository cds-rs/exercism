#[derive(Debug)]
struct Sentiment {
    is_question: bool,
    is_yelling: bool,
    is_silent: bool,
}

impl Sentiment {

    pub fn new(phrase: &str) -> Self {
        let mut has_alpha = false;
        let mut has_lower = false;
        let mut has_non_ws = false;
        let mut last_non_ws: Option<char> = None;

        // Single pass
        for c in phrase.chars() {
            if !c.is_whitespace() {
                has_non_ws = true;
                last_non_ws = Some(c);
            }
            if c.is_ascii_alphabetic() {
                has_alpha = true;
                has_lower |= c.is_ascii_lowercase();
            }
        }

        Self {
            is_silent: !has_non_ws,
            is_question: last_non_ws == Some('?'),
            is_yelling: has_alpha && !has_lower,
        }
    }
}



pub fn reply(message: &str) -> &str {
    let s = Sentiment::new(message);
    match (s.is_silent, s.is_question, s.is_yelling) {
        (false, true, false) => "Sure.",
        (false, false, true) => "Whoa, chill out!",
        (false, true, true) => "Calm down, I know what I'm doing!",
        (false, false, false) => "Whatever.",
        _ => "Fine. Be that way!",
    }
}
