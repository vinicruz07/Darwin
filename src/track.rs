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

    /// Circuito 1: Darwin Park (Traçado oval misto com chicanes suaves)
    pub fn circuit_1() -> Self {
        Self::generate_from_centerline(
            "Circuito 1 — Darwin Park",
            &[
                Vec2::new(200.0, 160.0),
                Vec2::new(450.0, 140.0),
                Vec2::new(700.0, 160.0),
                Vec2::new(840.0, 260.0),
                Vec2::new(850.0, 480.0),
                Vec2::new(700.0, 600.0),
                Vec2::new(550.0, 520.0),
                Vec2::new(400.0, 580.0),
                Vec2::new(220.0, 560.0),
                Vec2::new(120.0, 440.0),
                Vec2::new(110.0, 260.0),
            ],
            70.0,
        )
    }

    /// Circuito 2: Serpentina Técnica (Curvas fechadas em S)
    pub fn circuit_2() -> Self {
        Self::generate_from_centerline(
            "Circuito 2 — Serpentina Técnica",
            &[
                Vec2::new(180.0, 150.0),
                Vec2::new(450.0, 130.0),
                Vec2::new(750.0, 150.0),
                Vec2::new(850.0, 300.0),
                Vec2::new(650.0, 340.0),
                Vec2::new(400.0, 380.0),
                Vec2::new(200.0, 420.0),
                Vec2::new(350.0, 560.0),
                Vec2::new(650.0, 600.0),
                Vec2::new(820.0, 580.0),
                Vec2::new(850.0, 660.0),
                Vec2::new(450.0, 690.0),
                Vec2::new(150.0, 620.0),
                Vec2::new(120.0, 350.0),
            ],
            65.0,
        )
    }

    /// Circuito 3: Grand Prix Retrô (Traçado longo com grampo fechado)
    pub fn circuit_3() -> Self {
        Self::generate_from_centerline(
            "Circuito 3 — Grand Prix Retrô",
            &[
                Vec2::new(180.0, 180.0),
                Vec2::new(500.0, 150.0),
                Vec2::new(800.0, 180.0),
                Vec2::new(880.0, 350.0),
                Vec2::new(720.0, 420.0),
                Vec2::new(520.0, 320.0),
                Vec2::new(350.0, 340.0),
                Vec2::new(500.0, 480.0),
                Vec2::new(780.0, 550.0),
                Vec2::new(850.0, 660.0),
                Vec2::new(550.0, 680.0),
                Vec2::new(200.0, 650.0),
                Vec2::new(120.0, 420.0),
            ],
            60.0,
        )
    }

    fn generate_from_centerline(name: &str, points: &[Vec2], track_width: f32) -> Self {
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

