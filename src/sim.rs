#![forbid(unsafe_code)]

use crate::ga::{GaConfig, GeneticAlgorithm, Genome};
use crate::mlp::Mlp;
use crate::track::{LineSegment, Track, Vec2};
use rand::thread_rng;
use rayon::prelude::*;

pub const RAY_ANGLES: [f32; 7] = [
    -std::f32::consts::FRAC_PI_2,        // -90 deg
    -std::f32::consts::PI * 55.0 / 180.0, // -55 deg
    -std::f32::consts::PI * 25.0 / 180.0, // -25 deg
    0.0,                                 //   0 deg (frente)
    std::f32::consts::PI * 25.0 / 180.0,  // +25 deg
    std::f32::consts::PI * 55.0 / 180.0,  // +55 deg
    std::f32::consts::FRAC_PI_2,         // +90 deg
];

pub const MAX_RAY_DIST: f32 = 220.0;
pub const CAR_LENGTH: f32 = 18.0;
pub const CAR_WIDTH: f32 = 9.0;
pub const TIMEOUT_TICKS: u32 = 240; // 4 segundos a 60 FPS sem cruzar checkpoint = timeout

#[derive(Clone, Debug)]
pub struct Car {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub angle: f32,
    pub speed: f32,
    pub alive: bool,
    pub fitness: f32,
    pub next_checkpoint: usize,
    pub total_checkpoints: usize,
    pub ticks_since_checkpoint: u32,
    pub total_distance: f32,
    pub ray_readings: [f32; 7],
}

impl Car {
    pub fn new(spawn_pos: Vec2, spawn_angle: f32) -> Self {
        Self {
            pos: spawn_pos,
            prev_pos: spawn_pos,
            angle: spawn_angle,
            speed: 1.0,
            alive: true,
            fitness: 0.0,
            next_checkpoint: 1,
            total_checkpoints: 0,
            ticks_since_checkpoint: 0,
            total_distance: 0.0,
            ray_readings: [1.0; 7],
        }
    }

    pub fn respawn(&mut self, spawn_pos: Vec2, spawn_angle: f32) {
        self.pos = spawn_pos;
        self.prev_pos = spawn_pos;
        self.angle = spawn_angle;
        self.speed = 1.0;
        self.alive = true;
        self.fitness = 0.0;
        self.next_checkpoint = 1;
        self.total_checkpoints = 0;
        self.ticks_since_checkpoint = 0;
        self.total_distance = 0.0;
        self.ray_readings = [1.0; 7];
    }

    pub fn get_bounding_segments(&self) -> [LineSegment; 4] {
        let half_l = CAR_LENGTH / 2.0;
        let half_w = CAR_WIDTH / 2.0;

        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();

        // 4 vértices do retângulo orientado do carro
        let corners = [
            Vec2::new(
                self.pos.x + half_l * cos_a - half_w * sin_a,
                self.pos.y + half_l * sin_a + half_w * cos_a,
            ),
            Vec2::new(
                self.pos.x + half_l * cos_a + half_w * sin_a,
                self.pos.y + half_l * sin_a - half_w * cos_a,
            ),
            Vec2::new(
                self.pos.x - half_l * cos_a + half_w * sin_a,
                self.pos.y - half_l * sin_a - half_w * cos_a,
            ),
            Vec2::new(
                self.pos.x - half_l * cos_a - half_w * sin_a,
                self.pos.y - half_l * sin_a + half_w * cos_a,
            ),
        ];

        [
            LineSegment::new(corners[0], corners[1]),
            LineSegment::new(corners[1], corners[2]),
            LineSegment::new(corners[2], corners[3]),
            LineSegment::new(corners[3], corners[0]),
        ]
    }

    pub fn update_physics(&mut self, steer: f32, throttle: f32, dt: f32) {
        if !self.alive {
            return;
        }

        // Steer: [-1, 1], rotação máxima 3.2 rad/s
        let max_steer_speed = 3.2;
        self.angle += steer.clamp(-1.0, 1.0) * max_steer_speed * dt;

        // Throttle: [-1, 1], aceleração e frenagem com atrito aerodinâmico
        let accel = if throttle >= 0.0 {
            throttle * 120.0
        } else {
            throttle * 180.0
        };
        let friction = 0.5 * self.speed;
        self.speed = (self.speed + (accel - friction) * dt).clamp(0.2, 5.5);

        self.prev_pos = self.pos;
        self.pos.x += self.speed * self.angle.cos();
        self.pos.y += self.speed * self.angle.sin();

        let step_dist = self.pos.dist(self.prev_pos);
        self.total_distance += step_dist;
        self.ticks_since_checkpoint += 1;
    }

    pub fn update_sensors(&mut self, track: &Track) {
        for (i, &rel_angle) in RAY_ANGLES.iter().enumerate() {
            let ray_angle = self.angle + rel_angle;
            // 0.0 = colisão imediata, 1.0 = caminho livre
            self.ray_readings[i] = track.cast_ray(self.pos, ray_angle, MAX_RAY_DIST);
        }
    }

    pub fn check_checkpoint_and_collision(&mut self, track: &Track) {
        if !self.alive {
            return;
        }

        // 1. Timeout por estagnação
        if self.ticks_since_checkpoint > TIMEOUT_TICKS {
            self.alive = false;
            return;
        }

        // 2. Colisão com paredes
        let car_segs = self.get_bounding_segments();
        if track.check_wall_collision(&car_segs) {
            self.alive = false;
            return;
        }

        // 3. Verificação de Checkpoint
        let next_cp = &track.checkpoints[self.next_checkpoint % track.checkpoints.len()];
        let path_seg = LineSegment::new(self.prev_pos, self.pos);

        if path_seg.intersects_segment(next_cp) {
            self.total_checkpoints += 1;
            self.next_checkpoint = (self.next_checkpoint + 1) % track.checkpoints.len();
            self.ticks_since_checkpoint = 0;
        }

        // Cálculo contínuo de fitness
        let cp_pos = track.center_points[self.next_checkpoint % track.center_points.len()];
        let dist_to_next_cp = self.pos.dist(cp_pos);
        let progress_bonus = (150.0 - dist_to_next_cp.min(150.0)) / 150.0;

        self.fitness = (self.total_checkpoints as f32 * 1000.0)
            + (progress_bonus * 100.0)
            + (self.total_distance * 0.1);
    }
}

pub struct Agent {
    pub car: Car,
    pub mlp: Mlp,
}

pub struct Simulation {
    pub track: Track,
    pub agents: Vec<Agent>,
    pub ga: GeneticAlgorithm,
    pub best_historical_genome: Option<Genome>,
    pub best_historical_fitness: f32,
    pub topology: Vec<usize>,
    pub human_car: Option<Car>,
    pub champion_replay_car: Option<Car>,
    pub show_sensors: bool,
    pub turbo_multiplier: usize, // 1, 5, ou 50
}

impl Simulation {
    pub fn new(track: Track) -> Self {
        let topology = vec![8, 12, 8, 2];
        let mut rng = thread_rng();

        let dummy_mlp = Mlp::new_random(&topology, &mut rng);
        let gene_len = dummy_mlp.total_genes();

        let ga_config = GaConfig::default();
        let pop_size = ga_config.population_size;

        let ga = GeneticAlgorithm::new(ga_config, || {
            let mut r = thread_rng();
            Mlp::new_random(&topology, &mut r).to_genes()
        });

        assert_eq!(ga.population[0].genes.len(), gene_len);

        let mut agents = Vec::with_capacity(pop_size);
        for g in &ga.population {
            let mlp = Mlp::from_genes(&topology, &g.genes);
            let car = Car::new(track.spawn_pos, track.spawn_angle);
            agents.push(Agent { car, mlp });
        }

        Self {
            track,
            agents,
            ga,
            best_historical_genome: None,
            best_historical_fitness: 0.0,
            topology,
            human_car: None,
            champion_replay_car: None,
            show_sensors: true,
            turbo_multiplier: 1,
        }
    }

    pub fn set_track(&mut self, track: Track) {
        self.track = track;
        self.restart_generation();
    }

    pub fn restart_generation(&mut self) {
        let spawn_pos = self.track.spawn_pos;
        let spawn_angle = self.track.spawn_angle;

        for (agent, genome) in self.agents.iter_mut().zip(&self.ga.population) {
            agent.mlp = Mlp::from_genes(&self.topology, &genome.genes);
            agent.car.respawn(spawn_pos, spawn_angle);
        }

        if let Some(ref mut h) = self.human_car {
            h.respawn(spawn_pos, spawn_angle);
        }
        if let Some(ref mut c) = self.champion_replay_car {
            c.respawn(spawn_pos, spawn_angle);
        }
    }

    pub fn alive_count(&self) -> usize {
        self.agents.iter().filter(|a| a.car.alive).count()
    }

    pub fn get_leader_index(&self) -> usize {
        let mut best_idx = 0;
        let mut best_fit = -1.0;

        for (i, agent) in self.agents.iter().enumerate() {
            if agent.car.fitness > best_fit {
                best_fit = agent.car.fitness;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Executa um único tick físico com avaliação paralela via Rayon
    pub fn step_single_tick(&mut self, dt: f32) {
        let track = &self.track;

        // Atualização dos agentes da IA em paralelo multi-core
        self.agents.par_iter_mut().for_each(|agent| {
            if !agent.car.alive {
                return;
            }

            // 1. Atualizar sensores laser
            agent.car.update_sensors(track);

            // 2. Montar vetor de entrada da MLP (8 entradas)
            let mut inputs = [0.0; 8];
            for i in 0..7 {
                inputs[i] = agent.car.ray_readings[i];
            }
            inputs[7] = agent.car.speed / 5.5; // Velocidade normalizada

            // 3. Propagação pela rede neural
            let outputs = agent.mlp.forward(&inputs);

            // 4. Aplicar comandos na física
            let steer = outputs[0];
            let throttle = outputs[1];
            agent.car.update_physics(steer, throttle, dt);

            // 5. Testar colisões e checkpoints
            agent.car.check_checkpoint_and_collision(track);
        });

        // Atualizar campeão histórico para visualização/duelo
        if let (Some(ref mut champ_car), Some(ref best_gen)) = (
            &mut self.champion_replay_car,
            &self.best_historical_genome,
        ) {
            if champ_car.alive {
                champ_car.update_sensors(track);
                let mut inputs = [0.0; 8];
                for i in 0..7 {
                    inputs[i] = champ_car.ray_readings[i];
                }
                inputs[7] = champ_car.speed / 5.5;

                let mlp = Mlp::from_genes(&self.topology, &best_gen.genes);
                let out = mlp.forward(&inputs);
                champ_car.update_physics(out[0], out[1], dt);
                champ_car.check_checkpoint_and_collision(track);
            }
        }

        // Testar colisão do carro humano
        if let Some(ref mut h) = self.human_car {
            if h.alive {
                h.check_checkpoint_and_collision(track);
            }
        }
    }

    /// Atualiza o loop da simulação (aplicando o multiplicador turbo e checando fim da geração)
    pub fn update(&mut self, dt: f32) {
        for _ in 0..self.turbo_multiplier {
            self.step_single_tick(dt);

            if self.alive_count() == 0 {
                self.end_generation_and_evolve();
                break;
            }
        }
    }

    pub fn end_generation_and_evolve(&mut self) {
        // Copiar os scores dos carros de volta para os genomas
        for (agent, genome) in self.agents.iter().zip(&mut self.ga.population) {
            genome.fitness = agent.car.fitness;
            genome.checkpoints_passed = agent.car.total_checkpoints;

            if genome.fitness > self.best_historical_fitness {
                self.best_historical_fitness = genome.fitness;
                self.best_historical_genome = Some(genome.clone());
            }
        }

        // Algoritmo Genético entra em ação
        let mut rng = thread_rng();
        self.ga.evolve(&mut rng);

        // Respawn de todos os carros na nova geração
        self.restart_generation();
    }

    /// Salta N gerações completas em modo headless ultra-rápido
    pub fn skip_generations(&mut self, generations: usize) {
        let dt = 1.0 / 60.0;
        let max_ticks_per_gen = 2500; // Limite de segurança por geração

        for _ in 0..generations {
            let mut tick = 0;
            while self.alive_count() > 0 && tick < max_ticks_per_gen {
                self.step_single_tick(dt);
                tick += 1;
            }
            self.end_generation_and_evolve();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_headless_step_and_evolve() {
        let track = Track::circuit_1();
        let mut sim = Simulation::new(track);

        assert_eq!(sim.agents.len(), 200);
        assert_eq!(sim.alive_count(), 200);

        // Simula 3 gerações inteiras
        sim.skip_generations(3);

        assert_eq!(sim.ga.generation, 4);
        assert_eq!(sim.ga.history.len(), 3);
        assert!(sim.best_historical_fitness >= 0.0);
    }
}

