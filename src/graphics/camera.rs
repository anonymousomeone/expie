use glam::{Mat4, Vec2};
use wgpu::util::DeviceExt;

pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Camera {
    pub fn new(device: &wgpu::Device) -> Camera {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        Camera {
            position: Vec2::new(0.5, 0.5),
            zoom: 0.25,

            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn to_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let translation = Mat4::IDENTITY * Mat4::from_translation(self.position.extend(1.0));

        let height = 1.0 / self.zoom;
        let width = height * aspect_ratio;
        let projection = glam::camera::rh::proj::directx::orthographic(
            0.0,
            width,
            0.0,
            height,
            -1.0,
            1.0
        );

        return projection * translation;
    }
}

pub struct CameraController {
    pub controller_type: ControllerType
}

impl CameraController {
    pub fn new(controller_type: ControllerType) -> CameraController {
        CameraController {
            controller_type,
        }
    }
}

pub enum ControllerType {
    Freecam,
}