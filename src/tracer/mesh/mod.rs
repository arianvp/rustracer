use nalgebra::{Point3, Vector3};
use std::path::Path;
use tracer::primitive::triangle::Triangle;
use tracer::primitive::Material;
use tracer::primitive::Primitive;
use tracer::primitive::Intersection;
use std::cmp::Ordering;
use bvh::ray::Ray;
use bvh::bvh::BVH;
use tobj;

pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub bvh: BVH,
}

impl Primitive for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let triangles = self.bvh.traverse(ray, &self.triangles);
        triangles
            .iter()
            .filter_map(|t| t.intersect(ray))
            .min_by(|a, b| {
                a.distance.partial_cmp(&b.distance).unwrap_or(
                    Ordering::Equal,
                )
            })
    }
}

impl Mesh {
    /// TODO I load simply one scene now
    pub fn load_from_path(
        path: &Path,
        translation: Vector3<f32>,
        scale: f32,
        material: Material,
    ) -> Result<Mesh, tobj::LoadError> {
        let (models, materials) = tobj::load_obj(path)?;

        let mut indices = vec![];
        let mut positions = vec![];
        let mut normals = vec![];

        for model in models {
            indices.extend(model.mesh.indices);
            positions.extend(model.mesh.positions);
            normals.extend(model.mesh.normals);
        }

        let indices = indices.chunks(3);
        let positions: Vec<_> = positions
            .chunks(3)
            .map(|p| Point3::new(p[0], p[1], p[2]))
            .collect();
        let normals: Vec<_> = normals
            .chunks(3)
            .map(|n| Vector3::new(n[0], n[1], n[2]))
            .collect();

        let mut triangles: Vec<_> = indices
            .map(|indices| {
                Triangle {
                    material: material.clone(),
                    //material: Material::Conductor{spec:1.0, color:Vector3::new(1.0,0.0,0.0)} ,
                    p0: (positions[indices[0] as usize] * scale) + translation,
                    p1: (positions[indices[1] as usize] * scale) + translation,
                    p2: (positions[indices[2] as usize] * scale) + translation,
                    n0: normals[indices[0] as usize], // TODO: normal interpolation!!!!
                    n1: normals[indices[1] as usize], // TODO: normal interpolation!!!!
                    n2: normals[indices[2] as usize], // TODO: normal interpolation!!!!
                    node_index: 0,
                }
            })
            .collect();

        let bvh = BVH::build(&mut triangles);
        let mesh = Mesh { triangles, bvh };

        Ok(mesh)
    }
}
