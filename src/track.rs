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

    /// Circuito 1: F1 Interlagos (GP Brasil — Traçado Final Suavizado)
    pub fn circuit_1() -> Self {
        Self::generate_from_centerline(
            "1: F1 Interlagos (Brasil)",
            &[
                // --- Setor 1: Reta Principal, S do Senna e Curva do Sol ---
                Vec2::new(350.0, 560.0), // Reta Principal (Largada / Chegada)
                Vec2::new(150.0, 480.0), // Fim da Reta Principal (Freada T1)
                Vec2::new(110.0, 410.0), // S do Senna (T1 - Perna Esquerda)
                Vec2::new(170.0, 350.0), // S do Senna (T2 - Perna Direita)
                Vec2::new(140.0, 240.0), // Curva do Sol (T3 - Contorno longo à esquerda)
                Vec2::new(220.0, 130.0), // Saída da Curva do Sol

                // --- Setor 2: Reta Oposta, Descida do Lago e Miolo ---
                Vec2::new(500.0, 95.0),  // Reta Oposta (Meio)
                Vec2::new(760.0, 80.0),  // Fim da Reta Oposta (Freada Descida do Lago)
                Vec2::new(835.0, 115.0), // Descida do Lago (T4 - Entrada suavizada)
                Vec2::new(835.0, 175.0), // Descida do Lago (T4 - Saída aberta)
                Vec2::new(760.0, 210.0), // Reta curta após o Lago (T5)
                Vec2::new(520.0, 210.0), // Reta interna rumo à Ferradura
                Vec2::new(340.0, 250.0), // Ferradura (T6 - Entrada)
                Vec2::new(320.0, 340.0), // Ferradura (T7 - Contorno à esquerda)
                Vec2::new(415.0, 385.0), // Laranjinha (T8 - Direita)
                Vec2::new(460.0, 295.0), // Pinheirinho (T9 - Grampo à esquerda)
                Vec2::new(585.0, 290.0), // Reta curta rumo ao Bico de Pato
                Vec2::new(635.0, 355.0), // Bico de Pato (T10 - Cotovelo à direita)
                Vec2::new(570.0, 425.0), // Mergulho (T11 - Entrada aberta)
                Vec2::new(610.0, 465.0), // Mergulho (T11 - Saída sem estrangular a parede)
                Vec2::new(710.0, 455.0), // Reta rumo à Junção

                // --- Setor 3: Junção e Subida dos Boxes ---
                Vec2::new(820.0, 425.0), // Junção (T12 - Esquerda)
                Vec2::new(840.0, 520.0), // Início da Subida dos Boxes (T13)
                Vec2::new(720.0, 585.0), // Subida dos Boxes
                Vec2::new(540.0, 595.0), // Curva do Café / Arquibancadas (T14/T15)
            ],
            36.0,
        )
    }
    

    // =========================================================================
    // 5 CIRCUITOS AUTOMOBILÍSTICOS REAIS (PROGRESSIVOS: VELOZ -> TÉCNICO -> SUPREMO)
    // =========================================================================

    /// Pista 1: F1 Monza (GP da Itália - "O Templo da Velocidade")
    /// Retas velozes, a clássica Variante del Rettifilo, Curva Grande, Lesmo 1/2,
    /// Variante Ascari e a lendária Parabólica Alboreto.
    pub fn oval_1() -> Self {
        Self::generate_from_centerline(
            "1: F1 Monza (Italia)",
            &[
                Vec2::new(320.0, 580.0), // Reta Principal (Largada)
                Vec2::new(560.0, 580.0), // Fim da Reta Principal
                Vec2::new(690.0, 580.0), // Freada forte para a Variante Rettifilo
                Vec2::new(730.0, 530.0), // Chicane Rettifilo (Direita)
                Vec2::new(760.0, 560.0), // Chicane Rettifilo (Esquerda / Saída)
                Vec2::new(820.0, 470.0), // Curva Grande (Contorno aberto em alta)
                Vec2::new(840.0, 340.0), // Curva Grande (Saída)
                Vec2::new(800.0, 220.0), // Curva di Lesmo 1 (Média velocidade)
                Vec2::new(730.0, 160.0), // Curva di Lesmo 2 (Fechando)
                Vec2::new(580.0, 200.0), // Reta do Serraglio
                Vec2::new(430.0, 250.0), // Freada para a Variante Ascari
                Vec2::new(350.0, 280.0), // Ascari (Corte à esquerda)
                Vec2::new(300.0, 220.0), // Ascari (Corte à direita)
                Vec2::new(220.0, 250.0), // Ascari (Saída rápida)
                Vec2::new(130.0, 350.0), // Reta Oposta rumo à Parabólica
                Vec2::new(110.0, 480.0), // Entrada da Curva Parabólica
                Vec2::new(180.0, 560.0), // Parabólica abrindo para a reta
            ],
            48.0,
        )
    }

    /// Pista 2: F1 Red Bull Ring (GP da Áustria - Spielberg)
    /// Traçado de montanha clássico: Curva 1 Niki Lauda, subida íngreme para o cotovelo
    /// da Curva 3 (Remus), descida veloz e sequência técnica de médias/rápidas.
    pub fn oval_2() -> Self {
        Self::generate_from_centerline(
            "2: F1 Red Bull Ring (Austria)",
            &[
                Vec2::new(260.0, 560.0), // Reta Principal (Largada)
                Vec2::new(520.0, 560.0), // Fim da Reta
                Vec2::new(650.0, 540.0), // Curva 1 (Niki Lauda - 90 graus à direita)
                Vec2::new(660.0, 440.0), // Subida em reta longa rumo à Curva 3
                Vec2::new(660.0, 270.0), // Freada mais forte da pista
                Vec2::new(670.0, 150.0), // Curva 3 (Remus - Grampo agudo!)
                Vec2::new(580.0, 150.0), // Descida íngreme em reta
                Vec2::new(440.0, 210.0), // Curva 4 (Glock - Direita em descida)
                Vec2::new(330.0, 260.0), // Curva 5
                Vec2::new(250.0, 310.0), // Curva 6 (Rindt - Esquerda rápida)
                Vec2::new(170.0, 360.0), // Curva 7
                Vec2::new(140.0, 430.0), // Curva 9 (Dupla à direita rápida)
                Vec2::new(170.0, 510.0), // Curva 10 (Red Bull Mobile - Entrada da reta)
            ],
            46.0,
        )
    }

    /// Pista 3: Laguna Seca (IndyCar / IMSA - com o lendário Sacarrolhas / Corkscrew)
    /// Curva 1 rápida, grampo Andretti Hairpin, subida cega para o Sacarrolhas (Curvas 8 e 8A)
    /// com queda brutal de desnível e retorno técnico.
    pub fn oval_3() -> Self {
        Self::generate_from_centerline(
            "3: Laguna Seca (Corkscrew)",
            &[
                Vec2::new(420.0, 580.0), // Reta Principal (Largada)
                Vec2::new(640.0, 580.0),
                Vec2::new(760.0, 550.0), // Curva 1 (Kink suave)
                Vec2::new(840.0, 470.0), // Freada para o Andretti Hairpin (Curva 2)
                Vec2::new(820.0, 360.0), // Grampo Andretti (Fechado duplo à esquerda)
                Vec2::new(730.0, 390.0),
                Vec2::new(670.0, 470.0), // Curva 3 (Direita rápida)
                Vec2::new(570.0, 440.0), // Curva 4 (Direita aberta)
                Vec2::new(490.0, 360.0), // Curva 5 (Esquerda em subida íngreme)
                Vec2::new(490.0, 240.0), // Curva 6 (Subida da Rahal Straight)
                Vec2::new(410.0, 150.0), // Topo cego antes do Corkscrew
                Vec2::new(340.0, 160.0), // THE CORKSCREW (T8: Esquerda cega em queda)
                Vec2::new(320.0, 220.0), // THE CORKSCREW (T8A: Direita caindo no abismo)
                Vec2::new(260.0, 270.0), // Curva Rainey (T9: Esquerda em alta)
                Vec2::new(170.0, 340.0), // Curva 10 (Direita com inclinação)
                Vec2::new(130.0, 440.0), // Freada para a Curva 11 (Cotovelo final à esquerda)
                Vec2::new(220.0, 540.0), // Aceleração plena rumo à reta
            ],
            45.0,
        )
    }

    /// Pista 4: Suzuka International (F1 GP do Japão - O Oito de Ouro)
    /// Considerada pelos pilotos o teste definitivo: Os Esses do Setor 1, Degner 1 e 2,
    /// Hairpin fechado, Spoon Curve (Colher) e a lendária 130R em alta velocidade.
    pub fn oval_4() -> Self {
        Self::generate_from_centerline(
            "4: Suzuka GP (Japao)",
            &[
                Vec2::new(280.0, 580.0), // Reta Principal (Largada)
                Vec2::new(480.0, 580.0),
                Vec2::new(600.0, 550.0), // Curva 1 e 2 (Dupla direita veloz)
                Vec2::new(650.0, 470.0),
                Vec2::new(590.0, 410.0), // S-Curves (S1: Esquerda fluida)
                Vec2::new(660.0, 350.0), // S-Curves (S2: Direita fluida)
                Vec2::new(580.0, 290.0), // S-Curves (S3: Esquerda técnica)
                Vec2::new(640.0, 230.0), // Curva Dunlop (Subida longa em apoio)
                Vec2::new(730.0, 170.0), // Degner 1 (Direita rápida e cega)
                Vec2::new(820.0, 160.0), // Degner 2 (90 graus com zebra alta)
                Vec2::new(850.0, 250.0), // Sob a ponte rumo ao Hairpin
                Vec2::new(830.0, 370.0), // Hairpin T11 (Grampo de 1ª marcha!)
                Vec2::new(740.0, 330.0), // Curva 200R (Aceleração longa)
                Vec2::new(630.0, 210.0), // Entrada da Spoon Curve (Colher - T13)
                Vec2::new(510.0, 150.0), // Spoon Curve (T14 - Saída em apoio)
                Vec2::new(350.0, 200.0), // Reta dos Boxes Oposta (Reta longa)
                Vec2::new(200.0, 310.0), // Curva 130R (Esquerda de pé cravado em 8ª marcha!)
                Vec2::new(140.0, 430.0), // Chicane Casio Triangle (Freada brutal)
                Vec2::new(190.0, 520.0), // Saída da Chicane rumo à reta
            ],
            42.0,
        )
    }

    /// Pista 5: Nürburgring Nordschleife Retrô ("O Inferno Verde" - Desafio Supremo)
    /// O circuito mais implacável do mundo automobilístico condensado:
    /// Flugplatz, Aremberg, Fuchsröhre, Caracciola Karussell, Hohe Acht, Brünnchen
    /// e a colossal reta de Döttinger Höhe.
    pub fn oval_5() -> Self {
        Self::generate_from_centerline(
            "5: Nurburgring Nordschleife (Inferno Verde)",
            &[
                Vec2::new(220.0, 600.0), // Reta de Döttinger Höhe (Largada)
                Vec2::new(450.0, 600.0),
                Vec2::new(700.0, 600.0), // Antoniusbuche / Tiergarten
                Vec2::new(820.0, 550.0), // Hatzenbach (Sequência de chicanes)
                Vec2::new(850.0, 450.0), // Hocheichen
                Vec2::new(790.0, 360.0), // Flugplatz (Salto veloz em curva)
                Vec2::new(840.0, 270.0), // Schwedenkreuz (Média-alta)
                Vec2::new(850.0, 170.0), // Aremberg (Hairpin em descida profunda)
                Vec2::new(740.0, 130.0), // Fuchsröhre (Compressão no fundo do vale)
                Vec2::new(620.0, 150.0), // Adenauer Forst (Chicane cega técnica)
                Vec2::new(540.0, 230.0), // Metzgesfeld
                Vec2::new(460.0, 210.0), // Kallenhard
                Vec2::new(410.0, 280.0), // Wehrseifen (Grampo estreitíssimo)
                Vec2::new(450.0, 360.0), // Breidscheid (Ponto mais baixo)
                Vec2::new(540.0, 420.0), // Ex-Mühle (Subida íngreme)
                Vec2::new(620.0, 380.0), // Bergwerk (Curva crucial de tração)
                Vec2::new(650.0, 290.0), // Kesselchen (Subida em alta velocidade)
                Vec2::new(580.0, 240.0), // Klostertal
                Vec2::new(490.0, 170.0), // CARACCIOLA KARUSSELL (Curva inclinada de concreto!)
                Vec2::new(380.0, 180.0), // Hohe Acht (Ponto mais alto da floresta)
                Vec2::new(280.0, 230.0), // Wippermann
                Vec2::new(220.0, 300.0), // Eschbach
                Vec2::new(160.0, 360.0), // Brünnchen ("Pula-Pula" dos fãs)
                Vec2::new(110.0, 440.0), // Pflanzgarten (Salto técnico duplo)
                Vec2::new(120.0, 530.0), // Schwalbenschwanz (Pequeno Karussell)
                Vec2::new(160.0, 580.0), // Galgenkopf (Entrada para a reta infinita)
            ],
            38.0,
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

