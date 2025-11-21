pub fn get_current_time() -> String {
    let now = js_sys::Date::new_0();
    format!("{:02}:{:02}", now.get_hours(), now.get_minutes())
}

pub fn format_timestamp(timestamp: i64) -> String {
    if timestamp == 0 {
        return get_current_time();
    }
    let date = js_sys::Date::new(&((timestamp as f64) * 1000.0).into());
    format!("{:02}:{:02}", date.get_hours(), date.get_minutes())
}

