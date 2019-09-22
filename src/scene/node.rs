use std::sync::{Arc, Mutex};
use crate::{
    physics::Body,
    scene::{
        camera::Camera,
        mesh::Mesh,
        light::Light,
        particle_system::ParticleSystem,
        transform::Transform,
    },
    resource::model::Model,
};
use rg3d_core::{
    math::{
        vec3::Vec3,
        mat4::Mat4,
    },
    visitor::{
        Visit,
        VisitResult,
        Visitor,
    },
    pool::Handle,
};

pub enum NodeKind {
    Base,
    Light(Light),
    Camera(Camera),
    Mesh(Mesh),
    ParticleSystem(ParticleSystem),
}

impl Visit for NodeKind {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        match self {
            NodeKind::Base => Ok(()),
            NodeKind::Light(light) => light.visit(name, visitor),
            NodeKind::Camera(camera) => camera.visit(name, visitor),
            NodeKind::Mesh(mesh) => mesh.visit(name, visitor),
            NodeKind::ParticleSystem(particle_system) => particle_system.visit(name, visitor)
        }
    }
}

impl Clone for NodeKind {
    fn clone(&self) -> Self {
        match &self {
            NodeKind::Base => NodeKind::Base,
            NodeKind::Camera(camera) => NodeKind::Camera(camera.clone()),
            NodeKind::Light(light) => NodeKind::Light(light.clone()),
            NodeKind::Mesh(mesh) => NodeKind::Mesh(mesh.clone()),
            NodeKind::ParticleSystem(particle_system) => NodeKind::ParticleSystem(particle_system.clone())
        }
    }
}

impl NodeKind {
    /// Creates new NodeKind based on variant id.
    pub fn new(id: u8) -> Result<Self, String> {
        match id {
            0 => Ok(NodeKind::Base),
            1 => Ok(NodeKind::Light(Default::default())),
            2 => Ok(NodeKind::Camera(Default::default())),
            3 => Ok(NodeKind::Mesh(Default::default())),
            4 => Ok(NodeKind::ParticleSystem(Default::default())),
            _ => Err(format!("Invalid node kind {}", id))
        }
    }

    /// Returns actual variant id.
    pub fn id(&self) -> u8 {
        match self {
            NodeKind::Base => 0,
            NodeKind::Light(_) => 1,
            NodeKind::Camera(_) => 2,
            NodeKind::Mesh(_) => 3,
            NodeKind::ParticleSystem(_) => 4,
        }
    }
}

pub struct Node {
    name: String,
    kind: NodeKind,
    pub(in crate::scene) local_transform: Transform,
    pub(in crate::scene) visibility: bool,
    pub(in crate::scene) global_visibility: bool,
    pub(in crate::scene) parent: Handle<Node>,
    pub(in crate::scene) children: Vec<Handle<Node>>,
    pub(in crate::scene) global_transform: Mat4,
    inv_bind_pose_transform: Mat4,
    body: Handle<Body>,
    /// A resource from which this node was instantiated from, can work in pair
    /// with `original` handle to get corresponding node from resource.
    resource: Option<Arc<Mutex<Model>>>,
    /// Handle to node in scene of model resource from which this node
    /// was instantiated from.
    original: Handle<Node>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            kind: NodeKind::Base,
            name: String::new(),
            children: Vec::new(),
            parent: Handle::none(),
            visibility: true,
            global_visibility: true,
            local_transform: Transform::identity(),
            global_transform: Mat4::identity(),
            inv_bind_pose_transform: Mat4::identity(),
            body: Handle::none(),
            resource: None,
            original: Handle::none(),
        }
    }
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            name: String::from("Node"),
            children: Vec::new(),
            parent: Handle::none(),
            visibility: true,
            global_visibility: true,
            local_transform: Transform::identity(),
            global_transform: Mat4::identity(),
            inv_bind_pose_transform: Mat4::identity(),
            body: Handle::none(),
            resource: None,
            original: Handle::none(),
        }
    }

    /// Creates copy of node without copying children nodes and physics body.
    /// Children nodes has to be copied explicitly.
    pub fn make_copy(&self, original: Handle<Node>) -> Self {
        Self {
            kind: self.kind.clone(),
            name: self.name.clone(),
            local_transform: self.local_transform.clone(),
            global_transform: self.global_transform,
            visibility: self.visibility,
            global_visibility: self.global_visibility,
            inv_bind_pose_transform: self.inv_bind_pose_transform,
            children: Vec::new(),
            parent: Handle::none(),
            body: Handle::none(),
            resource: self.get_resource(),
            original,
        }
    }

    #[inline]
    pub fn get_original_handle(&self) -> Handle<Node> {
        self.original
    }

    #[inline]
    pub fn set_original_handle(&mut self, original: Handle<Node>) {
        self.original = original;
    }

    #[inline]
    pub fn set_body(&mut self, body: Handle<Body>) {
        self.body = body;
    }

    #[inline]
    pub fn get_body(&self) -> Handle<Body> {
        self.body
    }

    #[inline]
    pub fn get_kind(&self) -> &NodeKind {
        &self.kind
    }

    #[inline]
    pub fn set_resource(&mut self, resource_handle: Arc<Mutex<Model>>) {
        self.resource = Some(resource_handle);
    }

    #[inline]
    pub fn get_resource(&self) -> Option<Arc<Mutex<Model>>> {
        self.resource.as_ref().and_then(|r| Some(r.clone()))
    }

    #[inline]
    pub fn get_local_transform(&self) -> &Transform {
        &self.local_transform
    }

    #[inline]
    pub fn get_local_transform_mut(&mut self) -> &mut Transform {
        &mut self.local_transform
    }

    #[inline]
    pub fn get_kind_mut(&mut self) -> &mut NodeKind {
        &mut self.kind
    }

    #[inline]
    pub fn set_visibility(&mut self, visibility: bool) {
        self.visibility = visibility;
    }

    #[inline]
    pub fn get_visibility(&self) -> bool {
        self.visibility
    }

    #[inline]
    pub fn get_global_visibility(&self) -> bool {
        self.global_visibility
    }

    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn get_parent(&self) -> Handle<Node> {
        self.parent
    }

    #[inline]
    pub fn get_children(&self) -> &[Handle<Node>] {
        &self.children
    }

    #[inline]
    pub fn get_global_transform(&self) -> &Mat4 {
        &self.global_transform
    }

    pub fn set_inv_bind_pose_transform(&mut self, transform: Mat4) {
        self.inv_bind_pose_transform = transform;
    }

    pub fn get_inv_bind_pose_transform(&self) -> &Mat4 {
        &self.inv_bind_pose_transform
    }

    #[inline]
    pub fn get_global_position(&self) -> Vec3 {
        self.global_transform.position()
    }

    #[inline]
    pub fn get_look_vector(&self) -> Vec3 {
        self.global_transform.look()
    }

    #[inline]
    pub fn get_side_vector(&self) -> Vec3 {
        self.global_transform.side()
    }

    #[inline]
    pub fn get_up_vector(&self) -> Vec3 {
        self.global_transform.up()
    }
}

impl Visit for Node {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        let mut kind_id: u8 = self.kind.id();
        kind_id.visit("KindId", visitor)?;
        if visitor.is_reading() {
            self.kind = NodeKind::new(kind_id)?;
        }

        self.kind.visit("Kind", visitor)?;
        self.name.visit("Name", visitor)?;
        self.local_transform.visit("Transform", visitor)?;
        self.visibility.visit("Visibility", visitor)?;
        self.parent.visit("Parent", visitor)?;
        self.children.visit("Children", visitor)?;
        self.body.visit("Body", visitor)?;
        self.resource.visit("Resource", visitor)?;

        // TODO: Is this needed?
        self.inv_bind_pose_transform.visit("InvBindPoseTransform", visitor)?;

        visitor.leave_region()
    }
}