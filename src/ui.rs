#![forbid(unsafe_code)]

use crate::mlp::ForwardTrace;
use crate::sim::{Car, Simulation, CAR_LENGTH, CAR_WIDTH, MAX_RAY_DIST, RAY_ANGLES};
use crate::track::Track;
use macroquad::prelude::*;

pub const SIDEBAR_WIDTH: f32 = 360.0;

pub fn render_frame(sim: &Simulation) {
    clear_background(Color::new(0.08, 0.09, 0.12, 1.0));

    // 1. Desenhar a pista de corrida
    render_track(&sim.track);

    // 2. Desenhar os carros da população
    let leader_idx = sim.get_leader_index();
    for (i, agent) in sim.agents.iter().enumerate() {
        if !agent.car.alive {
            continue;
        }

        if i == leader_idx {
            // Líder destacado em amarelo ouro / ciano
            draw_car(&agent.car, Color::new(1.0, 0.85, 0.1, 1.0));
            if sim.show_sensors {
                draw_sensors(&agent.car);
            }
        } else {
            // Demais carros semitransparentes
            draw_car(&agent.car, Color::new(0.2, 0.6, 1.0, 0.25));
        }
    }

    // 3. Desenhar Campeão Histórico se ativo
    if let Some(ref champ) = sim.champion_replay_car {
        if champ.alive {
            draw_car(champ, Color::new(1.0, 0.2, 0.9, 0.9));
        }
    }

    // 4. Desenhar Carro Humano se ativo
    if let Some(ref human) = sim.human_car {
        if human.alive {
            draw_car(human, Color::new(0.1, 1.0, 0.3, 1.0));
            // Indicador "VOCÊ" sobre o carro
            draw_text(
                "VOCÊ",
                human.pos.x - 14.0,
                human.pos.y - 14.0,
                14.0,
                Color::new(0.1, 1.0, 0.3, 1.0),
            );
        }
    }

    // 5. Desenhar o Painel Lateral Analítico (HUD)
    render_sidebar(sim, leader_idx);
}

fn render_track(track: &Track) {
    // Desenhar checkpoints como linhas pontilhadas sutis
    for (i, cp) in track.checkpoints.iter().enumerate() {
        let color = if i == 0 {
            Color::new(1.0, 1.0, 1.0, 0.8) // Linha de largada
        } else {
            Color::new(0.3, 0.35, 0.45, 0.3)
        };
        draw_line(cp.a.x, cp.a.y, cp.b.x, cp.b.y, 1.5, color);
    }

    // Desenhar as paredes internas e externas com efeito neon sutil
    for wall in &track.inner_walls {
        draw_line(
            wall.a.x,
            wall.a.y,
            wall.b.x,
            wall.b.y,
            3.0,
            Color::new(0.9, 0.2, 0.3, 0.9),
        );
    }

    for wall in &track.outer_walls {
        draw_line(
            wall.a.x,
            wall.a.y,
            wall.b.x,
            wall.b.y,
            3.0,
            Color::new(0.1, 0.5, 0.9, 0.9),
        );
    }
}

fn draw_car(car: &Car, color: Color) {
    let half_l = CAR_LENGTH / 2.0;
    let half_w = CAR_WIDTH / 2.0;

    let cos_a = car.angle.cos();
    let sin_a = car.angle.sin();

    // Vértices do corpo do carro
    let c0 = Vec2::new(
        car.pos.x + half_l * cos_a - half_w * sin_a,
        car.pos.y + half_l * sin_a + half_w * cos_a,
    );
    let c1 = Vec2::new(
        car.pos.x + half_l * cos_a + half_w * sin_a,
        car.pos.y + half_l * sin_a - half_w * cos_a,
    );
    let c2 = Vec2::new(
        car.pos.x - half_l * cos_a + half_w * sin_a,
        car.pos.y - half_l * sin_a - half_w * cos_a,
    );
    let c3 = Vec2::new(
        car.pos.x - half_l * cos_a - half_w * sin_a,
        car.pos.y - half_l * sin_a + half_w * cos_a,
    );

    // Desenha corpo
    draw_triangle(
        vec2(c0.x, c0.y),
        vec2(c1.x, c1.y),
        vec2(c2.x, c2.y),
        color,
    );
    draw_triangle(
        vec2(c2.x, c2.y),
        vec2(c3.x, c3.y),
        vec2(c0.x, c0.y),
        color,
    );

    // Farol frontal (ponto luminoso na frente)
    let front_center_x = car.pos.x + half_l * cos_a;
    let front_center_y = car.pos.y + half_l * sin_a;
    draw_circle(front_center_x, front_center_y, 2.0, WHITE);
}

fn draw_sensors(car: &Car) {
    for (i, &rel_angle) in RAY_ANGLES.iter().enumerate() {
        let angle = car.angle + rel_angle;
        let t = car.ray_readings[i];
        let ray_len = MAX_RAY_DIST * t;

        let end_x = car.pos.x + angle.cos() * ray_len;
        let end_y = car.pos.y + angle.sin() * ray_len;

        // Se t < 0.25 (muito perto da parede), cor fica vermelha viva; senão verde neon
        let col = if t < 0.25 {
            Color::new(1.0, 0.2, 0.2, 0.8)
        } else if t < 0.6 {
            Color::new(1.0, 0.8, 0.1, 0.5)
        } else {
            Color::new(0.2, 1.0, 0.4, 0.3)
        };

        draw_line(car.pos.x, car.pos.y, end_x, end_y, 1.0, col);
        draw_circle(end_x, end_y, 2.5, col);
    }
}

fn render_sidebar(sim: &Simulation, leader_idx: usize) {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let panel_x = screen_w - SIDEBAR_WIDTH;

    // Fundo do painel lateral escuro com borda divisória
    draw_rectangle(
        panel_x,
        0.0,
        SIDEBAR_WIDTH,
        screen_h,
        Color::new(0.05, 0.06, 0.08, 0.95),
    );
    draw_line(panel_x, 0.0, panel_x, screen_h, 2.0, Color::new(0.15, 0.2, 0.3, 1.0));

    let mut y = 25.0;
    let margin = panel_x + 16.0;

    // Cabeçalho
    draw_text("DARWIN RACING", margin, y, 22.0, Color::new(0.2, 0.8, 1.0, 1.0));
    y += 18.0;
    draw_text(
        &format!("Pista: {}", sim.track.name),
        margin,
        y,
        13.0,
        Color::new(0.9, 0.8, 0.3, 1.0),
    );
    y += 16.0;
    draw_text(
        "MLP + Algoritmos Genéticos (Safe Rust)",
        margin,
        y,
        13.0,
        Color::new(0.6, 0.65, 0.75, 1.0),
    );
    y += 24.0;

    // Bloco 1: Telemetria da Geração
    let alive = sim.alive_count();
    let total = sim.agents.len();
    draw_text(
        &format!("Geração: {}", sim.ga.generation),
        margin,
        y,
        17.0,
        WHITE,
    );
    y += 20.0;
    draw_text(
        &format!("Vivos: {} / {}", alive, total),
        margin,
        y,
        15.0,
        if alive > 0 { Color::new(0.3, 0.9, 0.4, 1.0) } else { RED },
    );
    y += 18.0;

    let turbo_str = match sim.turbo_multiplier {
        1 => "1x (Tempo Real 60fps)",
        5 => "5x (Rápido)",
        _ => "50x (Turbo Rayon Multi-Core)",
    };
    draw_text(&format!("Velocidade: {}", turbo_str), margin, y, 14.0, YELLOW);
    y += 18.0;

    draw_text(
        &format!("Melhor Fitness Histórico: {:.0}", sim.best_historical_fitness),
        margin,
        y,
        14.0,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );
    y += 26.0;

    // Bloco 2: Visualizador da MLP em Tempo Real (Carro Líder)
    draw_text("Cérebro do Líder (MLP)", margin, y, 16.0, Color::new(0.4, 0.8, 1.0, 1.0));
    y += 12.0;

    let leader = &sim.agents[leader_idx];
    let mut inputs = [0.0; 8];
    for i in 0..7 {
        inputs[i] = leader.car.ray_readings[i];
    }
    inputs[7] = leader.car.speed / 5.5;

    let (trace, outputs) = leader.mlp.forward_with_trace(&inputs);
    draw_mlp_graph(margin, y, SIDEBAR_WIDTH - 32.0, 140.0, &sim.topology, &trace, &leader.mlp);
    y += 150.0;

    // Indicador dos atuadores da MLP
    let steer_val = outputs[0];
    let throttle_val = outputs[1];
    draw_text(
        &format!("Volante: {:+.2} | Motor: {:+.2}", steer_val, throttle_val),
        margin,
        y,
        13.0,
        Color::new(0.8, 0.85, 0.9, 1.0),
    );
    y += 24.0;

    // Bloco 3: Gráfico de Evolução do Fitness
    draw_text("Evolução do Fitness", margin, y, 16.0, Color::new(0.4, 0.8, 1.0, 1.0));
    y += 14.0;
    draw_fitness_chart(margin, y, SIDEBAR_WIDTH - 32.0, 95.0, &sim.ga.history);
    y += 110.0;

    // Bloco 4: Controles e Atalhos
    draw_text("Controles & Atalhos", margin, y, 15.0, Color::new(0.9, 0.9, 0.9, 1.0));
    y += 18.0;

    let controls = [
        "[Espaço]: Alternar Velocidade (1x / 5x / 50x)",
        "[G]: Pular 10 Gerações Instantaneamente",
        "[1, 2, 3]: Trocar Pista (Darwin, S-Curves, GP)",
        "[H]: Modo Humano vs IA (WASD ou Setas)",
        "[R]: Alternar Raios Laser",
        "[S]: Salvar Genoma  |  [L]: Carregar Genoma",
    ];

    for c in &controls {
        draw_text(c, margin, y, 12.0, Color::new(0.65, 0.7, 0.8, 1.0));
        y += 16.0;
    }
}

fn draw_mlp_graph(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    topology: &[usize],
    trace: &ForwardTrace,
    mlp: &crate::mlp::Mlp,
) {
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.1, 0.14, 0.9));

    let num_layers = topology.len();
    let col_spacing = w / (num_layers as f32);

    // Calcular posições (X, Y) de cada neurônio
    let mut node_coords: Vec<Vec<Vec2>> = Vec::with_capacity(num_layers);

    for (l, &count) in topology.iter().enumerate() {
        let mut coords = Vec::with_capacity(count);
        let col_x = x + col_spacing * (l as f32 + 0.5);
        let row_spacing = h / (count as f32 + 1.0);

        for n in 0..count {
            let node_y = y + row_spacing * (n as f32 + 1.0);
            coords.push(Vec2::new(col_x, node_y));
        }
        node_coords.push(coords);
    }

    // 1. Desenhar conexões / pesos sinápticos entre camadas adjacentes
    for l in 0..(num_layers - 1) {
        let layer_weights = &mlp.layers[l];
        let in_count = topology[l];
        let out_count = topology[l + 1];

        for o in 0..out_count {
            for i in 0..in_count {
                let weight = layer_weights.weights[o * in_count + i];
                let from = node_coords[l][i];
                let to = node_coords[l + 1][o];

                let (col, alpha) = if weight >= 0.0 {
                    (Color::new(0.2, 0.6, 1.0, 1.0), (weight / 2.0).clamp(0.08, 0.6))
                } else {
                    (Color::new(1.0, 0.3, 0.3, 1.0), (-weight / 2.0).clamp(0.08, 0.6))
                };

                draw_line(
                    from.x,
                    from.y,
                    to.x,
                    to.y,
                    1.0,
                    Color::new(col.r, col.g, col.b, alpha),
                );
            }
        }
    }

    // 2. Desenhar neurônios com brilho proporcional à ativação
    for (l, coords) in node_coords.iter().enumerate() {
        for (n, pt) in coords.iter().enumerate() {
            let act = trace.activations[l][n];
            // Brilho verde para ativação positiva, magenta para negativa
            let node_color = if act >= 0.0 {
                Color::new(0.2, 0.8 + 0.2 * act, 0.3, 0.9)
            } else {
                Color::new(0.8 - 0.2 * act, 0.2, 0.6, 0.9)
            };

            draw_circle(pt.x, pt.y, 4.0, node_color);
            draw_circle_lines(pt.x, pt.y, 4.0, 1.0, WHITE);
        }
    }
}

fn draw_fitness_chart(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    history: &[crate::ga::GenerationStats],
) {
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.1, 0.14, 0.9));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(0.2, 0.25, 0.35, 1.0));

    if history.len() < 2 {
        draw_text(
            "Aguardando término da Geração 1...",
            x + 15.0,
            y + h / 2.0 + 4.0,
            12.0,
            GRAY,
        );
        return;
    }

    let max_fitness = history
        .iter()
        .map(|s| s.best_fitness)
        .fold(10.0_f32, |a, b| a.max(b));

    let n = history.len();
    let dx = w / (n - 1) as f32;

    // Desenhar linhas do gráfico (Melhor Fitness = Verde, Médio = Amarelo)
    for i in 0..(n - 1) {
        let s0 = &history[i];
        let s1 = &history[i + 1];

        let x0 = x + i as f32 * dx;
        let x1 = x + (i + 1) as f32 * dx;

        let y0_best = y + h - (s0.best_fitness / max_fitness) * (h - 10.0) - 5.0;
        let y1_best = y + h - (s1.best_fitness / max_fitness) * (h - 10.0) - 5.0;

        let y0_avg = y + h - (s0.avg_fitness / max_fitness) * (h - 10.0) - 5.0;
        let y1_avg = y + h - (s1.avg_fitness / max_fitness) * (h - 10.0) - 5.0;

        // Média em amarelo
        draw_line(
            x0,
            y0_avg,
            x1,
            y1_avg,
            1.5,
            Color::new(0.9, 0.8, 0.2, 0.7),
        );
        // Melhor em verde neon
        draw_line(
            x0,
            y0_best,
            x1,
            y1_best,
            2.0,
            Color::new(0.2, 1.0, 0.4, 1.0),
        );
    }

    // Legenda no canto do gráfico
    draw_circle(x + 10.0, y + 10.0, 3.0, Color::new(0.2, 1.0, 0.4, 1.0));
    draw_text("Melhor", x + 18.0, y + 13.0, 10.0, WHITE);

    draw_circle(x + 75.0, y + 10.0, 3.0, Color::new(0.9, 0.8, 0.2, 0.7));
    draw_text("Média", x + 83.0, y + 13.0, 10.0, WHITE);
}

