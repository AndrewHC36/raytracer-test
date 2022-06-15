use rand::Rng;
use crate::{Color, Ray, Vec3};
use crate::hit::HitRecord;

pub trait Scatter : Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color
}

impl Lambertian {
    pub fn new(a: Color) -> Lambertian {
        Lambertian {
            albedo: a
        }
    }
}

impl Scatter for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        // simple diffuse model
        // let target = rec.p + rec.normal + Vec3::rand_in_unit_sphere();
        // true lambertian reflection
        // let target = rec.p + rec.normal + Vec3::rand_in_unit_sphere().normalized();
        // hemisphere diffuse model (alternative model)
        // let target = rec.p + Vec3::rand_in_hemisphere(rec.normal);


        let mut scatter_dir = rec.normal + Vec3::rand_in_unit_sphere().normalized();
        if scatter_dir.near_zero() {
            // Catches degenerate scatter direction
            scatter_dir = rec.normal;
        }
        let scattered = Ray::new(rec.p, scatter_dir);

        Some((self.albedo, scattered))
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(a: Color, f: f64) -> Metal {
        Metal {
            albedo: a,
            fuzz: f,
        }
    }
}

impl Scatter for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = r_in.direction().reflect(rec.normal).normalized();
        let scattered = Ray::new(rec.p, reflected + self.fuzz*Vec3::rand_in_unit_sphere());

        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Dielectric {
        Dielectric {
            ir: index_of_refraction,
        }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // Using Schlick's approximation for reflectance
        let r0 = ((1.0-ref_idx) / (1.0+ref_idx)).powi(2);
        r0 + (1.0-r0)*(1.0-cosine).powi(5)
    }
}

impl Scatter for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let refr_rat = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_dir = r_in.direction().normalized();
        let cos_theta = ((-1.0)*unit_dir).dot(rec.normal).min(1.0);
        let sin_theta = (1.0-cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refr = refr_rat*sin_theta > 1.0;
        let will_refl = rng.gen::<f64>() < Self::reflectance(cos_theta, refr_rat);

        let dir = if cannot_refr || will_refl {
            unit_dir.reflect(rec.normal)
        } else {
            unit_dir.refract(rec.normal, refr_rat)
        };

        let scattered = Ray::new(rec.p, dir);

        Some((Color::new(1.0, 1.0, 1.0), scattered))
    }
}