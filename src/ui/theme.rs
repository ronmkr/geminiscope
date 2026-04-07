use ratatui::style::Color;

pub fn get_color(color_str: &str) -> Color {
    if color_str.starts_with('#') {
        if let Ok(c) = hex_to_rgb(color_str) {
            return Color::Rgb(c.0, c.1, c.2);
        }
    }
    
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => Color::Reset,
    }
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return Err(()); }
    
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
    
    Ok((r, g, b))
}
