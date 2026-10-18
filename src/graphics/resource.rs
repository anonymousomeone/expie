use std::collections::HashMap;

use image::GenericImageView;

use crate::graphics::util;

pub struct ResourceManager {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub textures: Vec<wgpu::Texture>,
    pub samplers: Vec<wgpu::Sampler>,
    // pub texture_views: Vec<wgpu::TextureView>,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub group_layouts: Vec<wgpu::BindGroupLayout>,
    pub texture_map: HashMap<String, TextureId>,
}

impl ResourceManager {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> ResourceManager {
        ResourceManager {
            device,
            queue,
            textures: Vec::new(),
            samplers: Vec::new(),
            bind_groups: Vec::new(),
            group_layouts: Vec::new(),
            texture_map: HashMap::new(),
        }
    }

    pub fn load_texture_from_path(&mut self, path: &str) -> TextureId {
        // is this a good idea? probably not
        match self.texture_map.get(&path.to_string()) {
            Some(id) => return *id,
            None => {}
        };

        let bytes = std::fs::read(path).unwrap();
        let image = image::load_from_memory(&bytes).unwrap();
        let data = image.to_rgba8();

        let (width, height) = image.dimensions();

        let id = self.load_texture(width, height, &data);
        self.texture_map.insert(path.to_string(), id);
        return id;
    }

    pub fn load_texture(&mut self, width: u32, height: u32, data: &[u8]) -> TextureId {
        let texture = util::load_texture(
            &self.device,
            &self.queue,
            width,
            height,
            data
        );

        self.textures.push(texture);
        return TextureId((self.textures.len() - 1) as u32);
    }

    pub fn new_sampler(&mut self) -> SamplerId {
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        self.samplers.push(sampler);
        return SamplerId((self.samplers.len() - 1) as u32);
    }

    pub fn new_bind_group(&mut self,
        layout: BindGroupLayoutId,
        entries: Vec<wgpu::BindGroupEntry>,
        label: Option<&str>,
    ) -> BindGroupId {
        let bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: self.group_layouts.get(layout.0 as usize).unwrap(),
                entries: &entries,
                label,
            }
        );

        self.bind_groups.push(bind_group);
        return BindGroupId((self.bind_groups.len() - 1) as u32);
    }

    pub fn new_layout(&mut self,
        entries: Vec<wgpu::BindGroupLayoutEntry>,
        label: Option<&str>,
    ) -> BindGroupLayoutId {
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label,
        });

        self.group_layouts.push(bind_group_layout);
        return BindGroupLayoutId((self.group_layouts.len() - 1) as u32);
    }

    pub fn get_bind_group(&self, id: BindGroupId) ->& wgpu::BindGroup {
        return &self.bind_groups[id.0 as usize];
    }

    pub fn get_texture(&self, id: TextureId) -> &wgpu::Texture {
        return &self.textures[id.0 as usize];
    }

    pub fn get_sampler(&self, id: SamplerId) -> wgpu::Sampler {
        return self.samplers[id.0 as usize].clone();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerId(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindGroupId(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindGroupLayoutId(u32);