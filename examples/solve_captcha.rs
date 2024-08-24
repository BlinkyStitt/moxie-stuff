#![feature(string_remove_matches)]

use tesseract::Tesseract;
use tracing::info;

fn filter_numbers_and_symbols(input: &mut String) {
    input.retain(|c| c.is_numeric() || "+-*/=".contains(c));
}

fn solve_captcha(image_path: &str) -> anyhow::Result<i64> {
    let left = 84;
    let top = 433;
    let width = 735;
    let height = 142;

    let config = viuer::Config {
        ..Default::default()
    };

    if let Err(e) = viuer::print_from_file(image_path, &config) {
        println!("Failed to display image: {}", e);
    }

    let mut tess = Tesseract::new(None, Some("eng"))?;
    tess = tess.set_image(image_path)?;
    tess = tess.set_variable("tessedit_char_whitelist", "Whatis0123456789+-*/? ")?;
    tess = tess.set_rectangle(left, top, width, height);

    tess = tess.recognize()?;

    let mut text = tess.get_text()?;

    println!("CAPTCHA text: '{}'", text.trim());

    filter_numbers_and_symbols(&mut text);

    let math =
        math_parse::MathParse::parse(&text).map_err(|e| anyhow::anyhow!("math parse: {:?}", e))?;

    let answer = math
        .solve_int(None)
        .map_err(|e| anyhow::anyhow!("solve int: {:?}", e))?;

    Ok(answer)
}

fn main() {
    let captcha_image = "30_plus_5.jpg";

    match solve_captcha(captcha_image) {
        Ok(solution) => println!("CAPTCHA solution: {}", solution),
        Err(e) => eprintln!("Failed to solve CAPTCHA: {}", e),
    }
}
