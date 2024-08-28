use moxie_stuff::solve_captcha;

fn main() {
    let captcha_path = "30_plus_5.jpg";
    let left = 84;
    let top = 433;
    let width = 735;
    let height = 142;

    let config = viuer::Config {
        ..Default::default()
    };

    if let Err(e) = viuer::print_from_file(captcha_path, &config) {
        println!("Failed to display image: {}", e);
    }

    match solve_captcha(captcha_path, left, top, width, height) {
        Ok(solution) => println!("CAPTCHA solution: {}", solution),
        Err(e) => eprintln!("Failed to solve CAPTCHA: {}", e),
    }
}
