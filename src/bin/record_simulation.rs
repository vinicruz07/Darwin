use darwin_racing::canvas::Canvas;
use darwin_racing::sim::Simulation;
use darwin_racing::track::Track;
use darwin_racing::ui;
use std::fs::File;
use std::io::Write;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const DOWNSCALE: usize = 2; // Reduz para 640x360 para gerar um GIF leve e super fluido

fn main() {
    println!(">>> Gerando gravacao da simulacao...");

    let track = Track::circuit_1(); // Interlagos
    let mut sim = Simulation::new(track);
    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let dt = 1.0 / 60.0;
    let target_frames = 300; // ~10 segundos a 30 FPS ou 5 segundos a 60 FPS
    let step_per_frame = 2;  // Captura 1 a cada 2 ticks físicos

    // Cria diretório temporário para os frames raw
    std::fs::create_dir_all("target/gif_frames").unwrap();
    let mut raw_file = File::create("target/gif_frames/frames.raw").unwrap();

    let mut frame_count = 0;

    for tick in 0..(target_frames * step_per_frame) {
        sim.step_single_tick(dt);

        if sim.alive_count() == 0 {
            sim.end_generation_and_evolve();
        }

        if tick % step_per_frame == 0 {
            ui::render_frame(&mut canvas, &sim);

            // Reduz a resolução para 640x360 em RGB para o GIF
            let out_w = WIDTH / DOWNSCALE;
            let out_h = HEIGHT / DOWNSCALE;
            let mut rgb_buffer = Vec::with_capacity(out_w * out_h * 3);

            for y in 0..out_h {
                let src_y = y * DOWNSCALE;
                for x in 0..out_w {
                    let src_x = x * DOWNSCALE;
                    let pixel = canvas.buffer[src_y * WIDTH + src_x];
                    let r = ((pixel >> 16) & 0xFF) as u8;
                    let g = ((pixel >> 8) & 0xFF) as u8;
                    let b = (pixel & 0xFF) as u8;
                    rgb_buffer.push(r);
                    rgb_buffer.push(g);
                    rgb_buffer.push(b);
                }
            }

            raw_file.write_all(&rgb_buffer).unwrap();
            frame_count += 1;
            if frame_count % 50 == 0 {
                println!("  Capturados {}/{} frames...", frame_count, target_frames);
            }
        }
    }

    println!(">>> Gravacao concluida: {} frames gerados com sucesso!", frame_count);
}
