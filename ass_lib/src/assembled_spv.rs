#[derive(Serialize, Deserialize, Debug)]
pub struct AssembledSpirv {
    pub vertex_shader: SpirvModule,
    pub fragment_shader: SpirvModule,
    pub textures: HashMap<String, Texture>,
    pub push_constants: Vec<PushConstant>,
}
