use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tinygfx::color::BlackWhite::{Black, White};
use tinygfx::{Frame, Text, TextAlignment};

#[allow(unused)]
mod assets {
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

fn text_benchmark(c: &mut Criterion) {
    c.bench_function("text 20", |b| {
        b.iter(|| {
            let mut buffer = [0u8; 400 * 300 / 8];
            let mut frame = Frame::new(400, 300, |mut renderer| {
                renderer.clear(White);

                let words = black_box([
                    "river",
                    "buyer",
                    "idea",
                    "map",
                    "contract",
                    "sister",
                    "message",
                    "garbage",
                    "assignment",
                    "bread",
                ]);

                let clip = renderer.full_frame();
                for (i, w) in words.iter().enumerate() {
                    let mut text =
                        Text::new(200 + i as i32, i as i32 * 30, w, &assets::ROBOTO_30, Black);
                    text.align(TextAlignment::Center);
                    text.draw(clip, &mut renderer);
                }
            });
            frame.mirror_x(true);
            frame.mirror_y(true);
            frame.draw_part(0, &mut buffer);
            buffer
        })
    });
}

criterion_group!(benches, text_benchmark);
criterion_main!(benches);
