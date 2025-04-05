use std::sync::Arc;

use bevy::math::{Vec3, Vec3A, vec3a};

use crate::{material::Material, renderer::Ray};

#[derive(Default, Clone)]
pub struct Scene {
    objects: Vec<SceneObject>,
    spheres: Spheres,
    triangles: Triangles,
}

impl Scene {
    pub fn add_object<T: Into<SceneObject>>(&mut self, object: T) {
        let so = object.into();
        match so.clone() {
            SceneObject::Sphere(s) => self.spheres.add(&s),
            // SceneObject::Quad(_) => self.objects.push(so),
        };
    }

    pub fn add_triangle(&mut self, tri: Triangle) {
        self.triangles.push(tri);
    }

    pub fn add_triangles(&mut self, tris: &[Triangle]) {
        for tri in tris {
            self.triangles.push(tri.clone());
        }
    }

    pub fn closest_hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut res = None;
        let mut tmax = tmax;

        // for s in self.spheres.iter() {
        //     if let Some(intersection) = s.intersect(ray, tmin, tmax) {
        //         tmax = intersection.t;
        //         res = Some(intersection);
        //     }
        // }
        // for q in self.quads.iter() {
        //     if let Some(intersection) = q.intersect(ray, tmin, tmax) {
        //         tmax = intersection.t;
        //         res = Some(intersection);
        //     }
        // }

        if let Some(hit) = self.spheres.intersect_spheres(ray, tmin, tmax) {
            tmax = hit.t;
            res = Some(hit);
        }

        // for obj in self.objects.iter() {
        //     if let Some(intersection) = obj.intersect(ray, tmin, tmax) {
        //         tmax = intersection.t;
        //         res = Some(intersection);
        //     }
        // }

        // for tri in self.triangles.iter() {
        //     if let Some(hit) = tri.intersect(ray, tmin, tmax) {
        //         tmax = hit.t;
        //         res = Some(hit);
        //     }
        // }

        if let Some(hit) = self.triangles.intersect(ray, tmin, tmax) {
            res = Some(hit);
        }

        res
    }
}

#[derive(Clone)]
pub struct HitRecord {
    pub pos: Vec3A,
    pub normal: Vec3A,
    pub t: f32,
    pub material: Arc<dyn Material + Sync + Send>,
}

#[derive(Clone)]
pub enum SceneObject {
    Sphere(Sphere),
    // Quad(Quad),
}

impl SceneObject {
    fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        match self {
            SceneObject::Sphere(s) => s.intersect(ray, tmin, tmax),
            // SceneObject::Quad(q) => q.intersect(ray, tmin, tmax),
        }
    }
}

impl From<Sphere> for SceneObject {
    fn from(value: Sphere) -> Self {
        Self::Sphere(value)
    }
}

// impl From<Quad> for SceneObject {
//     fn from(value: Quad) -> Self {
//         Self::Quad(value)
//     }
// }

#[derive(Clone)]
pub struct Sphere {
    pos: Vec3A,
    r: f32,
    r_squared: f32,
    material: Arc<dyn Material + Sync + Send>,
}

impl Sphere {
    pub fn new(pos: Vec3, r: f32, material: Arc<dyn Material + Sync + Send>) -> Sphere {
        Sphere {
            pos: pos.into(),
            r,
            r_squared: r * r,
            material,
        }
    }

    #[inline(always)]
    fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let oc = self.pos - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.r_squared;
        let disc = h * h - a * c;

        if disc < 0.0 {
            None
        } else {
            let sqrtd = disc.sqrt();

            let mut t = (h - sqrtd) / a;
            if t <= tmin || tmax <= t {
                t = (h + sqrtd) / a;
                if t <= tmin || tmax <= t {
                    return None;
                }
            }

            let pos = ray.origin + t * ray.direction;

            Some(HitRecord {
                pos,
                normal: (pos - self.pos) / self.r,
                t,
                material: self.material.clone(),
            })
        }
    }
}

#[derive(Clone, Default)]
struct Spheres {
    s_x: Vec<f32>,
    s_y: Vec<f32>,
    s_z: Vec<f32>,
    r_squared: Vec<f32>,
    r_inv: Vec<f32>,
    materials: Vec<Arc<dyn Material + Sync + Send>>,
}

impl Spheres {
    fn add(&mut self, s: &Sphere) {
        self.s_x.push(s.pos.x);
        self.s_y.push(s.pos.y);
        self.s_z.push(s.pos.z);
        self.r_squared.push(s.r_squared);
        self.r_inv.push(1.0 / s.r);
        self.materials.push(s.material.clone());
    }

    #[inline(always)]
    fn intersect_spheres(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut closest = tmax;
        let mut hit = None;
        let a = ray.direction.length_squared();
        let a_inv = 1.0 / a;

        for i in 0..self.s_x.len() {
            let oc_x = self.s_x[i] - ray.origin.x;
            let oc_y = self.s_y[i] - ray.origin.y;
            let oc_z = self.s_z[i] - ray.origin.z;

            let h = ray.direction.x * oc_x + ray.direction.y * oc_y + ray.direction.z * oc_z;
            let c = oc_x * oc_x + oc_y * oc_y + oc_z * oc_z - self.r_squared[i];
            let disc = h * h - a * c;

            if disc > 0.0 {
                let sqrtd = disc.sqrt();

                let t = (h - sqrtd) * a_inv;
                if t > tmin && t < tmax && t < closest {
                    let pos = ray.origin + t * ray.direction;
                    let normal =
                        (pos - vec3a(self.s_x[i], self.s_y[i], self.s_z[i])) * self.r_inv[i];

                    closest = t;
                    hit = Some(HitRecord {
                        pos,
                        normal,
                        t,
                        material: self.materials[i].clone(),
                    })
                }

                let t = (h + sqrtd) * a_inv;
                if t > tmin && t < tmax && t < closest {
                    let pos = ray.origin + t * ray.direction;
                    let normal =
                        (pos - vec3a(self.s_x[i], self.s_y[i], self.s_z[i])) * self.r_inv[i];

                    closest = t;
                    hit = Some(HitRecord {
                        pos,
                        normal,
                        t,
                        material: self.materials[i].clone(),
                    })
                }
            }
        }

        hit
    }
}

// #[derive(Clone)]
// pub struct Quad {
//     q: Vec3,
//     u: Vec3,
//     v: Vec3,
//     w: Vec3,
//     material: Arc<dyn Material + Sync + Send>,
//     normal: Vec3,
//     d: f32,
// }
//
// impl Quad {
//     pub fn new(q: Vec3, u: Vec3, v: Vec3, material: Arc<dyn Material + Sync + Send>) -> Self {
//         let n = u.cross(v);
//         let normal = n.normalize();
//         Quad {
//             q,
//             u,
//             v,
//             w: n / n.dot(n),
//             material,
//             normal,
//             d: normal.dot(q),
//         }
//     }
//
//     #[inline(always)]
//     pub fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
//         let denom = self.normal.dot(ray.direction);
//
//         if denom.abs() < 1e-8 {
//             return None;
//         }
//
//         let t = (self.d - self.normal.dot(ray.origin)) / denom;
//
//         if t <= tmin || tmax <= t {
//             return None;
//         }
//
//         let pos = ray.origin + t * ray.direction;
//         let planar_hitpt_vector = pos - self.q;
//         let alpha = self.w.dot(planar_hitpt_vector.cross(self.v));
//         let beta = self.w.dot(self.u.cross(planar_hitpt_vector));
//
//         if !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&beta) {
//             return None;
//         }
//
//         let normal = if ray.direction.dot(self.normal) < 0.0 {
//             self.normal
//         } else {
//             -self.normal
//         };
//
//         Some(HitRecord {
//             pos,
//             normal,
//             t,
//             material: self.material.clone(),
//         })
//     }
// }

#[derive(Clone)]
pub struct Triangle {
    v0: Vec3A,
    e1: Vec3A,
    e2: Vec3A,
    normal: Vec3A,
    material: Arc<dyn Material + Send + Sync>,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: Arc<dyn Material + Sync + Send>) -> Self {
        let e1 = Vec3A::from(v1 - v0);
        let e2 = Vec3A::from(v2 - v0);
        let normal = e1.cross(e2).normalize().into();
        Self {
            v0: v0.into(),
            e1,
            e2,
            normal,
            material,
        }
    }

    pub fn quad(
        p0: Vec3,
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        material: Arc<dyn Material + Sync + Send>,
    ) -> [Triangle; 2] {
        [
            Self::new(p0, p1, p2, material.clone()),
            Self::new(p1, p3, p2, material),
        ]
    }

    pub fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let ray_cross_e2 = ray.direction.cross(self.e2);
        let det = self.e1.dot(ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let s = ray.origin - self.v0;
        let u = inv_det * s.dot(ray_cross_e2);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let s_cross_e1 = s.cross(self.e1);
        let v = inv_det * ray.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = inv_det * self.e2.dot(s_cross_e1);

        if t > f32::EPSILON && tmin < t && tmax > t {
            let pos = ray.origin + ray.direction * t;
            let normal = if ray.direction.dot(self.normal) < 0.0 {
                self.normal
            } else {
                -self.normal
            };

            Some(HitRecord {
                pos,
                normal,
                t,
                material: self.material.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Default)]
pub struct Triangles {
    v0: Vec<Vec3A>,
    e1: Vec<Vec3A>,
    e2: Vec<Vec3A>,
    normal: Vec<Vec3A>,
    material: Vec<Arc<dyn Material + Send + Sync>>,
}

impl Triangles {
    pub fn push(&mut self, tri: Triangle) {
        self.v0.push(tri.v0);
        self.e1.push(tri.e1);
        self.e2.push(tri.e2);
        self.normal.push(tri.normal);
        self.material.push(tri.material);
    }

    #[inline(always)]
    pub fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut closest = tmax;
        let mut idx = None;
        let mut pos = Vec3A::ZERO;

        for i in 0..self.v0.len() {
            let ray_cross_e2 = ray.direction.cross(self.e2[i]);
            let det = self.e1[i].dot(ray_cross_e2);

            if det > -f32::EPSILON && det < f32::EPSILON {
                continue;
            }

            let inv_det = 1.0 / det;
            let s = ray.origin - self.v0[i];
            let u = inv_det * s.dot(ray_cross_e2);
            if !(0.0..=1.0).contains(&u) {
                continue;
            }

            let s_cross_e1 = s.cross(self.e1[i]);
            let v = inv_det * ray.direction.dot(s_cross_e1);
            if v < 0.0 || u + v > 1.0 {
                continue;
            }

            let t = inv_det * self.e2[i].dot(s_cross_e1);

            if t > f32::EPSILON && tmin < t && closest > t {
                closest = t;
                idx = Some(i);
                pos = ray.origin + ray.direction * t;
            }
        }

        if let Some(i) = idx {
            let normal = if ray.direction.dot(self.normal[i]) < 0.0 {
                self.normal[i]
            } else {
                -self.normal[i]
            };
            Some(HitRecord {
                pos,
                normal,
                t: closest,
                material: self.material[i].clone(),
            })
        } else {
            None
        }
    }
}
