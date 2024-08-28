use tesseract::Tesseract;

fn filter_numbers_and_symbols(input: &mut String) {
    input.retain(|c| c.is_numeric() || "+-*/=".contains(c));
}

pub fn solve_captcha(
    image_path: &str,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> anyhow::Result<i64> {
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
