#![forbid(unsafe_code)]

use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genome {
    pub genes: Vec<f32>,
    pub fitness: f32,
    pub checkpoints_passed: usize,
}

impl Genome {
    pub fn new(genes: Vec<f32>) -> Self {
        Self {
            genes,
            fitness: 0.0,
            checkpoints_passed: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GaConfig {
    pub population_size: usize,
    pub elitism_count: usize,
    pub tournament_size: usize,
    pub crossover_rate: f32,
    pub mutation_rate: f32,
    pub mutation_strength: f32,
}

impl Default for GaConfig {
    fn default() -> Self {
        Self {
            population_size: 200,
            elitism_count: 10,
            tournament_size: 5,
            crossover_rate: 0.85,
            mutation_rate: 0.15,
            mutation_strength: 0.30,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationStats {
    pub generation: usize,
    pub best_fitness: f32,
    pub avg_fitness: f32,
    pub best_checkpoints: usize,
}

pub struct GeneticAlgorithm {
    pub config: GaConfig,
    pub population: Vec<Genome>,
    pub generation: usize,
    pub history: Vec<GenerationStats>,
}

impl GeneticAlgorithm {
    pub fn new(config: GaConfig, initial_genes_fn: impl Fn() -> Vec<f32>) -> Self {
        let mut population = Vec::with_capacity(config.population_size);
        for _ in 0..config.population_size {
            population.push(Genome::new(initial_genes_fn()));
        }

        Self {
            config,
            population,
            generation: 1,
            history: Vec::new(),
        }
    }

    pub fn record_generation_stats(&mut self) -> GenerationStats {
        let mut best_fit = -f32::INFINITY;
        let mut sum_fit = 0.0;
        let mut best_checkpoints = 0;

        for g in &self.population {
            if g.fitness > best_fit {
                best_fit = g.fitness;
                best_checkpoints = g.checkpoints_passed;
            }
            sum_fit += g.fitness;
        }

        let avg_fit = if self.population.is_empty() {
            0.0
        } else {
            sum_fit / self.population.len() as f32
        };

        let stats = GenerationStats {
            generation: self.generation,
            best_fitness: best_fit,
            avg_fitness: avg_fit,
            best_checkpoints,
        };

        self.history.push(stats.clone());
        stats
    }

    pub fn evolve(&mut self, rng: &mut impl Rng) {
        self.record_generation_stats();

        // 1. Ordena decrescente por fitness
        self.population.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut next_population = Vec::with_capacity(self.config.population_size);

        // 2. Elitismo: preserva os melhores exatamente como são
        let elite_count = self.config.elitism_count.min(self.population.len());
        for i in 0..elite_count {
            let mut elite = self.population[i].clone();
            elite.fitness = 0.0;
            elite.checkpoints_passed = 0;
            next_population.push(elite);
        }

        let normal_dist = Normal::new(0.0, self.config.mutation_strength)
            .unwrap_or_else(|_| Normal::new(0.0, 0.1).unwrap());

        // 3. Reprodução (Torneio + Crossover + Mutação) até completar a nova população
        while next_population.len() < self.config.population_size {
            let p1 = Self::tournament_select(&self.population, self.config.tournament_size, rng);
            let p2 = Self::tournament_select(&self.population, self.config.tournament_size, rng);

            let mut child_genes = if rng.gen_bool(self.config.crossover_rate as f64) {
                Self::crossover_blx_alpha(&p1.genes, &p2.genes, 0.15, rng)
            } else {
                p1.genes.clone()
            };

            Self::mutate(&mut child_genes, self.config.mutation_rate, &normal_dist, rng);

            next_population.push(Genome::new(child_genes));
        }

        self.population = next_population;
        self.generation += 1;
    }

    fn tournament_select<'a>(population: &'a [Genome], k: usize, rng: &mut impl Rng) -> &'a Genome {
        let mut best = &population[rng.gen_range(0..population.len())];
        for _ in 1..k {
            let candidate = &population[rng.gen_range(0..population.len())];
            if candidate.fitness > best.fitness {
                best = candidate;
            }
        }
        best
    }

    fn crossover_blx_alpha(p1: &[f32], p2: &[f32], alpha: f32, rng: &mut impl Rng) -> Vec<f32> {
        let len = p1.len();
        let mut child = Vec::with_capacity(len);

        for i in 0..len {
            let g1 = p1[i];
            let g2 = p2[i];
            let min = g1.min(g2);
            let max = g1.max(g2);
            let range = max - min;
            let low = min - alpha * range;
            let high = max + alpha * range;
            child.push(rng.gen_range(low..=high).clamp(-5.0, 5.0));
        }

        child
    }

    fn mutate(genes: &mut [f32], rate: f32, normal: &Normal<f32>, rng: &mut impl Rng) {
        for gene in genes.iter_mut() {
            if rng.gen_bool(rate as f64) {
                if rng.gen_bool(0.05) {
                    // 5% das mutações são substituições completas (reset de peso)
                    *gene = rng.gen_range(-2.0..2.0);
                } else {
                    // 95% das mutações são pequenas perturbações normais
                    let delta = normal.sample(rng);
                    *gene = (*gene + delta).clamp(-5.0, 5.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_ga_elitism_preserves_best() {
        let mut rng = thread_rng();
        let config = GaConfig {
            population_size: 10,
            elitism_count: 2,
            tournament_size: 3,
            crossover_rate: 0.8,
            mutation_rate: 0.1,
            mutation_strength: 0.2,
        };

        let mut ga = GeneticAlgorithm::new(config, || vec![0.0; 5]);
        for (i, ind) in ga.population.iter_mut().enumerate() {
            ind.fitness = i as f32 * 10.0;
            ind.genes[0] = ind.fitness;
        }

        // Indivíduo com maior fitness tem fitness = 90.0 e genes[0] = 90.0
        ga.evolve(&mut rng);

        assert_eq!(ga.population.len(), 10);
        // O primeiro da nova geração (elite 1) deve ter exatamente os genes do melhor pai
        assert!((ga.population[0].genes[0] - 90.0).abs() < 1e-6);
        // O segundo da nova geração (elite 2) deve ter genes[0] = 80.0
        assert!((ga.population[1].genes[0] - 80.0).abs() < 1e-6);
    }
}

