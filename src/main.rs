mod register;

use register::register;

fn main() {
    register(
        "https://www.bklynlibrary.org/calendar/monthly-live-music-brooklyn-heights-library-20230609".to_string(),
        "Joshua".to_string(),
        "Hopkins".to_string(),
        "laova0154v@kzccv.com".to_string(),
    )
    .unwrap();
}
