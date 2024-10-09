use std::{u16, u8};

use super::Value;

pub fn operate<F>(pair: Option<(&Value, &Value)>, f: F) -> Option<u64>
    where F: Fn(u64, u64) -> u64 {
    if let Some(pair) = pair {
        if let Value::Number(first) = pair.0  {
            if let Value::Number(second) = pair.1 {
                return Some(f(first.clone(), second.clone()));
            }
        }
    }
    None
}

pub fn cond_sign(value: bool) -> u64 {
    if value {
        1
    } else {
        0
    }
}

pub fn get_u8(slice: &[u8]) -> Option<u8> {
    Some(u8::from_be_bytes(slice.try_into().ok()?))
}

pub fn get_u16(slice: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(slice.try_into().ok()?))
}

// pub fn get_u32(slice: &[u8]) -> Option<u32> {
//     Some(u32::from_be_bytes(slice.try_into().ok()?))
// }

pub fn get_u64(slice: &[u8]) -> Option<u64> {
    Some(u64::from_be_bytes(slice.try_into().ok()?))
}

pub enum Direction {
    Forward,
    Backward,
}

pub fn get_relative_reference(reference: u16) -> (Direction, u16) {
    if reference & (0b1 << 15) != 0 {
        (Direction::Backward, reference ^ (0b1 << 15))
    } else {
        (Direction::Forward, reference)
    }
}
