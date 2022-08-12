use goldfisher_web::Goldfish;

use gloo_worker::Registrable;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


fn main() {
    Goldfish::registrar().register();
}
