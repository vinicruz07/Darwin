#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dist(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineSegment {
    pub a: Vec2,
    pub b: Vec2,
}

impl LineSegment {
    pub const fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }

    /// Calcula a interseção entre um raio `(origin, dir * max_range)` e o segmento `[a, b]`.
    /// Retorna `Some(t)` com `t in [0.0, 1.0]` representando a fração da distância do raio até o impacto.
    pub fn intersect_ray(&self, origin: Vec2, dir: Vec2, max_dist: f32) -> Option<f32> {
        let p = origin;
        let r = Vec2::new(dir.x * max_dist, dir.y * max_dist);
        let q = self.a;
        let s = Vec2::new(self.b.x - self.a.x, self.b.y - self.a.y);

        let r_cross_s = r.x * s.y - r.y * s.x;
        let q_minus_p = Vec2::new(q.x - p.x, q.y - p.y);

        if r_cross_s.abs() < 1e-8 {
            return None; // Linhas paralelas
        }

        let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / r_cross_s;
        let u = (q_minus_p.x * r.y - q_minus_p.y * r.x) / r_cross_s;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            Some(t)
        } else {
            None
        }
    }

    /// Testa interseção entre dois segmentos de reta finitos
    pub fn intersects_segment(&self, other: &LineSegment) -> bool {
        let p = self.a;
        let r = Vec2::new(self.b.x - self.a.x, self.b.y - self.a.y);
        let q = other.a;
        let s = Vec2::new(other.b.x - other.a.x, other.b.y - other.a.y);

        let r_cross_s = r.x * s.y - r.y * s.x;
        let q_minus_p = Vec2::new(q.x - p.x, q.y - p.y);

        if r_cross_s.abs() < 1e-8 {
            return false;
        }

        let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / r_cross_s;
        let u = (q_minus_p.x * r.y - q_minus_p.y * r.x) / r_cross_s;

        (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub spawn_pos: Vec2,
    pub spawn_angle: f32,
    pub inner_walls: Vec<LineSegment>,
    pub outer_walls: Vec<LineSegment>,
    pub checkpoints: Vec<LineSegment>,
    pub center_points: Vec<Vec2>,
}

impl Track {
    pub fn check_wall_collision(&self, car_segments: &[LineSegment]) -> bool {
        for seg in car_segments {
            for wall in &self.inner_walls {
                if seg.intersects_segment(wall) {
                    return true;
                }
            }
            for wall in &self.outer_walls {
                if seg.intersects_segment(wall) {
                    return true;
                }
            }
        }
        false
    }

    pub fn cast_ray(&self, origin: Vec2, angle: f32, max_dist: f32) -> f32 {
        let dir = Vec2::new(angle.cos(), angle.sin());
        let mut closest_t = 1.0;

        for wall in &self.inner_walls {
            if let Some(t) = wall.intersect_ray(origin, dir, max_dist) {
                if t < closest_t {
                    closest_t = t;
                }
            }
        }

        for wall in &self.outer_walls {
            if let Some(t) = wall.intersect_ray(origin, dir, max_dist) {
                if t < closest_t {
                    closest_t = t;
                }
            }
        }

        closest_t
    }

    pub fn circuit_1() -> Self {
        Self::generate_from_centerline(
            "1: F1 Interlagos (Brasil)",
            &[
                // --- Setor 1: Reta Principal, S do Senna e Curva do Sol ---
                Vec2::new(280.0, 380.0), // Reta Principal (Largada / Chegada)
                Vec2::new(330.0, 545.0), // Fim da Reta Principal (Freada T1)
                Vec2::new(350.0, 555.0), // S do Senna (T1 - Perna Esquerda)
                Vec2::new(390.0, 530.0), // S do Senna (T2 - Perna Direita)
                Vec2::new(455.0, 550.0), // Curva do Sol (T3 - Contorno longo à esquerda)
                Vec2::new(515.0, 510.0), // Saída da Curva do Sol

                // --- Setor 2: Reta Oposta, Descida do Lago e Miolo ---
                Vec2::new(565.0, 350.0), // Reta Oposta (Meio)
                Vec2::new(610.0, 195.0), // Fim da Reta Oposta (Freada Descida do Lago)
                Vec2::new(600.0, 175.0), // Descida do Lago (T4 - Esquerda rápida)
                Vec2::new(535.0, 165.0), // Reta curta após o Lago (T5)
                Vec2::new(495.0, 180.0), // Início da descida para a Ferradura
                Vec2::new(380.0, 325.0), // Ferradura (T6 - Entrada à esquerda)
                Vec2::new(335.0, 330.0), // Ferradura (Meio da curva)
                Vec2::new(300.0, 300.0), // Ferradura (T7 - Saída subindo)
                Vec2::new(285.0, 240.0), // Laranjinha (T8 - Entrada)
                Vec2::new(305.0, 225.0), // Laranjinha (Contorno à direita)
                Vec2::new(340.0, 245.0), // Entrada do Pinheirinho
                Vec2::new(365.0, 225.0), // Pinheirinho (T9 - Grampo à esquerda)
                Vec2::new(360.0, 200.0), // Saída do Pinheirinho
                Vec2::new(320.0, 165.0), // Reta curta rumo ao Bico de Pato
                Vec2::new(310.0, 120.0), // Bico de Pato (T10 - Freada)
                Vec2::new(325.0, 105.0), // Bico de Pato (Cotovelo fechado à direita)
                Vec2::new(375.0, 160.0), // Descida após o Bico de Pato
                Vec2::new(410.0, 170.0), // Mergulho (T11 - Curva à esquerda)
                Vec2::new(450.0, 145.0), // Freada para a Junção

                // --- Setor 3: Junção e Subida dos Boxes ---
                Vec2::new(500.0, 65.0),  // Junção (T12 - Entrada)
                Vec2::new(490.0, 50.0),  // Junção (Contorno à esquerda)
                Vec2::new(425.0, 35.0),  // Início da Subida dos Boxes (T13)
                Vec2::new(350.0, 55.0),  // Subida dos Boxes
                Vec2::new(300.0, 90.0),  // Curva do Café (T14)
                Vec2::new(255.0, 260.0), // Arquibancadas (T15 - Entrada da Reta)
            ],
            32.0, // Se a pista ficar larga demais nesta escala, reduza para ~32.0
        )
    }

    /// Circuito 2: F1 Monza (GP Italia — Rettifilo, Curva Grande, Lesmo 1/2, Ascari, Parabolica)
    pub fn circuit_2() -> Self {
        Self::generate_from_centerline(
            "2: F1 Monza (Italia)",
            &[
                Vec2::new(220.0, 590.0), // Reta Principal (Largada)
                Vec2::new(520.0, 600.0), // Metade da Reta
                Vec2::new(780.0, 590.0), // Variante del Rettifilo (Entrada)
                Vec2::new(830.0, 520.0), // Chicane Rettifilo (Saída)
                Vec2::new(820.0, 340.0), // Curva Grande
                Vec2::new(760.0, 170.0), // Curva di Lesmo 1
                Vec2::new(600.0, 140.0), // Curva di Lesmo 2
                Vec2::new(470.0, 250.0), // Descida do Serraglio
                Vec2::new(340.0, 290.0), // Variante Ascari (Entrada esquerda)
                Vec2::new(250.0, 230.0), // Variante Ascari (Saída direita)
                Vec2::new(130.0, 290.0), // Reta Oposta rumo à Parabolica
                Vec2::new(110.0, 470.0), // Curva Parabolica (Alboreto)
            ],
            50.0,
        )
    }

    /// Circuito 3: F1 Spa-Francorchamps (Eau Rouge, Raidillon, Kemmel, Pouhon, Bus Stop)
    pub fn circuit_3() -> Self {
        Self::generate_from_centerline(
            "3: F1 Spa (Belgica)",
            &[
                Vec2::new(180.0, 300.0), // Largada rumo a La Source
                Vec2::new(130.0, 170.0), // Hairpin La Source
                Vec2::new(290.0, 150.0), // Descida para Eau Rouge
                Vec2::new(380.0, 210.0), // Eau Rouge / Raidillon (S em subida)
                Vec2::new(520.0, 130.0), // Reta Kemmel
                Vec2::new(780.0, 140.0), // Les Combes (Chicane)
                Vec2::new(840.0, 260.0), // Bruxelles / Rivage
                Vec2::new(660.0, 340.0), // Entrada de Pouhon
                Vec2::new(520.0, 450.0), // Pouhon (Curva dupla rápida)
                Vec2::new(680.0, 560.0), // Campus / Stavelot
                Vec2::new(820.0, 620.0), // Paul Frere
                Vec2::new(520.0, 640.0), // Blanchimont (Alta velocidade)
                Vec2::new(250.0, 560.0), // Chicane Bus Stop
            ],
            48.0,
        )
    }

    /// Constrói qualquer circuito fechado a partir de uma lista de coordenadas `(x, y)` da linha central (Centerline)
    /// e da largura da pista em pixels (`track_width`).
    ///
    /// ### Formato para criar novos circuitos de F1:
    /// - A área livre de desenho da pista na janela vai de `X: 80.0 .. 860.0` e `Y: 80.0 .. 660.0`
    ///   (já que os últimos 360px à direita são ocupados pelo painel analítico).
    /// - O vetor `points` define os pontos sequenciais do traçado na ordem em que os carros vão correr.
    /// - `points[0]` é a Linha de Largada, e os carros nascem apontados de `points[0]` para `points[1]`.
    /// - O circuito fecha automaticamente ligando o último ponto de volta ao `points[0]`.
    pub fn generate_from_centerline(name: &str, points: &[Vec2], track_width: f32) -> Self {
        let n = points.len();
        let half_w = track_width / 2.0;

        let mut inner_points = Vec::with_capacity(n);
        let mut outer_points = Vec::with_capacity(n);
        let mut checkpoints = Vec::with_capacity(n);

        for i in 0..n {
            let prev = points[(i + n - 1) % n];
            let curr = points[i];
            let next = points[(i + 1) % n];

            // Vetor tangente médio no ponto
            let d1 = Vec2::new(curr.x - prev.x, curr.y - prev.y);
            let d2 = Vec2::new(next.x - curr.x, next.y - curr.y);
            let tan_x = d1.x + d2.x;
            let tan_y = d1.y + d2.y;
            let len = (tan_x * tan_x + tan_y * tan_y).sqrt().max(1e-4);

            // Vetor normal apontando para dentro
            let norm_x = -tan_y / len;
            let norm_y = tan_x / len;

            let in_pt = Vec2::new(curr.x + norm_x * half_w, curr.y + norm_y * half_w);
            let out_pt = Vec2::new(curr.x - norm_x * half_w, curr.y - norm_y * half_w);

            inner_points.push(in_pt);
            outer_points.push(out_pt);
            checkpoints.push(LineSegment::new(in_pt, out_pt));
        }

        let mut inner_walls = Vec::with_capacity(n);
        let mut outer_walls = Vec::with_capacity(n);

        for i in 0..n {
            let next_i = (i + 1) % n;
            inner_walls.push(LineSegment::new(inner_points[i], inner_points[next_i]));
            outer_walls.push(LineSegment::new(outer_points[i], outer_points[next_i]));
        }

        let p0 = points[0];
        let p1 = points[1];
        let spawn_angle = (p1.y - p0.y).atan2(p1.x - p0.x);

        Self {
            name: name.to_string(),
            spawn_pos: p0,
            spawn_angle,
            inner_walls,
            outer_walls,
            checkpoints,
            center_points: points.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raycast_intersection() {
        let wall = LineSegment::new(Vec2::new(10.0, -10.0), Vec2::new(10.0, 10.0));
        let origin = Vec2::new(0.0, 0.0);
        let dir = Vec2::new(1.0, 0.0);
        let max_dist = 20.0;

        let hit = wall.intersect_ray(origin, dir, max_dist);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 0.5).abs() < 1e-4); // A parede está a 10px, com alcance de 20px -> t = 0.5
    }

    #[test]
    fn test_track_generation_and_checkpoints() {
        let track = Track::circuit_1();
        assert!(!track.inner_walls.is_empty());
        assert!(!track.outer_walls.is_empty());
        assert_eq!(track.checkpoints.len(), track.center_points.len());
    }
}

