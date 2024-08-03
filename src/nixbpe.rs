pub struct Nixbpe {
    pub n: bool,
    pub i: bool,
    pub x: bool,
    pub b: bool,
    pub p: bool,
    pub e: bool,
}

impl Nixbpe {
    pub fn new() -> Self {
        Self {
            n: false,
            i: false,
            x: false,
            b: false,
            p: false,
            e: false,
        }
    }
    pub fn set_direct(&mut self) {
        self.n = true;
        self.i = true;
    }
    pub fn set_indirect(&mut self) {
        self.n = true;
        self.i = false;
    }
    pub fn set_immediate(&mut self) {
        self.n = false;
        self.i = true;
    }
    pub fn set_indexed(&mut self) {
        self.x = true;
    }
    pub fn set_base_relative(&mut self) {
        self.b = true;
    }
    pub fn set_pc_relative(&mut self) {
        self.p = true;
    }
    pub fn set_extended(&mut self) {
        self.e = true;
    }
    pub fn as_bin_string(&self) -> String {
        format!("{}{}{}{}{}{}", self.n as i32, self.i as i32, self.x as i32, self.b as i32, self.p as i32, self.e as i32)
    }
}
