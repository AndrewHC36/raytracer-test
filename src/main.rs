mod vec3;
mod ray;
mod hit;
mod sphere;
mod camera;
mod material;

use std::fs::File;
use std::io::{BufWriter, stderr, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use png;
use rand::Rng;
use threadpool::ThreadPool;
use crate::camera::Camera;
use crate::hit::{Hit, World};
use crate::material::{Dielectric, Lambertian, Metal};
use crate::vec3::{Color, Point3, Vec3};
use crate::ray::Ray;
use crate::sphere::Sphere;


// gets the color of the ray at intersection
fn ray_color(r: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        // Exceeding the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        // material (description of ray behaviour)
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            attenuation * ray_color(&scattered, world, depth-1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        // background
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t*Color::new(0.5, 0.7, 1.0)
    }
}

fn main() {
    // Image
    const ASPECT_RATIO: f64 = 3.0 / 2.0;
    const IMAGE_WIDTH: u32 = 1200;
    const IMAGE_HEIGHT: u32 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u32;
    const SAMPLES_PER_PIXEL: u32 = 500;
    const MAX_DEPTH: u64 = 50;

    // World
    let mut world = World::new();

    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_center = Arc::new(Dielectric::new(1.1));
    // let mat_left = Rc::new(Dielectric::new(1.5));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));
    let mat_left_inner = Arc::new(Dielectric::new(1.5));

    world.push(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground)));
    world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center)));
    // world.push(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left)));
    world.push(Box::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right)));
    world.push(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.4, mat_left_inner)));

    let arc_world = Arc::new(world);

    // Camera
    let lookfrom = Point3::new(12.0, 3.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, -1.0);
    let cam = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        ASPECT_RATIO,
        0.1,
        (lookfrom - lookat).length()
    );

    // Rendering
    // let mut vec_data = vec![0; (IMAGE_HEIGHT*IMAGE_WIDTH*3) as usize];
    // let mut data_slice = vec_data.as_mut_slice();
    let mut data = Arc::new(Mutex::new(vec![0; (IMAGE_HEIGHT*IMAGE_WIDTH*3) as usize]));

    let pool = threadpool::Builder::new()
        .num_threads(8)
        .thread_stack_size(2_000_000)
        .build();

    for y in 0..IMAGE_HEIGHT {
            let data_clone = data.clone();
            let arc_world = arc_world.clone();
            pool.execute(move || {
                eprintln!("Thread: y:{} -- STARTED", y);
                stderr().flush().unwrap();

                for x in 0..IMAGE_WIDTH {
                    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                    for _ in 0..SAMPLES_PER_PIXEL {
                        let mut rng = rand::thread_rng();

                        let rand_u: f64 = rng.gen();
                        let rand_v: f64 = rng.gen();

                        let u = ((x as f64) + rand_u) / ((IMAGE_WIDTH - 1) as f64);
                        let v = ((y as f64) + rand_v) / ((IMAGE_HEIGHT - 1) as f64);

                        // ray generation from the camera (0,0,0) to corresponding coordinates of each pixel of the output image
                        let r = cam.get_ray(u, v);
                        pixel_color += ray_color(&r, &arc_world, MAX_DEPTH);
                    }
                    let (r, g, b) = pixel_color.color_rgb(SAMPLES_PER_PIXEL);
                    let mut data = data_clone.lock().unwrap();
                    data[((IMAGE_HEIGHT-y-1)*IMAGE_WIDTH*3 + x*3 + 0) as usize] = r;
                    data[((IMAGE_HEIGHT-y-1)*IMAGE_WIDTH*3 + x*3 + 1) as usize] = g;
                    data[((IMAGE_HEIGHT-y-1)*IMAGE_WIDTH*3 + x*3 + 2) as usize] = b;
                }
                eprintln!("Thread: y:{} ## Completed", y);
                stderr().flush().unwrap();
            });
    }
    pool.join();

    let path = Path::new("./src/output.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, IMAGE_WIDTH, IMAGE_HEIGHT);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    {
        let mut rdt = data.lock().expect("Failed to retrieve image data");
        writer.write_image_data(rdt.as_mut_slice()).expect("Fail to save the image");
    }
}
