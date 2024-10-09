use crate::vm::message::MessageType;

pub struct Program {
    code: Vec<u8>,
    internal: Option<usize>,
    external: Option<usize>,
    view: Option<usize>,
}

pub struct ProgramReaderFromBytes<'a> {
    offset: usize,
    bytes: &'a [u8],
}

impl<'a> ProgramReaderFromBytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            offset: 0,
            bytes: bytes,
        }
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        let value = self.bytes.get(self.offset)?.clone();
        self.offset += 1; 
        Some(value)
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        let value = self.bytes.get(self.offset..(self.offset + size_of::<u64>()))?;
        self.offset += value.len(); 
        Some(u64::from_be_bytes(value.try_into().ok()?))
    }

    pub fn load(&mut self) -> Option<Program> {
        let mut internal = None;
        if self.read_u8()? == 1 {
            internal = self.read_u64().map(|x| x as usize);
        }
        let mut external = None;
        if self.read_u8()? == 1 {
            external = self.read_u64().map(|x| x as usize);
        }
        let mut view = None;
        if self.read_u8()? == 1 {
            view = self.read_u64().map(|x| x as usize);
        }
        let code = Vec::from(self.bytes.get(self.offset..self.bytes.len())?);
        Some(
            Program { code: code, internal: internal, external: external, view: view }
        )
    }
}

impl Program {
    pub fn get_code(&self) -> Vec<u8> {
        self.code.clone()
    }

    pub fn get_internal(&self) -> Option<usize> {
        self.internal.clone()
    }

    pub fn get_external(&self) -> Option<usize> {
        self.external.clone()
    }

    pub fn get_view(&self) -> Option<usize> {
        self.view.clone()
    }

    pub fn get_entrypoint(&self, message_type: MessageType) -> Option<usize> {
        match message_type {
            MessageType::Internal => self.get_internal(),
            MessageType::External => self.get_external(),
            MessageType::View => self.get_view(),
        }
    }
}
