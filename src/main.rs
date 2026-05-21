use std::fs;

const PATH: &str = "roms/octojam6title.ch8";

fn main() {
    let rom = fs::read(PATH).expect("failed to read ROM");
    pollster::block_on(async move {
        ch8::run(&rom).await;
    });
}
