pub type Bool = bool;

pub const TRUE: Bool = true;
pub const FALSE: Bool = false;

pub fn and(a: Bool, b: Bool) -> Bool {
    a && b
}

pub fn or(a: Bool, b: Bool) -> Bool {
    a || b
}

pub fn not(a: Bool) -> Bool {
    !a
}
