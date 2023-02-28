use engine;
fn main() {
    env_logger::init();
    pollster::block_on(engine::App::run());
}
