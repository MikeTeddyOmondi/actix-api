use intasend::Intasend;

pub struct Payments;

impl Payments {
    pub fn init() -> Intasend {
        let intasend_public_key =
            std::env::var("INTASEND_PUBLIC_KEY").expect("INTASEND_PUBLIC_KEY must be set");
        let intasend_secret_key =
            std::env::var("INTASEND_SECRET_KEY").expect("INTASEND_SECRET_KEY must be set");

        Intasend::new(intasend_public_key, intasend_secret_key, true)
    }
}
