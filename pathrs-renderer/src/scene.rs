use glam::{vec3, Vec3};

use crate::{Ray, material::Material};

#[cfg(feature = "simd")]
use crate::simd::*;

#[derive(Default, Clone)]
pub struct Scene {
    objects: Vec<SceneObject>,
    spheres: Spheres,
    triangles: Triangles,

    pub materials: Vec<Material>,

    #[cfg(feature = "simd")]
    triangles_simd: TrianglesSIMD,
    #[cfg(feature = "simd")]
    spheres_simd: SpheresSIMD,
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

    pub fn add_material<M: Into<Material>>(&mut self, mat: M) -> u32 {
        self.materials.push(mat.into());
        (self.materials.len() - 1) as u32
    }

    #[cfg(feature = "simd")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn collect_simd(&mut self) {
        self.triangles_simd.from_tris(self.triangles.clone());
        self.spheres_simd = SpheresSIMD::from_spheres(self.spheres.clone());
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
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


        #[cfg(feature = "simd")]
        let r_o = Vec3x8::from(ray.origin);
        #[cfg(feature = "simd")]
        let r_d = Vec3x8::from(ray.direction);

        #[cfg(not(feature = "simd"))]
        if let Some(hit) = self.spheres.intersect_spheres(ray, tmin, tmax) {
            tmax = hit.t;
            res = Some(hit);
        }

        #[cfg(feature = "simd")]
        if let Some(hit) = self.spheres_simd.intersect(ray, tmin, tmax, r_o, r_d) {
            tmax = hit.t;
            res = Some(hit);
        }

        #[cfg(not(feature = "simd"))]
        if let Some(hit) = self.triangles.intersect(ray, tmin, tmax) {
            res = Some(hit);
        }

        #[cfg(feature = "simd")]
        if let Some(hit) = self.triangles_simd.intersect(ray, tmin, tmax, r_o, r_d) {
            res = Some(hit);
        }

        res
    }
}

#[derive(Clone)]
pub struct HitRecord {
    pub pos: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material: u32,
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
    pos: Vec3,
    r: f32,
    r_squared: f32,
    material: u32,
}

impl Sphere {
    pub fn new(pos: Vec3, r: f32, material: u32) -> Sphere {
        Sphere {
            pos,
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
                material: self.material
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
    materials: Vec<u32>,
}

impl Spheres {
    fn add(&mut self, s: &Sphere) {
        self.s_x.push(s.pos.x);
        self.s_y.push(s.pos.y);
        self.s_z.push(s.pos.z);
        self.r_squared.push(s.r_squared);
        self.r_inv.push(1.0 / s.r);
        self.materials.push(s.material);
    }

    #[inline(always)]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
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
                        (pos - vec3(self.s_x[i], self.s_y[i], self.s_z[i])) * self.r_inv[i];

                    closest = t;
                    hit = Some(HitRecord {
                        pos,
                        normal,
                        t,
                        material: self.materials[i],
                    })
                }

                let t = (h + sqrtd) * a_inv;
                if t > tmin && t < tmax && t < closest {
                    let pos = ray.origin + t * ray.direction;
                    let normal =
                        (pos - vec3(self.s_x[i], self.s_y[i], self.s_z[i])) * self.r_inv[i];

                    closest = t;
                    hit = Some(HitRecord {
                        pos,
                        normal,
                        t,
                        material: self.materials[i],
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
    v0: Vec3,
    e1: Vec3,
    e2: Vec3,
    normal: Vec3,
    material: u32,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: u32) -> Self {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let normal = e1.cross(e2).normalize();
        Self {
            v0: v0.into(),
            e1,
            e2,
            normal,
            material,
        }
    }

    pub fn quad(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, material: u32) -> [Triangle; 2] {
        [
            Self::new(p0, p1, p2, material),
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
                material: self.material,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Default)]
pub struct Triangles {
    count: usize,
    v0: Vec<Vec3>,
    e1: Vec<Vec3>,
    e2: Vec<Vec3>,
    normal: Vec<Vec3>,
    material: Vec<u32>,
}

impl Triangles {
    pub fn push(&mut self, tri: Triangle) {
        self.count += 1;
        self.v0.push(tri.v0);
        self.e1.push(tri.e1);
        self.e2.push(tri.e2);
        self.normal.push(tri.normal);
        self.material.push(tri.material);
    }

    #[inline(always)]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut closest = tmax;
        let mut idx = None;
        let mut pos = Vec3::ZERO;

        for i in 0..self.count {
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
                material: self.material[i],
            })
        } else {
            None
        }
    }
}

#[cfg(feature = "simd")]
#[derive(Clone, Default)]
pub struct TrianglesSIMD {
    packed_count: usize,

    v0_x: Vec<f32x8>,
    v0_y: Vec<f32x8>,
    v0_z: Vec<f32x8>,

    e1_x: Vec<f32x8>,
    e1_y: Vec<f32x8>,
    e1_z: Vec<f32x8>,

    e2_x: Vec<f32x8>,
    e2_y: Vec<f32x8>,
    e2_z: Vec<f32x8>,

    normal: Vec<Vec3>,
    material: Vec<u32>,
}

#[cfg(feature = "simd")]
macro_rules! pack_f32x8 {
    ($src:expr, $field:tt, $i:ident) => {
        f32x8::from_array([
            $src[$i].$field,
            $src[$i + 1].$field,
            $src[$i + 2].$field,
            $src[$i + 3].$field,
            $src[$i + 4].$field,
            $src[$i + 5].$field,
            $src[$i + 6].$field,
            $src[$i + 7].$field,
        ])
    };
}

macro_rules! push8 {
    ($dst:expr, $src:expr, $i:ident) => {
        $dst.push($src[$i]);
        $dst.push($src[$i + 1]);
        $dst.push($src[$i + 2]);
        $dst.push($src[$i + 3]);
        $dst.push($src[$i + 4]);
        $dst.push($src[$i + 5]);
        $dst.push($src[$i + 6]);
        $dst.push($src[$i + 7]);
    };
}

#[cfg(feature = "simd")]
impl TrianglesSIMD {
    pub fn from_tris(&mut self, tris: Triangles) {
        let count = tris.count / 8;
        let mut i = 0;
        while i < count {
            let base = i * 8;
            self.v0_x.push(pack_f32x8!(tris.v0, x, base));
            self.v0_y.push(pack_f32x8!(tris.v0, y, base));
            self.v0_z.push(pack_f32x8!(tris.v0, z, base));

            self.e1_x.push(pack_f32x8!(tris.e1, x, base));
            self.e1_y.push(pack_f32x8!(tris.e1, y, base));
            self.e1_z.push(pack_f32x8!(tris.e1, z, base));

            self.e2_x.push(pack_f32x8!(tris.e2, x, base));
            self.e2_y.push(pack_f32x8!(tris.e2, y, base));
            self.e2_z.push(pack_f32x8!(tris.e2, z, base));

            push8!(self.normal, tris.normal, base);
            push8!(self.material, tris.material, base);

            self.packed_count += 1;

            i += 1;
        }
    }

    #[inline(always)]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32, r_o: Vec3x8, r_d: Vec3x8) -> Option<HitRecord> {
        let tmin = f32x8::splat(tmin);

        let mut closest = f32x8::splat(tmax);
        let mut closest_idx = i32x8::splat(-1);
        let mut closest_det = f32x8::ZERO;

        let mut tri_idx = i32x8::from_array([0, 1, 2, 3, 4, 5, 6, 7]);
        let stride = i32x8::splat(8);

        for i in 0..self.packed_count {
            let v0 = Vec3x8 {
                x: self.v0_x[i],
                y: self.v0_y[i],
                z: self.v0_z[i],
            };
            let e1 = Vec3x8 {
                x: self.e1_x[i],
                y: self.e1_y[i],
                z: self.e1_z[i],
            };
            let e2 = Vec3x8 {
                x: self.e2_x[i],
                y: self.e2_y[i],
                z: self.e2_z[i],
            };

            let ray_cross_e2 = r_d.cross(e2);

            let det = e1.dot(ray_cross_e2);

            let inv_det = f32x8::ONE / det;

            let s = r_o - v0;

            let s_cross_e1 = s.cross(e1);

            let u = inv_det * s.dot(ray_cross_e2);
            let v = inv_det * r_d.dot(s_cross_e1);

            let t = inv_det * e2.dot(s_cross_e1);

            let mut misses = det.cmp_gt(f32x8::NEGATIVE_EPSILON) & det.cmp_lt(f32x8::EPSILON);
            misses |= u.cmp_lt(f32x8::ZERO);
            misses |= v.cmp_lt(f32x8::ZERO);
            misses |= (u + v).cmp_gt(f32x8::ONE);
            misses |= t.cmp_lt(f32x8::EPSILON);
            misses |= t.cmp_lt(tmin);
            misses |= t.cmp_gt(closest);

            closest = f32x8::blend(t, closest, misses);
            closest_idx = i32x8::select(tri_idx, closest_idx, misses);
            closest_det = f32x8::blend(det, closest_det, misses);

            tri_idx += stride;
        }

        let mask = closest_idx.transmute_f32x8().movemask();

        if mask == 0xFF {
            None
        } else {
            let v1 = f32x8::min(closest, closest.permute::<0b10_11_00_01>());
            let v2 = f32x8::min(v1, v1.permute::<0b01_00_11_10>());
            let v3 = f32x8::min(v2, f32x8::permute_2f128::<0b0000_0001>(v2, v2));

            let t = v3[0];

            let mask = v3.cmp_eq(closest).as_f32x8().movemask();
            let i = mask.trailing_zeros() as usize;

            let tri_idx = closest_idx[i] as usize;
            let det = closest_det[i];

            let pos = ray.origin + t * ray.direction;

            let mut normal = self.normal[tri_idx];
            if det < 0.0 {
                normal = -normal;
            }

            let material = self.material[tri_idx];

            Some(HitRecord {
                pos,
                normal,
                t,
                material,
            })
        }
    }
}

#[cfg(feature = "simd")]
#[derive(Clone, Default)]
pub struct SpheresSIMD {
    packed_count: usize,

    pos_x: Vec<f32x8>,
    pos_y: Vec<f32x8>,
    pos_z: Vec<f32x8>,

    r_squared: Vec<f32x8>,

    r_inv: Vec<f32>,
    material: Vec<u32>,
}

#[cfg(feature = "simd")]
impl SpheresSIMD {
    fn from_spheres(spheres: Spheres) -> SpheresSIMD {
        let mut pos_x = Vec::new();
        let mut pos_y = Vec::new();
        let mut pos_z = Vec::new();

        let mut r_squared = Vec::new();

        let mut r_inv = Vec::new();
        let mut material = Vec::new();

        let count = spheres.s_x.len();
        let mut i = 0;

        while i < count {
            let mut x = [0.0; 8];
            let mut y = [0.0; 8];
            let mut z = [0.0; 8];
            let mut rsq = [0.0; 8];

            let max_j = (count - i).min(8);

            for j in 0..max_j {
                x[j] = spheres.s_x[i + j];
                y[j] = spheres.s_y[i + j];
                z[j] = spheres.s_z[i + j];
                rsq[j] = spheres.r_squared[i + j];

                r_inv.push(spheres.r_inv[i + j]);
                material.push(spheres.materials[i + j]);
            }

            for _ in max_j..8 {
                r_inv.push(0.0);
                material.push(0);
            }

            pos_x.push(f32x8::from_array(x));
            pos_y.push(f32x8::from_array(y));
            pos_z.push(f32x8::from_array(z));
            r_squared.push(f32x8::from_array(rsq));

            i += 8;
        }

        let packed_count = pos_x.len();

        SpheresSIMD {
            packed_count,
            pos_x,
            pos_y,
            pos_z,
            r_squared,
            r_inv,
            material,
        }
    }

    #[inline(always)]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn intersect(&self, ray: &Ray, tmin: f32, tmax: f32, r_o: Vec3x8, r_d: Vec3x8) -> Option<HitRecord> {
        let a = f32x8::splat(ray.direction.length_squared());
        let a_inv = f32x8::splat(1.0 / a[0]);

        let tmin = f32x8::splat(tmin);

        let mut closest_t = f32x8::splat(tmax);
        let mut closest_idx = i32x8::splat(-1);

        let mut s_idx = i32x8::from_array([0, 1, 2, 3, 4, 5, 6, 7]);
        let stride = i32x8::splat(8);

        for i in 0..self.packed_count {
            let pos = Vec3x8 {
                x: self.pos_x[i],
                y: self.pos_y[i],
                z: self.pos_z[i],
            };

            let oc = pos - r_o;
            let h = r_d.dot(oc);
            let c = oc.dot(oc) - self.r_squared[i];
            let disc = h * h - a * c;

            let sqrtd = disc.sqrt();

            let t = f32x8::min((h - sqrtd) * a_inv, (h + sqrtd) * a_inv);

            let mut miss = disc.cmp_lt(f32x8::ZERO);
            miss |= t.cmp_lt(tmin);
            miss |= t.cmp_gt(closest_t);

            closest_t = f32x8::blend(t, closest_t, miss);
            closest_idx = i32x8::select(s_idx, closest_idx, miss);

            s_idx += stride;
        }

        let mask = closest_idx.transmute_f32x8().movemask();

        if mask == 0xFF {
            None
        } else {
            let v1 = f32x8::min(closest_t, closest_t.permute::<0b10_11_00_01>());
            let v2 = f32x8::min(v1, v1.permute::<0b01_00_11_10>());
            let v3 = f32x8::min(v2, f32x8::permute_2f128::<0b0000_0001>(v2, v2));

            let t = v3[0];

            let mask = v3.cmp_eq(closest_t).as_f32x8().movemask();
            let i = mask.trailing_zeros() as usize;

            let s_idx = closest_idx[i] as usize;

            let s_pack = s_idx / 8;
            let s_lane = s_idx % 8;
            let s_pos = vec3(
                self.pos_x[s_pack][s_lane],
                self.pos_y[s_pack][s_lane],
                self.pos_z[s_pack][s_lane],
            );

            let pos = ray.origin + t * ray.direction;

            let normal = (pos - s_pos) * self.r_inv[s_idx];

            let material = self.material[s_idx];

            Some(HitRecord {
                pos,
                normal,
                t,
                material,
            })
        }
    }
}
