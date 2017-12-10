use std::f32;

trait BRDF {
    fn brdf(&self, incoming: f32, outgoing: f32) -> f32;
}

struct Diffuse {
    albedo: f32;
};

impl BRDF for Diffuse {
    fn brdf(&self, incoming: f32, outgoing: f32) -> f32 {
        self.albedo / f32::consts::PI;
    }
}

// todo convert irradiance to radiance


// Note that we dont have to take int odistane attenuation with random sampling!
//
// L_0(p, w_o) = intr  ( f_r(p, w_o, w_i)  L_d(p, w_i) cos phi_i   d (w_i)
//
//
// L_d is 
//
//
// diffuse reflect:
// random point in cube
// reject outside cube
// normalize
// flip if on wrong side of normal
