use crate::{Point3, Ray, Vec3};


#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    cu: Vec3,
    cv: Vec3,
    lens_radius: f64
}

impl Camera {
    pub fn new(lookfrom: Point3,
               lookat: Point3,
               vup: Vec3,
               vert_fov: f64,
               aspect_ratio: f64,
               aperture: f64,
               focus_dist: f64) -> Camera {
        // Converting FOV into radians
        let theta = std::f64::consts::PI / 180.0 * vert_fov;
        let vph = 2.0 * (theta/2.0).tan();
        let vpw = aspect_ratio * vph;

        let cw = (lookfrom - lookat).normalized();
        let cu = vup.cross(cw).normalized();
        let cv = cw.cross(cu);

        let h = focus_dist * vpw * cu;
        let v = focus_dist * vph * cv;
        let llc = lookfrom - h/2.0 - v/2.0 - focus_dist * cw;

        Camera {
            origin: lookfrom,
            horizontal: h,
            vertical: v,
            cu,
            cv,
            lower_left_corner: llc,
            lens_radius: aperture/2.0,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let rd = self.lens_radius * Vec3::rand_in_unit_disk();
        let offset = self.cu * rd.x() + self.cv * rd.y();

        Ray::new(self.origin + offset,
                 self.lower_left_corner + u*self.horizontal + v*self.vertical - self.origin - offset
        )
    }
}