use std::rc::Rc;
use std::sync::Arc;
use crate::hit::HitRecord;
use crate::{Hit, Point3, Ray, Vec3};
use crate::material::Scatter;

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Scatter>,
}

impl Sphere {
    pub fn new(c: Point3, r: f64, m: Arc<dyn Scatter>) -> Sphere {
        Sphere {
            center: c,
            radius: r,
            mat: m,
        }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center;  // difference between ray origin and center of circle

        // **simplified** quadratic formula
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.length().powi(2) - self.radius.powi(2);
        let discriminant = half_b*half_b - a*c;

        if discriminant < 0.0 {
            return None;
        }

        // Find the nearest root that lies in the acceptable range
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let p = r.at(root);
        let mut rec = HitRecord {
            p,
            normal: Vec3::new(0.0, 0.0, 0.0),
            mat: self.mat.clone(),
            t: root,
            front_face: false,
        };

        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}