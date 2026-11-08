//! Geometry only; no OBJ/MTL loading or appearance data.
#[derive(Debug)]
pub struct Face {
    pub vertex_indices: [usize; 3],
    /// Indices `vt` por esquina, si la cara los declara.
    pub uv_indices: Option<[usize; 3]>,
    /// Indices `vn` por esquina, si la cara los declara.
    pub normal_indices: Option<[usize; 3]>,
}

#[derive(Debug)]
pub struct Model {
    pub vertices: Vec<[f64; 3]>,
    pub uvs: Vec<[f64; 2]>,
    pub normals: Vec<[f64; 3]>,
    pub faces: Vec<Face>,
    /// Esfera envolvente en espacio de objeto (centro de la caja y radio maximo).
    pub bounds_center: [f64; 3],
    pub bounds_radius: f64,
}
