#[inline]
pub fn is_hori_ws(c: u8) -> bool {
    c as char == ' ' || c as char == '\t'
}

#[inline]
pub fn is_vert_ws(c: u8) -> bool {
    c as char == '\n'
}

#[inline]
pub fn is_ws(c: u8) -> bool {
    is_vert_ws(c) || is_hori_ws(c)
}

#[inline]
pub fn is_alpha_str(st: &str) -> bool {
    for ch in st.chars() {
        if !ch.is_alphabetic() {
            return false;
        }
    }
    true
}

#[inline]
pub fn is_ident(st: &str) -> bool {
    let mut chars = st.chars();
    if let Some(ch) = chars.next() {
        if !ch.is_alphabetic() && ch != '_' {
            return false;
        }
    }
    while let Some(ch) = chars.next() {
        if !ch.is_alphanumeric() && ch != '_' {
            return false;
        }
    }
    true
}

#[inline]
pub fn is_all_num(st: &str) -> bool {
    for ch in st.chars() {
        if !ch.is_digit(10) {
            return false;
        }
    }
    true
}
