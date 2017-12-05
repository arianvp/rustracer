use cgmath::{Point3, Vector3, ElementWise};
use std::path::Path;
use tracer::primitive::triangle::Triangle;
use tracer::primitive::Material;
use tobj;

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn from_tobj_to_mesh(mesh: &tobj::Mesh, translation: Vector3<f32>, scale: f32) -> Mesh {
        let positions: Vec<Point3<f32>> = mesh.positions
            .chunks(3)
            .map(|i| Point3::new(i[0], i[1], i[2]))
            .collect();
        let normals: Vec<Vector3<f32>> = mesh.normals
            .chunks(3)
            .map(|i| Vector3::new(i[0], i[1], i[2]))
            .collect();

        Mesh {
            triangles: mesh.indices
                .chunks(3)
                .map(|indices| {
                    Triangle {
                        material: Material::Dielectric{ n1: 1.0, n2: 1.21, absorbance: Vector3::new(0.0, 0.0, 0.0) },
                        //material: Material::Conductor{spec:1.0, color:Vector3::new(1.0,0.0,0.0)} ,
                        p0: (positions[indices[0] as usize] * scale) + translation,
                        p1: (positions[indices[1] as usize] * scale) + translation,
                        p2: (positions[indices[2] as usize] * scale) + translation,
                        normal: normals[indices[0] as usize], // TODO: normal interpolation!!!!
                    }
                })
                .collect(),
        }
    }

    /// TODO I load simply one scene now
    pub fn load_from_path(path: &Path, translation: Vector3<f32>, scale: f32) -> Result<Mesh, tobj::LoadError> {
        let (models, materials) = tobj::load_obj(path)?;
        Ok(Mesh::from_tobj_to_mesh(&models[0].mesh, translation, scale))
    }
}
