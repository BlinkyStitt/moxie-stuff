use tesseract::Tesseract;
use tracing::debug;

fn filter_numbers_and_symbols(input: &mut String) {
    input.retain(|c| c.is_numeric() || "+-*/=".contains(c));
}

pub fn solve_captcha_math(mut text: String) -> anyhow::Result<i64> {
    filter_numbers_and_symbols(&mut text);

    let text = text.trim();

    debug!("CAPTCHA text: '{}'", text);

    let math =
        math_parse::MathParse::parse(text).map_err(|e| anyhow::anyhow!("math parse: {:?}", e))?;

    let answer = math
        .solve_int(None)
        .map_err(|e| anyhow::anyhow!("solve int: {:?}", e))?;

    Ok(answer)
}

pub enum CaptchaImage<'a> {
    Filename(&'a str),
    Data(&'a [u8]),
}

pub fn solve_captcha(
    captcha_image: &CaptchaImage,
    language: Option<&str>,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> anyhow::Result<i64> {
    let oem = tesseract::OcrEngineMode::TesseractOnly;

    let mut tess = match captcha_image {
        CaptchaImage::Filename(image_path) => {
            Tesseract::new_with_oem(Some(image_path), language, oem)?
        }
        CaptchaImage::Data(image_data) => Tesseract::new_with_data(image_data, language, oem)?,
    };

    tess = tess.set_variable("tessedit_char_whitelist", "Whatis0123456789+-*/? ")?;
    tess = tess.set_rectangle(left, top, width, height);

    tess = tess.recognize()?;

    let text = tess.get_text()?;

    let answer = solve_captcha_math(text)?;

    Ok(answer)
}
