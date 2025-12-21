#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInputBase,
    InvalidOutputBase,
    InvalidDigit(u32),
}

fn to_decimal(digits: &[u32], base: u32) -> Result<u32, Error> {
    if base < 2 {
        return Err(Error::InvalidInputBase);
    }

    let mut value = 0;
    for digit in digits.iter().skip_while(|&d| *d == 0) {
        if *digit >= base {
            return Err(Error::InvalidDigit(*digit));
        }
        value = value * base + digit;
    }
    Ok(value)
}

fn from_decimal(mut number: u32, base: u32) -> Vec<u32> {
    let mut res = Vec::new();
    loop {
        let (q, r) = (number / base, number % base);
        res.push(r);
        number = q;
        if number < base {
            if number > 0 {
                res.push(number);
            }
            break;
        }
    }
    res.into_iter().rev().collect()
}

pub fn convert(number: &[u32], from_base: u32, to_base: u32) -> Result<Vec<u32>, Error> {
    if to_base < 2 {
        return Err(Error::InvalidOutputBase);
    }
    let decimal_value = to_decimal(number, from_base)?;
    Ok(from_decimal(decimal_value, to_base))
}
