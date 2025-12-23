#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInputBase,
    InvalidOutputBase,
    InvalidDigit(u32),
}

fn to_decimal(digits: &[u32], base: u32) -> Result<u32, Error> {
    //          digits: &[u32]    -- shared borrow, R only
    //          base: u32         -- owned (Copy), O R W

    if base < 2 {
        return Err(Error::InvalidInputBase);
    }

    let mut value = 0;                              // value: O R W
    for digit in digits.iter().skip_while(|&d| *d == 0) {
        // digits.iter(): creates iterator borrowing digits
        // |&d| *d == 0: closure takes &u32, &d pattern copies u32 out (Copy)
        // digit: &u32  -- shared borrow of slice element

        if *digit >= base {                           // *digit: deref to read u32
            return Err(Error::InvalidDigit(*digit));  // *digit: copied (Copy)
        }
        value = value * base + digit;              // digit: auto-deref to u32, all Copy
    }
    Ok(value)                                      // value: moved into Ok (Copy)
}

fn from_decimal(mut number: u32, base: u32) -> Result<Vec<u32>, Error> {
    //           number: u32      -- owned (Copy), O R W; `mut` allows reassignment
    //           base: u32        -- owned (Copy), O R W

    if base < 2 {
        return Err(Error::InvalidOutputBase);
    }

    let mut res = Vec::new();            // res: O R W (owns heap allocation)
    loop {
        let (q, r) = (number / base, number % base); // q, r: new u32 values (Copy)
        res.push(r);                               // r: copied into Vec (Copy)
        number = q;                                // q: copied into number (Copy)
        if number < base {
            if number > 0 {
                res.push(number);                  // number: copied into Vec (Copy)
            }
            break;
        }
    }
    Ok(res.into_iter().rev().collect())            // res: moved into iterator, consumed
}                                                  // res already moved; nothing to drop

pub fn convert(number: &[u32], from_base: u32, to_base: u32) -> Result<Vec<u32>, Error> {
    //          number: &[u32]    -- shared borrow, R only
    //          from_base: u32    -- owned (Copy), O R W
    //          to_base: u32      -- owned (Copy), O R W

    let decimal_value = to_decimal(number, from_base)?;
    // number: reborrow (same & passed through)
    // from_base: copied (Copy)
    // decimal_value: O R W (owns returned u32)
    // ?: early returns Err if to_decimal fails, otherwise unwraps Ok

    from_decimal(decimal_value, to_base)
    // decimal_value: copied (Copy)
    // to_base: copied (Copy)
    // Result<Vec<u32>, Error>: moved to caller
}
