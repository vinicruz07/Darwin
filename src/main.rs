#![forbid(unsafe_code)]

mod canvas;
mod ga;
mod mlp;
mod sim;
mod track;
mod ui;

use canvas::Canvas;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use sim::{Car, Simulation};
use track::Track;

const WINDOW_WIDTH: usize = 1280;
const WINDOW_HEIGHT: usize = 720;

fn main() {
    let initial_track = Track::circuit_1();
    let mut sim = Simulation::new(initial_track);
    let mut canvas = Canvas::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut window = Window::new(
        "Darwin Racing - Neuroevolucao (MLP + GA em Safe Rust)",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .expect("Falha ao inicializar janela Win32 GDI");

    // Limita a 60 FPS
    window.set_target_fps(60);

    let dt = 1.0 / 60.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- 1. Processamento de Teclas e Controles ---

        // Alternar multiplicador de velocidade (1x -> 5x -> 50x Turbo)
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            sim.turbo_multiplier = match sim.turbo_multiplier {
                1 => 5,
                5 => 50,
                _ => 1,
            };
        }

        // Pular 10 gerações instantaneamente
        if window.is_key_pressed(Key::G, KeyRepeat::No) {
            sim.skip_generations(10);
        }

        // Alternar circuitos (1, 2 ou 3)
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            sim.set_track(Track::circuit_1());
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            sim.set_track(Track::circuit_2());
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            sim.set_track(Track::circuit_3());
        }

        // Alternar feixes de sensores visíveis
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            sim.show_sensors = !sim.show_sensors;
        }

        // Ativar/Desativar modo Humano vs IA
        if window.is_key_pressed(Key::H, KeyRepeat::No) {
            if sim.human_car.is_some() {
                sim.human_car = None;
                sim.champion_replay_car = None;
            } else {
                sim.human_car = Some(Car::new(sim.track.spawn_pos, sim.track.spawn_angle));
                if sim.best_historical_genome.is_some() {
                    sim.champion_replay_car = Some(Car::new(sim.track.spawn_pos, sim.track.spawn_angle));
                }
            }
        }

        // Salvar melhor genoma em arquivo JSON
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            if let Some(ref best_gen) = sim.best_historical_genome {
                if let Ok(json_str) = serde_json::to_string_pretty(best_gen) {
                    let _ = std::fs::write("best_genome.json", json_str);
                    println!("Melhor genoma salvo em 'best_genome.json'!");
                }
            }
        }

        // Carregar genoma salvo de 'best_genome.json'
        if window.is_key_pressed(Key::L, KeyRepeat::No) {
            if let Ok(json_str) = std::fs::read_to_string("best_genome.json") {
                if let Ok(loaded_gen) = serde_json::from_str::<ga::Genome>(&json_str) {
                    sim.best_historical_genome = Some(loaded_gen.clone());
                    // Injeta o genoma carregado como elite 0 da população atual
                    sim.ga.population[0] = loaded_gen;
                    sim.restart_generation();
                    println!("Genoma carregado com sucesso!");
                }
            }
        }

        // Controle do Carro do Jogador Humano (WASD ou Setas)
        if let Some(ref mut human) = sim.human_car {
            if human.alive {
                let mut steer = 0.0;
                if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
                    steer -= 1.0;
                }
                if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
                    steer += 1.0;
                }

                let mut throttle = 0.0;
                if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
                    throttle += 1.0;
                }
                if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
                    throttle -= 1.0;
                }

                human.update_physics(steer, throttle, dt);
            }
        }

        // --- 2. Atualizar Simulação Físico-Genética ---
        sim.update(dt);

        // --- 3. Renderizar Gráficos no Framebuffer em Software ---
        ui::render_frame(&mut canvas, &sim);

        // --- 4. Enviar Framebuffer diretamente para a Janela Win32 GDI ---
        if let Err(e) = window.update_with_buffer(&canvas.buffer, WINDOW_WIDTH, WINDOW_HEIGHT) {
            eprintln!("Erro ao atualizar janela: {:?}", e);
            break;
        }
    }
}
