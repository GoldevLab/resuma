//! In-memory bookings for the flow-booking template.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;

pub const SERVICES: &[(&str, &str)] = &[("cut", "Haircut"), ("beard", "Beard trim")];

pub const SLOTS: &[&str] = &["10:00", "11:00", "12:00", "16:00", "17:00"];

static TAKEN: Lazy<Mutex<HashSet<(String, String)>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn available_slots(date: &str) -> Vec<&'static str> {
    if date.is_empty() {
        return vec![];
    }
    let taken = TAKEN.lock();
    SLOTS
        .iter()
        .copied()
        .filter(|t| !taken.contains(&(date.to_string(), (*t).to_string())))
        .collect()
}

pub fn book(
    name: String,
    phone: String,
    service: String,
    date: String,
    time: String,
) -> Result<(), (&'static str, String)> {
    if name.trim().len() < 2 {
        return Err(("name", "Enter your name".into()));
    }
    if phone.trim().len() < 6 {
        return Err(("phone", "Enter a valid phone".into()));
    }
    if !SERVICES.iter().any(|(k, _)| *k == service) {
        return Err(("service", "Pick a service".into()));
    }
    if date.is_empty() || time.is_empty() {
        return Err(("time", "Pick date and time".into()));
    }
    let key = (date.clone(), time.clone());
    let mut taken = TAKEN.lock();
    if !taken.insert(key) {
        return Err(("time", "That slot was just taken".into()));
    }
    let _ = (name, phone, service);
    Ok(())
}
