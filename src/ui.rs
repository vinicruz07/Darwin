#![forbid(unsafe_code)]

use crate::canvas::{Canvas, Color};
use crate::mlp::ForwardTrace;
use crate::sim::{Car, Simulation, CAR_LENGTH, CAR_WIDTH, MAX_RAY_DIST, RAY_ANGLES};
use crate::track::{Track, Vec2};

pub const SIDEBAR_WIDTH: f32 = 360.0;

pub fn render_frame(canvas: &mut Canvas, sim: &Simulation) {
    // 1. Limpar fundo da tela com cinza grafite escuro
    canvas.clear(Color::rgb(20, 23, 30));

    // 2. Desenhar a pista
    render_track(canvas, &sim.track);

    // 3. Desenhar a população de carros
    let leader_idx = sim.get_leader_index();
    for (i, agent) in sim.agents.iter().enumerate() {
        if !agent.car.alive {
            continue;
        }

        if i == leader_idx {
            // Líder destacado em amarelo ouro vivo
            draw_car(canvas, &agent.car, Color::rgb(255, 215, 25));
            if sim.show_sensors {
                draw_sensors(canvas, &agent.car);
            }
        } else {
            // Demais carros semitransparentes em azul ciano suave
            draw_car(canvas, &agent.car, Color::rgba(50, 150, 255, 60));
        }
    }

    // 4. Desenhar Campeão Histórico se ativo
    if let Some(ref champ) = sim.champion_replay_car {
        if champ.alive {
            draw_car(canvas, champ, Color::rgba(255, 50, 220, 220));
        }
    }

    // 5. Desenhar Carro Humano se ativo
    if let Some(ref human) = sim.human_car {
        if human.alive {
            draw_car(canvas, human, Color::rgb(30, 255, 80));
            canvas.draw_text("VOCE", human.pos.x - 16.0, human.pos.y - 18.0, 1, Color::rgb(30, 255, 80));
        }
    }

    // 6. Desenhar Painel Lateral Analítico
    render_sidebar(canvas, sim, leader_idx);
}

fn render_track(canvas: &mut Canvas, track: &Track) {
    // Checkpoints sutis
    for (i, cp) in track.checkpoints.iter().enumerate() {
        let color = if i == 0 {
            Color::rgba(255, 255, 255, 200) // Largada
        } else {
            Color::rgba(80, 90, 115, 80)
        };
        canvas.draw_line(cp.a.x, cp.a.y, cp.b.x, cp.b.y, 1.0, color);
    }

    // Paredes internas (vermelho neon suave)
    for wall in &track.inner_walls {
        canvas.draw_line(wall.a.x, wall.a.y, wall.b.x, wall.b.y, 3.0, Color::rgb(230, 50, 75));
    }

    // Paredes externas (azul elétrico suave)
    for wall in &track.outer_walls {
        canvas.draw_line(wall.a.x, wall.a.y, wall.b.x, wall.b.y, 3.0, Color::rgb(30, 130, 230));
    }
}

fn draw_car(canvas: &mut Canvas, car: &Car, color: Color) {
    let half_l = CAR_LENGTH / 2.0;
    let half_w = CAR_WIDTH / 2.0;

    let cos_a = car.angle.cos();
    let sin_a = car.angle.sin();

    let c0 = (
        car.pos.x + half_l * cos_a - half_w * sin_a,
        car.pos.y + half_l * sin_a + half_w * cos_a,
    );
    let c1 = (
        car.pos.x + half_l * cos_a + half_w * sin_a,
        car.pos.y + half_l * sin_a - half_w * cos_a,
    );
    let c2 = (
        car.pos.x - half_l * cos_a + half_w * sin_a,
        car.pos.y - half_l * sin_a - half_w * cos_a,
    );
    let c3 = (
        car.pos.x - half_l * cos_a - half_w * sin_a,
        car.pos.y - half_l * sin_a + half_w * cos_a,
    );

    // Desenha corpo do carro com 2 triângulos
    canvas.draw_triangle(c0.0, c0.1, c1.0, c1.1, c2.0, c2.1, color);
    canvas.draw_triangle(c2.0, c2.1, c3.0, c3.1, c0.0, c0.1, color);

    // Farol frontal luminoso
    let front_center_x = car.pos.x + half_l * cos_a;
    let front_center_y = car.pos.y + half_l * sin_a;
    canvas.draw_circle(front_center_x, front_center_y, 2.0, Color::rgb(255, 255, 255));
}

fn draw_sensors(canvas: &mut Canvas, car: &Car) {
    for (i, &rel_angle) in RAY_ANGLES.iter().enumerate() {
        let angle = car.angle + rel_angle;
        let t = car.ray_readings[i];
        let ray_len = MAX_RAY_DIST * t;

        let end_x = car.pos.x + angle.cos() * ray_len;
        let end_y = car.pos.y + angle.sin() * ray_len;

        let col = if t < 0.25 {
            Color::rgba(255, 50, 50, 200)
        } else if t < 0.6 {
            Color::rgba(255, 200, 30, 140)
        } else {
            Color::rgba(50, 255, 100, 90)
        };

        canvas.draw_line(car.pos.x, car.pos.y, end_x, end_y, 1.0, col);
        canvas.draw_circle(end_x, end_y, 2.5, col);
    }
}

fn render_sidebar(canvas: &mut Canvas, sim: &Simulation, leader_idx: usize) {
    let screen_w = canvas.width as f32;
    let screen_h = canvas.height as f32;
    let panel_x = screen_w - SIDEBAR_WIDTH;

    // Fundo do painel lateral escuro com borda divisória
    canvas.draw_rect(panel_x, 0.0, SIDEBAR_WIDTH, screen_h, Color::rgba(13, 15, 20, 245));
    canvas.draw_line(panel_x, 0.0, panel_x, screen_h, 2.0, Color::rgb(40, 50, 75));

    let mut y = 20.0;
    let margin = panel_x + 16.0;

    // Cabeçalho
    canvas.draw_text("DARWIN RACING", margin, y, 2, Color::rgb(50, 200, 255));
    y += 20.0;
    canvas.draw_text(&format!("Pista: {}", sim.track.name), margin, y, 1, Color::rgb(230, 200, 75));
    y += 14.0;
    canvas.draw_text("MLP + Algoritmos Geneticos (Safe Rust)", margin, y, 1, Color::rgb(150, 165, 190));
    y += 22.0;

    // Bloco 1: Telemetria da Geração
    let alive = sim.alive_count();
    let total = sim.agents.len();
    canvas.draw_text(&format!("Geracao: {}", sim.ga.generation), margin, y, 1, Color::rgb(255, 255, 255));
    y += 16.0;
    canvas.draw_text(
        &format!("Vivos: {} / {}", alive, total),
        margin,
        y,
        1,
        if alive > 0 { Color::rgb(80, 230, 100) } else { Color::rgb(240, 50, 50) },
    );
    y += 16.0;

    let turbo_str = match sim.turbo_multiplier {
        1 => "1x (Tempo Real 60fps)",
        5 => "5x (Rapido)",
        _ => "50x (Turbo Rayon Multi-Core)",
    };
    canvas.draw_text(&format!("Velocidade: {}", turbo_str), margin, y, 1, Color::rgb(255, 220, 50));
    y += 16.0;

    canvas.draw_text(
        &format!("Melhor Fitness Historico: {:.0}", sim.best_historical_fitness),
        margin,
        y,
        1,
        Color::rgb(255, 215, 50),
    );
    y += 24.0;

    // Bloco 2: Visualizador da MLP em Tempo Real (Carro Líder)
    canvas.draw_text("Cerebro do Lider (MLP)", margin, y, 1, Color::rgb(100, 200, 255));
    y += 12.0;

    let leader = &sim.agents[leader_idx];
    let mut inputs = [0.0; 8];
    for i in 0..7 {
        inputs[i] = leader.car.ray_readings[i];
    }
    inputs[7] = leader.car.speed / 5.5;

    let (trace, outputs) = leader.mlp.forward_with_trace(&inputs);
    draw_mlp_graph(canvas, margin, y, SIDEBAR_WIDTH - 32.0, 140.0, &sim.topology, &trace, &leader.mlp);
    y += 150.0;

    // Indicador dos atuadores da MLP
    let steer_val = outputs[0];
    let throttle_val = outputs[1];
    canvas.draw_text(
        &format!("Volante: {:+.2} | Motor: {:+.2}", steer_val, throttle_val),
        margin,
        y,
        1,
        Color::rgb(200, 215, 230),
    );
    y += 22.0;

    // Bloco 3: Gráfico de Evolução do Fitness
    canvas.draw_text("Evolucao do Fitness", margin, y, 1, Color::rgb(100, 200, 255));
    y += 14.0;
    draw_fitness_chart(canvas, margin, y, SIDEBAR_WIDTH - 32.0, 95.0, &sim.ga.history);
    y += 105.0;

    // Bloco 4: Controles e Atalhos
    canvas.draw_text("Controles & Atalhos", margin, y, 1, Color::rgb(230, 230, 230));
    y += 16.0;

    let controls = [
        "[Espaco]: Alternar Velocidade (1x/5x/50x)",
        "[G]: Pular 10 Geracoes Instantaneamente",
        "[N / Backspace]: Resetar IA (Geracao 1)",
        "[1, 2, 3]: Pistas F1 (Brasil, Italia, Spa)",
        "[H]: Modo Humano vs IA (WASD ou Setas)",
        "[R]: Alternar Raios Laser",
        "[S]: Salvar Genoma  |  [L]: Carregar",
    ];

    for c in &controls {
        canvas.draw_text(c, margin, y, 1, Color::rgb(165, 175, 200));
        y += 14.0;
    }
}

fn draw_mlp_graph(
    canvas: &mut Canvas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    topology: &[usize],
    trace: &ForwardTrace,
    mlp: &crate::mlp::Mlp,
) {
    canvas.draw_rect(x, y, w, h, Color::rgba(20, 25, 35, 230));
    canvas.draw_rect_lines(x, y, w, h, 1.0, Color::rgb(50, 65, 90));

    let num_layers = topology.len();
    let col_spacing = w / (num_layers as f32);

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

    // Conexões / pesos sinápticos
    for l in 0..(num_layers - 1) {
        let layer_weights = &mlp.layers[l];
        let in_count = topology[l];
        let out_count = topology[l + 1];

        for o in 0..out_count {
            for i in 0..in_count {
                let weight = layer_weights.weights[o * in_count + i];
                let from = node_coords[l][i];
                let to = node_coords[l + 1][o];

                let (col_rgb, alpha) = if weight >= 0.0 {
                    ((50, 150, 255), ((weight / 2.0).clamp(0.1, 0.7) * 255.0) as u8)
                } else {
                    ((255, 75, 75), ((-weight / 2.0).clamp(0.1, 0.7) * 255.0) as u8)
                };

                canvas.draw_line(
                    from.x,
                    from.y,
                    to.x,
                    to.y,
                    1.0,
                    Color::rgba(col_rgb.0, col_rgb.1, col_rgb.2, alpha),
                );
            }
        }
    }

    // Neurônios com brilho proporcional à ativação
    for (l, coords) in node_coords.iter().enumerate() {
        for (n, pt) in coords.iter().enumerate() {
            let act = trace.activations[l][n];
            let node_color = if act >= 0.0 {
                let g = (200.0 + 55.0 * act).clamp(0.0, 255.0) as u8;
                Color::rgb(50, g, 80)
            } else {
                let r = (200.0 - 55.0 * act).clamp(0.0, 255.0) as u8;
                Color::rgb(r, 50, 150)
            };

            canvas.draw_circle(pt.x, pt.y, 4.0, node_color);
            canvas.draw_circle_lines(pt.x, pt.y, 4.0, Color::rgb(255, 255, 255));
        }
    }
}

fn draw_fitness_chart(
    canvas: &mut Canvas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    history: &[crate::ga::GenerationStats],
) {
    canvas.draw_rect(x, y, w, h, Color::rgba(20, 25, 35, 230));
    canvas.draw_rect_lines(x, y, w, h, 1.0, Color::rgb(50, 65, 90));

    if history.len() < 2 {
        canvas.draw_text("Aguardando termino da Geracao 1...", x + 15.0, y + h / 2.0 - 4.0, 1, Color::rgb(130, 130, 130));
        return;
    }

    let max_fitness = history
        .iter()
        .map(|s| s.best_fitness)
        .fold(10.0_f32, |a, b| a.max(b));

    let n = history.len();
    let dx = w / (n - 1) as f32;

    for i in 0..(n - 1) {
        let s0 = &history[i];
        let s1 = &history[i + 1];

        let x0 = x + i as f32 * dx;
        let x1 = x + (i + 1) as f32 * dx;

        let y0_best = y + h - (s0.best_fitness / max_fitness) * (h - 10.0) - 5.0;
        let y1_best = y + h - (s1.best_fitness / max_fitness) * (h - 10.0) - 5.0;

        let y0_avg = y + h - (s0.avg_fitness / max_fitness) * (h - 10.0) - 5.0;
        let y1_avg = y + h - (s1.avg_fitness / max_fitness) * (h - 10.0) - 5.0;

        canvas.draw_line(x0, y0_avg, x1, y1_avg, 1.0, Color::rgba(230, 205, 50, 180));
        canvas.draw_line(x0, y0_best, x1, y1_best, 2.0, Color::rgb(50, 255, 100));
    }

    // Legenda
    canvas.draw_circle(x + 10.0, y + 10.0, 3.0, Color::rgb(50, 255, 100));
    canvas.draw_text("Melhor", x + 18.0, y + 7.0, 1, Color::rgb(255, 255, 255));

    canvas.draw_circle(x + 75.0, y + 10.0, 3.0, Color::rgb(230, 205, 50));
    canvas.draw_text("Media", x + 83.0, y + 7.0, 1, Color::rgb(255, 255, 255));
}
