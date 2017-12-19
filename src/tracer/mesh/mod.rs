use nalgebra::{Point3, Vector3};
use std::path::Path;
use tracer::primitive::triangle::Triangle;
use tracer::primitive::Material;
use tracer::primitive::Primitive;
use tracer::primitive::Intersection;
use std::cmp::Ordering;
use bvh::ray::Ray;
use bvh::bvh::BVH;
use std::fs::File;
use std::io::BufReader;
use obj;
use obj::Obj;
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
    ///
    pub fn load_from_path2(path: &Path) -> Mesh {
        let file_input =
            BufReader::new(File::open(path).expect("Failed to open .obj file."));
        let obj: Obj<Triangle> = obj::load_obj(file_input).expect("Failed to decode .obj file data.");
        let mut triangles: Vec<Triangle> = obj.vertices;
        let bvh = BVH::build(&mut triangles);
        Mesh { triangles, bvh }
    }
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

        println!("indices: {:?}", indices.len());
        println!("positions: {:?}", positions.len());
        println!("normals: {:?}", normals.len());

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
                let p0 = (positions[indices[0] as usize] * scale) + translation;
                let p1 = (positions[indices[1] as usize] * scale) + translation;
                let p2 = (positions[indices[2] as usize] * scale) + translation;
                let  n0;
                let  n1;
                let  n2;

                if normals.len() == 0 {
                    n0 = (p1 - p0).cross(&(p1 - p2)).normalize();
                    n1 = (p1 - p0).cross(&(p1 - p2)).normalize();
                    n2 = (p1 - p0).cross(&(p1 - p2)).normalize();
                } else {
                    n0 = normals[indices[0] as usize];
                    n1 = normals[indices[1] as usize];
                    n2 = normals[indices[2] as usize];
                }

                Triangle {
                    material: material.clone(),
                    //material: Material::Conductor{spec:1.0, color:Vector3::new(1.0,0.0,0.0)} ,
                    p0,
                    p1,
                    p2,
                    n0,
                    n1,
                    n2,
                    node_index: 0,
                }
            })
            .collect();

        let bvh = BVH::build(&mut triangles);
        let mesh = Mesh { triangles, bvh };

        Ok(mesh)
    }
}
