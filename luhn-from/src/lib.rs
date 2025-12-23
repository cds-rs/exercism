
#[derive(Debug)]
pub struct Luhn {
    is_valid: bool,
}

/// The numeric types in the tests all implement ToString
impl<T: ToString> From<T> for Luhn {
    fn from(input: T) -> Self {
        Self {
            is_valid: is_valid_luhn(&input.to_string()),
        }
    }
}

impl Luhn {
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}


/// We're private and we're free
static DOUBLED: [u32; 10] = [0, 2, 4, 6, 8, 1, 3, 5, 7, 9];

fn is_valid_luhn(code: &str) -> bool {
    let Some(digits) = code
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_digit(10))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };

    digits.len() > 1
        && digits
            .iter()
            .rev()
            .enumerate()
            .map(|(i, &dig)| {
                if i % 2 == 1 { DOUBLED[dig as usize] } else { dig }
            })
            .sum::<u32>()
            .is_multiple_of(10)
}
