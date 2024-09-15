use eyre::eyre;
use math_parse::MathParseErrors;
use tesseract::Tesseract;
use tracing::debug;

fn filter_numbers_and_symbols(input: &mut String) {
    input.retain(|c| c.is_numeric() || "+-*/=".contains(c));
}

pub fn solve_captcha_math(mut text: String) -> Result<i64, MathParseErrors> {
    filter_numbers_and_symbols(&mut text);

    let text = text.trim();

    debug!("Filtered CAPTCHA text: '{}'", text);

    let math = math_parse::MathParse::parse(text)?;

    let answer = math.solve_int(None)?;

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
) -> eyre::Result<i64> {
    let mut tess = Tesseract::new(None, language)?;

    tess = match captcha_image {
        CaptchaImage::Filename(image_path) => tess.set_image(image_path)?,
        CaptchaImage::Data(image_data) => tess.set_image_from_mem(image_data)?,
    };

    // TODO: allow custom char whitelist
    tess = tess.set_variable("tessedit_char_whitelist", "Whatis0123456789+-*/? ")?;
    tess = tess.set_rectangle(left, top, width, height);

    tess = tess.recognize()?;

    let text = tess.get_text()?;

    let answer = solve_captcha_math(text).map_err(|e| eyre!("solving captcha math: {e:?}"))?;

    Ok(answer)
}
