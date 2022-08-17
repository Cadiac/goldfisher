use goldfisher_web::Goldfish;

use gloo_worker::Registrable;

fn main() {
    Goldfish::registrar().register();
}
