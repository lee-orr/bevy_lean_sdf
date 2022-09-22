//! The root SDF object
use crate::{
    sdf_operations::SDFOperators,
    sdf_primitives::SDFPrimitive,
    sdf_shader::{SDFInstanceData, SDFRenderAsset},
};
use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::{PrepareAssetError, RenderAsset},
        renderer::RenderDevice,
    },
};

/// A single SDF Element
#[derive(Debug, Clone)]
pub struct SDFElement {
    /// The SDF Primitive
    pub primitive: SDFPrimitive,
    /// The transform matrix
    transform: Mat4,
    /// The transform matrix
    inverse: Mat4,
    // the scale
    scale: f32,
    /// Operation for joining the object with the previous object
    pub operation: SDFOperators,
}

impl Default for SDFElement {
    fn default() -> Self {
        let transform = Mat4::from_scale(Vec3::ONE);
        Self {
            primitive: SDFPrimitive::Sphere(1.),
            inverse: transform.inverse(),
            transform,
            scale: 1.,
            operation: SDFOperators::Union,
        }
    }
}

impl SDFElement {
    /// Create a new element - short for the default
    pub fn new() -> Self {
        Self::default()
    }

    /// Make `SDFElement` with a primitive
    pub fn with_primitive(mut self, primitive: SDFPrimitive) -> Self {
        self.primitive = primitive;
        self
    }

    /// Make `SDFElement` with an operation
    pub fn with_operation(mut self, operation: SDFOperators) -> Self {
        self.operation = operation;
        self
    }

    /// Make `SDFElement` with a translation
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        let (scale, rotation, _) = self.transform.to_scale_rotation_translation();
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inverse = self.transform.inverse();
        self
    }

    /// Make `SDFElement` with a rotation
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        let (scale, _, translation) = self.transform.to_scale_rotation_translation();
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inverse = self.transform.inverse();
        self
    }

    /// Make `SDFElement` with a scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        let (_, rotation, translation) = self.transform.to_scale_rotation_translation();
        let scale = scale.abs();
        self.scale = scale;
        let scale = scale * Vec3::ONE;
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inverse = self.transform.inverse();
        self
    }

    /// Get the value of the SDF at a given point
    pub fn value_at_point(&self, point: &Vec3) -> f32 {
        let scale = self.scale;
        let transform = self.inverse;
        self.primitive
            .value_at_point(&(transform.transform_point3(*point)))
            * scale
    }

    /// Get the value, taking into account previous values
    pub fn process_object_at_point(&self, point: &Vec3, previous: f32) -> f32 {
        let value = self.value_at_point(point);
        self.operation.value(&previous, &value)
    }

    /// Get the bounds of the element, potentially given a previous element
    pub fn get_bounds(&self, previous: &Option<(Vec3, Vec3)>) -> (Vec3, Vec3) {
        let bounds = self.primitive.get_bounds();
        let bounds = (
            self.transform.transform_point3(bounds.0),
            self.transform.transform_point3(bounds.1),
        );
        let mut bounds = (bounds.0.min(bounds.1), bounds.0.max(bounds.1));

        if let Some(previous) = previous {
            bounds = self.operation.get_bounds(previous, &bounds);
        }
        bounds
    }
}

/// The root SDF object
#[derive(Debug, Clone, TypeUuid, Default)]
#[uuid = "3e9f6f3f-730c-46d1-8e12-4715f4c6f861"]
pub struct SDFObject {
    /// A list of all the elements in the SDF
    pub elements: Vec<SDFElement>,
    /// The mesh handle for the current SDF object
    pub mesh_handle: Option<Handle<Mesh>>,
}

impl SDFObject {
    /// Add Element
    pub fn with_element(mut self, element: SDFElement) -> Self {
        self.elements.push(element);
        self
    }

    /// Calculate the value of the SDF Object at a given point
    pub fn value_at_point(&self, point: &Vec3) -> f32 {
        self.elements.iter().fold(f32::INFINITY, |value, element| {
            element.process_object_at_point(point, value)
        })
    }

    /// Calculate SDF Object bounds
    pub fn get_bounds(&self) -> (Vec3, Vec3) {
        self.elements
            .iter()
            .fold(None, |value, element| Some(element.get_bounds(&value)))
            .unwrap_or((Vec3::ZERO, Vec3::ZERO))
    }

    /// Get the locations of boxes designed to cover the surface at a given size
    pub fn generate_boxes(&self, resolution: usize, bounds: &(Vec3, Vec3)) -> (f32, Vec<Vec3>) {
        let size = (bounds.1 - bounds.0).max_element();
        let box_size = size / (resolution as f32);
        let half_box_size = box_size / 2.;
        let mut boxes: Vec<Vec3> = Vec::new();
        for x in (0..resolution).map(|x| {
            let x = x as f32;
            bounds.0.x + x * box_size + half_box_size
        }) {
            for y in (0..resolution).map(|y| {
                let y = y as f32;
                bounds.0.y + y * box_size + half_box_size
            }) {
                for z in (0..resolution).map(|z| {
                    let z = z as f32;
                    bounds.0.z + z * box_size + half_box_size
                }) {
                    let point = Vec3::new(x, y, z);
                    let sdf = self.value_at_point(&point);
                    if sdf <= box_size && sdf >= -box_size {
                        boxes.push(point);
                    }
                }
            }
        }
        (box_size, boxes)
    }

    /// Generate the contents of a texture
    pub fn generate_texture(&self, resolution: usize, bounds: &(Vec3, Vec3)) -> Vec<u8> {
        let size = (bounds.1 - bounds.0).max_element();
        let box_size = size / (resolution as f32);
        let half_box_size = box_size / 2.;
        let mut boxes: Vec<u8> = Vec::new();
        for x in (0..resolution).map(|x| {
            let x = x as f32;
            bounds.0.x + x * box_size + half_box_size
        }) {
            for y in (0..resolution).map(|y| {
                let y = y as f32;
                bounds.0.y + y * box_size + half_box_size
            }) {
                for z in (0..resolution).map(|z| {
                    let z = z as f32;
                    bounds.0.z + z * box_size + half_box_size
                }) {
                    let point = Vec3::new(x, y, z);
                    let sdf = self.value_at_point(&point);
                    if sdf <= box_size && sdf >= -box_size {
                        boxes.push(1);
                    } else {
                        boxes.push(0);
                    }
                }
            }
        }
        boxes
    }

    /// Get locations of boxes at all LODs
    pub fn generate_lod_boxes(
        &self,
        resolution: usize,
        max_lods: usize,
        min_box_size: f32,
    ) -> Vec<(f32, Vec<Vec<Vec3>>)> {
        let bounds = self.get_bounds();
        let mut lods: Vec<(f32, Vec<Vec<Vec3>>)> = Vec::new();

        loop {
            bevy::log::info!(
                "Getting box data for LOD {}, max is {}",
                lods.len(),
                &max_lods
            );
            if lods.len() >= max_lods {
                break;
            }
            if let Some((last_lod_size, last_lod_vecs)) = lods.last() {
                if *last_lod_size < min_box_size {
                    break;
                }
                let lod_half_size = last_lod_size / 2.;
                let mut lod = Vec::<Vec<Vec3>>::new();
                let mut new_size = lod_half_size / (resolution as f32);
                for current in last_lod_vecs.iter().flatten() {
                    let result = self.generate_boxes(
                        resolution,
                        &(*current - lod_half_size, *current + lod_half_size),
                    );
                    new_size = result.0;
                    lod.push(result.1);
                }
                lods.push((new_size, lod));
            } else {
                let result = self.generate_boxes(resolution, &bounds);
                lods.push((result.0, vec![result.1]));
            }
        }

        lods
    }

    /// Generate box mesh
    pub fn generate_box_mesh(
        &self,
        resolution: usize,
        target_lod: usize,
        min_box_size: f32,
    ) -> Mesh {
        let lod_boxes = self.generate_lod_boxes(resolution, target_lod, min_box_size);

        if let Some((size, boxes)) = lod_boxes.last() {
            let (mut positions, mut normals, mut uvs, mut indices) = (
                Vec::<[f32; 3]>::new(),
                Vec::<[f32; 3]>::new(),
                Vec::<[f32; 2]>::new(),
                Vec::<u32>::new(),
            );

            let mut starting_index = 0u32;
            for b in boxes.iter().flatten() {
                let (next_index, mut position, mut normal, mut uv, mut local_indices) =
                    build_box(b, *size, starting_index);

                positions.append(&mut position);
                normals.append(&mut normal);
                uvs.append(&mut uv);
                indices.append(&mut local_indices);

                starting_index = next_index;
            }
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.set_indices(Some(Indices::U32(indices)));
            mesh
        } else {
            Mesh::from(shape::Cube::default())
        }
    }
}

fn build_box(
    position: &Vec3,
    size: f32,
    start_index: u32,
) -> (u32, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    bevy::log::info!("Building box @ {} {}", position, &size);
    let half_size = size / 2.;

    let min = *position - half_size;
    let max = *position + half_size;

    let vertices = &[
        // Top
        ([min.x, min.y, max.z], [0., 0., 1.0], [0., 0.]),
        ([max.x, min.y, max.z], [0., 0., 1.0], [1.0, 0.]),
        ([max.x, max.y, max.z], [0., 0., 1.0], [1.0, 1.0]),
        ([min.x, max.y, max.z], [0., 0., 1.0], [0., 1.0]),
        // Bottom
        ([min.x, max.y, min.z], [0., 0., -1.0], [1.0, 0.]),
        ([max.x, max.y, min.z], [0., 0., -1.0], [0., 0.]),
        ([max.x, min.y, min.z], [0., 0., -1.0], [0., 1.0]),
        ([min.x, min.y, min.z], [0., 0., -1.0], [1.0, 1.0]),
        // Right
        ([max.x, min.y, min.z], [1.0, 0., 0.], [0., 0.]),
        ([max.x, max.y, min.z], [1.0, 0., 0.], [1.0, 0.]),
        ([max.x, max.y, max.z], [1.0, 0., 0.], [1.0, 1.0]),
        ([max.x, min.y, max.z], [1.0, 0., 0.], [0., 1.0]),
        // Left
        ([min.x, min.y, max.z], [-1.0, 0., 0.], [1.0, 0.]),
        ([min.x, max.y, max.z], [-1.0, 0., 0.], [0., 0.]),
        ([min.x, max.y, min.z], [-1.0, 0., 0.], [0., 1.0]),
        ([min.x, min.y, min.z], [-1.0, 0., 0.], [1.0, 1.0]),
        // Front
        ([max.x, max.y, min.z], [0., 1.0, 0.], [1.0, 0.]),
        ([min.x, max.y, min.z], [0., 1.0, 0.], [0., 0.]),
        ([min.x, max.y, max.z], [0., 1.0, 0.], [0., 1.0]),
        ([max.x, max.y, max.z], [0., 1.0, 0.], [1.0, 1.0]),
        // Back
        ([max.x, min.y, max.z], [0., -1.0, 0.], [0., 0.]),
        ([min.x, min.y, max.z], [0., -1.0, 0.], [1.0, 0.]),
        ([min.x, min.y, min.z], [0., -1.0, 0.], [1.0, 1.0]),
        ([max.x, min.y, min.z], [0., -1.0, 0.], [0., 1.0]),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = [
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ]
    .iter()
    .map(|v| v + start_index)
    .collect();
    let next_index = positions.len() as u32 + start_index;
    (next_index, positions, normals, uvs, indices)
}

impl RenderAsset for SDFObject {
    type ExtractedAsset = SDFObject;

    type PreparedAsset = SDFRenderAsset;

    type Param = SRes<RenderDevice>;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        sdf: Self::ExtractedAsset,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        bevy::log::info!("Preparing SDF Asset");
        let boxes = sdf.generate_lod_boxes(8, 4, 0.5);
        if let Some((size, boxes)) = boxes.last() {
            let half_size = Vec3::ONE * *size / 2.;
            Ok(SDFRenderAsset {
                instance_data: boxes
                    .iter()
                    .flatten()
                    .map(|b| {
                        let _texture = sdf.generate_texture(8, &(*b - half_size, *b + half_size));
                        SDFInstanceData { position: *b }
                    })
                    .collect(),
            })
        } else {
            Err(PrepareAssetError::RetryNextUpdate(sdf))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use assert_float_eq::*;
    use bevy::prelude::{EulerRot, Vec3};

    use super::*;
    use crate::sdf_primitives::SDFPrimitive;

    #[test]
    fn translates_a_sdf() {
        let sdf = SDFElement::default().with_translation(Vec3::X);

        let interior = sdf.value_at_point(&Vec3::X);
        let surface = sdf.value_at_point(&Vec3::ZERO);
        let outside = sdf.value_at_point(&Vec3::new(-0.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_a_sdf() {
        let sdf = SDFElement::default()
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.))
            .with_primitive(SDFPrimitive::Box(Vec3::new(1., 2., 1.)));

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_and_transforms_sdf() {
        let sdf = SDFElement::default()
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.))
            .with_primitive(SDFPrimitive::Box(Vec3::new(1., 2., 1.)))
            .with_translation(Vec3::X);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_a_sdf() {
        let sdf = SDFElement::default().with_scale(2.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::X * 2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -2.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_transforms_and_rotates_a_sdf() {
        let sdf = SDFElement::default()
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.))
            .with_primitive(SDFPrimitive::Box(Vec3::new(0.5, 1., 0.5)))
            .with_translation(Vec3::X)
            .with_scale(2.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn union_of_sdfs() {
        let sdf_a = SDFElement::default().with_translation(Vec3::X);
        let sdf_b = SDFElement::default().with_translation(-1. * Vec3::X);
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
            mesh_handle: None,
        };

        let interior_a = sdf.value_at_point(&Vec3::X);
        let interior_b = sdf.value_at_point(&(Vec3::X * -1.));
        let surface_a = sdf.value_at_point(&(Vec3::X * 2.));
        let surface_b = sdf.value_at_point(&(Vec3::X * -2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(interior_a, -1.);
        assert_float_absolute_eq!(interior_b, -1.);
        assert_float_absolute_eq!(surface_a, 0.);
        assert_float_absolute_eq!(surface_b, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn subtraction_of_sdfs() {
        let sdf_a = SDFElement {
            primitive: SDFPrimitive::Sphere(2.),
            ..default()
        };
        let sdf_b = SDFElement {
            primitive: SDFPrimitive::Sphere(1.),
            operation: SDFOperators::Subtraction,
            ..default()
        };
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
            mesh_handle: None,
        };

        let center = sdf.value_at_point(&Vec3::ZERO);
        let inner_surface = sdf.value_at_point(&Vec3::X);
        let mid_way = sdf.value_at_point(&(Vec3::X * 1.5));
        let outer_surface = sdf.value_at_point(&(Vec3::X * -2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(center, 1.);
        assert_float_absolute_eq!(mid_way, -0.5);
        assert_float_absolute_eq!(inner_surface, 0.);
        assert_float_absolute_eq!(outer_surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn intersection_of_sdfs() {
        let sdf_a = SDFElement {
            primitive: SDFPrimitive::Sphere(2.),
            ..default()
        };
        let sdf_b = SDFElement {
            primitive: SDFPrimitive::Sphere(1.),
            operation: SDFOperators::Intersection,
            ..default()
        };
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
            mesh_handle: None,
        };

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&Vec3::Y);
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn translates_bounds() {
        let sdf = SDFElement::default().with_translation(Vec3::X);

        let bounds = sdf.get_bounds(&None);

        assert_float_absolute_eq!(bounds.0.x, 0.);
        assert_float_absolute_eq!(bounds.0.y, -1.);
        assert_float_absolute_eq!(bounds.0.z, -1.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 1.);
        assert_float_absolute_eq!(bounds.1.z, 1.);
    }

    #[test]
    fn rotates_bounds() {
        let sdf = SDFElement::default()
            .with_primitive(SDFPrimitive::Box(Vec3::new(1., 2., 0.5)))
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 90. * PI / 180., 0.));

        let bounds = sdf.get_bounds(&None);

        assert_float_absolute_eq!(bounds.0.x, -0.5);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -1.);
        assert_float_absolute_eq!(bounds.1.x, 0.5);
        assert_float_absolute_eq!(bounds.1.y, 2.);
        assert_float_absolute_eq!(bounds.1.z, 1.);
    }

    #[test]
    fn scales_bounds() {
        let sdf = SDFElement::default().with_scale(2.);

        let bounds = sdf.get_bounds(&None);

        assert_float_absolute_eq!(bounds.0.x, -2.);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -2.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 2.);
        assert_float_absolute_eq!(bounds.1.z, 2.);
    }

    #[test]
    fn bounds_of_multiple_elements() {
        let sdf_a = SDFElement::default().with_translation(Vec3::X);
        let sdf_b = SDFElement::default().with_translation(-1. * Vec3::X);
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
            mesh_handle: None,
        };

        let bounds = sdf.get_bounds();

        assert_float_absolute_eq!(bounds.0.x, -2.);
        assert_float_absolute_eq!(bounds.0.y, -1.);
        assert_float_absolute_eq!(bounds.0.z, -1.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 1.);
        assert_float_absolute_eq!(bounds.1.z, 1.);
    }

    #[test]
    fn generate_boxes_on_surface() {
        let sdf_a = SDFElement::default().with_primitive(SDFPrimitive::Box(Vec3::ONE));
        let sdf = SDFObject {
            elements: vec![sdf_a],
            mesh_handle: None,
        };

        let result = sdf.generate_boxes(3, &sdf.get_bounds());
        assert_float_absolute_eq!(result.0, 2. / 3.);
        assert_eq!(result.1.len(), 9 * 2 + 8);
    }

    #[test]
    fn generate_boxes_on_surface_with_lod() {
        let sdf_a = SDFElement::default().with_primitive(SDFPrimitive::Box(Vec3::ONE));
        let sdf = SDFObject {
            elements: vec![sdf_a],
            mesh_handle: None,
        };

        let result = sdf.generate_lod_boxes(3, 2, 0.1);
        assert_eq!(result.len(), 2);
        assert_float_absolute_eq!(result[0].0, 2. / 3.);
        assert_eq!(result[0].1[0].len(), 9 * 2 + 8);
        assert_float_absolute_eq!(result[1].0, 2. / 9.);
        assert_eq!(result[1].1.len(), 9 * 2 + 8);
        assert_eq!(result[1].1[0].len(), 19);
    }
}
