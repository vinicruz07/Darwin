#![forbid(unsafe_code)]

mod ga;
mod mlp;
mod sim;
mod track;
mod ui;

use macroquad::prelude::*;
use sim::{Car, Simulation};
use track::Track;

fn window_conf() -> Conf {
    Conf {
        window_title: "Darwin Racing — Neuroevolução (MLP + GA em Safe Rust)".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let initial_track = Track::circuit_1();
    let mut sim = Simulation::new(initial_track);

    let dt = 1.0 / 60.0;

    loop {
        // --- 1. Processamento de Teclas e Controles ---

        // Alternar multiplicador de velocidade (1x -> 5x -> 50x Turbo)
        if is_key_pressed(KeyCode::Space) {
            sim.turbo_multiplier = match sim.turbo_multiplier {
                1 => 5,
                5 => 50,
                _ => 1,
            };
        }

        // Pular 10 gerações instantaneamente
        if is_key_pressed(KeyCode::G) {
            sim.skip_generations(10);
        }

        // Alternar circuitos (1, 2 ou 3)
        if is_key_pressed(KeyCode::Key1) {
            sim.set_track(Track::circuit_1());
        } else if is_key_pressed(KeyCode::Key2) {
            sim.set_track(Track::circuit_2());
        } else if is_key_pressed(KeyCode::Key3) {
            sim.set_track(Track::circuit_3());
        }

        // Alternar feixes de sensores visíveis
        if is_key_pressed(KeyCode::R) {
            sim.show_sensors = !sim.show_sensors;
        }

        // Ativar/Desativar modo Humano vs IA
        if is_key_pressed(KeyCode::H) {
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
        if is_key_pressed(KeyCode::S) {
            if let Some(ref best_gen) = sim.best_historical_genome {
                if let Ok(json_str) = serde_json::to_string_pretty(best_gen) {
                    let _ = std::fs::write("best_genome.json", json_str);
                    println!("Melhor genoma salvo em 'best_genome.json'!");
                }
            }
        }

        // Carregar genoma salvo de 'best_genome.json'
        if is_key_pressed(KeyCode::L) {
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
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                    steer -= 1.0;
                }
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                    steer += 1.0;
                }

                let mut throttle = 0.0;
                if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                    throttle += 1.0;
                }
                if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                    throttle -= 1.0;
                }

                human.update_physics(steer, throttle, dt);
            }
        }

        // --- 2. Atualizar Simulação Físico-Genética ---
        sim.update(dt);

        // --- 3. Renderizar Gráficos na Tela ---
        ui::render_frame(&sim);

        next_frame().await;
    }
}
