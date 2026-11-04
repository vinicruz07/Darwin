#![forbid(unsafe_code)]

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub in_size: usize,
    pub out_size: usize,
    /// Pesos ordenados linha por linha: out_size linhas, cada uma com in_size colunas.
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
}

impl Layer {
    pub fn new_random(in_size: usize, out_size: usize, rng: &mut impl Rng) -> Self {
        // Inicialização uniforme ampla [-1.5, 1.5] para pesos e [-1.0, 1.0] para biases.
        // Isso garante que na Geração 1 os indivíduos sejam realmente caóticos/aleatórios
        // (sem viés artificial de andar reto/suave no centro da tanh).
        let total_weights = in_size * out_size;
        let mut weights = Vec::with_capacity(total_weights);
        for _ in 0..total_weights {
            weights.push(rng.gen_range(-1.5..1.5));
        }
        let mut biases = Vec::with_capacity(out_size);
        for _ in 0..out_size {
            biases.push(rng.gen_range(-1.0..1.0));
        }

        Self {
            in_size,
            out_size,
            weights,
            biases,
        }
    }

    pub fn forward(&self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), self.in_size);
        assert_eq!(output.len(), self.out_size);

        for o in 0..self.out_size {
            let row_start = o * self.in_size;
            let mut sum = self.biases[o];
            for i in 0..self.in_size {
                sum += self.weights[row_start + i] * input[i];
            }
            output[o] = sum.tanh();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForwardTrace {
    pub activations: Vec<Vec<f32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mlp {
    pub topology: Vec<usize>,
    pub layers: Vec<Layer>,
}

impl Mlp {
    pub fn new_random(topology: &[usize], rng: &mut impl Rng) -> Self {
        assert!(topology.len() >= 2, "Topologia precisa de pelo menos 2 camadas");
        let mut layers = Vec::new();
        for i in 0..(topology.len() - 1) {
            layers.push(Layer::new_random(topology[i], topology[i + 1], rng));
        }
        Self {
            topology: topology.to_vec(),
            layers,
        }
    }

    pub fn total_genes(&self) -> usize {
        let mut sum = 0;
        for layer in &self.layers {
            sum += layer.weights.len() + layer.biases.len();
        }
        sum
    }

    pub fn forward(&self, input: &[f32]) -> [f32; 2] {
        let mut current_buf = input.to_vec();
        for layer in &self.layers {
            let mut next_buf = vec![0.0; layer.out_size];
            layer.forward(&current_buf, &mut next_buf);
            current_buf = next_buf;
        }
        assert_eq!(current_buf.len(), 2);
        [current_buf[0], current_buf[1]]
    }

    pub fn forward_with_trace(&self, input: &[f32]) -> (ForwardTrace, [f32; 2]) {
        let mut activations = Vec::with_capacity(self.topology.len());
        activations.push(input.to_vec());

        let mut current_buf = input.to_vec();
        for layer in &self.layers {
            let mut next_buf = vec![0.0; layer.out_size];
            layer.forward(&current_buf, &mut next_buf);
            activations.push(next_buf.clone());
            current_buf = next_buf;
        }

        let out = [current_buf[0], current_buf[1]];
        (ForwardTrace { activations }, out)
    }

    pub fn to_genes(&self) -> Vec<f32> {
        let mut genes = Vec::with_capacity(self.total_genes());
        for layer in &self.layers {
            genes.extend_from_slice(&layer.weights);
            genes.extend_from_slice(&layer.biases);
        }
        genes
    }

    pub fn from_genes(topology: &[usize], genes: &[f32]) -> Self {
        let mut layers = Vec::new();
        let mut offset = 0;

        for i in 0..(topology.len() - 1) {
            let in_size = topology[i];
            let out_size = topology[i + 1];
            let w_len = in_size * out_size;
            let b_len = out_size;

            assert!(
                offset + w_len + b_len <= genes.len(),
                "Tamanho do cromossomo insuficiente para a topologia fornecida"
            );

            let weights = genes[offset..offset + w_len].to_vec();
            offset += w_len;
            let biases = genes[offset..offset + b_len].to_vec();
            offset += b_len;

            layers.push(Layer {
                in_size,
                out_size,
                weights,
                biases,
            });
        }

        Self {
            topology: topology.to_vec(),
            layers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_mlp_genes_roundtrip() {
        let mut rng = thread_rng();
        let topology = [8, 12, 8, 2];
        let original_mlp = Mlp::new_random(&topology, &mut rng);
        let genes = original_mlp.to_genes();
        let reconstructed_mlp = Mlp::from_genes(&topology, &genes);

        let input = [0.1, -0.5, 0.8, 0.0, -0.2, 0.9, -0.7, 0.4];
        let out1 = original_mlp.forward(&input);
        let out2 = reconstructed_mlp.forward(&input);

        assert!((out1[0] - out2[0]).abs() < 1e-6);
        assert!((out1[1] - out2[1]).abs() < 1e-6);
    }

    #[test]
    fn test_mlp_forward_bounds() {
        let mut rng = thread_rng();
        let topology = [8, 12, 8, 2];
        let mlp = Mlp::new_random(&topology, &mut rng);
        let input = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let out = mlp.forward(&input);

        assert!(out[0] >= -1.0 && out[0] <= 1.0);
        assert!(out[1] >= -1.0 && out[1] <= 1.0);
        assert!(!out[0].is_nan());
        assert!(!out[1].is_nan());
    }
}

