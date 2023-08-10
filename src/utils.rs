use to_precision::FloatExt;

pub fn format_price(price: f64) -> String {
    match price {
        price if price > 1_000_000_000_000.0 => format!("${:.2}T", price / 1_000_000_000_000.0),
        price if price > 1_000_000_000.0 => format!("${:.2}B", price / 1_000_000_000.0),
        price if price > 1_000_000.0 => format!("${:.2}M", price / 1_000_000.0),
        price if price > 100_000.0 => format!("${:.2}K", price / 1_000.0),
        price if price > 1.0 => format!("${:.2}", price),
        _ => format!("${}", price.to_precision(5)),
    }
}
