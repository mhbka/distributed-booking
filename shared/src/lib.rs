use uuid::Uuid;

pub mod requests;
pub mod responses;
pub mod time;

/// Trait for things that are serializable to/from bytes.
/// 
/// ## Implementation
/// - For structs, the conversion must be in top-to-bottom order of struct fields,
/// where each struct field is also `Byteable`.
/// 
/// - For variable-length fields, the first byte (or 2) should be a `u8`/`u16` for the data's bytelength, 
/// followed by the actual data.
/// 
/// - For enums, the the first byte should be a discriminant for the actual variant, 
/// followed by the actual data. 
/// 
/// - For static-sized fields, it should just be the bytes.
/// 
/// ## Derive
/// If a struct's fields are all `Byteable`, you can use `ByteableDerive` to quickly get an implementation.
pub trait Byteable where Self: Sized {
    /// Deserialize the type from bytes in a Vec of bytes.
    /// 
    /// Errors if unable to.
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String>;
    /// Deserializes the type to a Vec of bytes.
    /// 
    /// TODO: use `&mut Vec<u8>` here too for optimization?
    fn to_bytes(self) -> Vec<u8>;
}

impl Byteable for bool {
    /// From a single `u8` where `0` is `false` and everything else is `true`.
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        if data.len() >= 1 {
            return Ok(data.remove(0) >= 1);
        }
        Err("0 bytes found".to_string())
    }
    
    fn to_bytes(self) -> Vec<u8> {
        match self {
            true => vec![1],
            false => vec![0]
        }
    }
}

impl Byteable for u8 {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        if data.len() >= 1 {
            return Ok(data.remove(0));
        }
        Err("0 bytes found".to_string())
    }

    fn to_bytes(self) -> Vec<u8> {
        vec![self]
    }
}

impl Byteable for u16 {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        if data.len() >= 2 {
            let bytes = data
                    .drain(..2)
                    .collect::<Vec<_>>()
                    .try_into()
                    .map_err(|err| "Somehow got an error though enough bytes".to_string())?;
            return Ok(
                u16::from_ne_bytes(bytes)
            );
        }
        Err("<2 bytes found".to_string())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Byteable for Uuid {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        if data.len() >= 16 {
            let bytes = data
                .drain(..16)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|err| "Somehow got an error though enough bytes".to_string())?;
            return Ok(
                Uuid::from_bytes(bytes)
            );
        }
        Err(format!("Not enough bytes (len: {})", data.len()))
    }

    fn to_bytes(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Byteable for String {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        let length = u16::from_bytes(data)?;

        if data.len() >= length as usize {
            let bytes = data
                .drain(..length as usize)
                .collect::<Vec<_>>();

            return Ok(
                String::from_utf8(bytes)
                    .map_err(|err| format!("Unable to parse bytes to string: {err}"))?
            )
        }

        Err(format!("Not enough bytes (len: {})", data.len()))
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = (self.len() as u16).to_bytes();
        bytes.extend(self.bytes());
        bytes
    }
}

impl<T: Byteable> Byteable for Vec<T> {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        let length = u16::from_bytes(data)?;

        if data.len() >= length as usize {
            let mut bytes = data
                .drain(..length as usize)
                .collect::<Vec<_>>();

            let mut items = Vec::new();
            while bytes.len() > 0 {
                let item = T::from_bytes(&mut bytes)?;
                items.push(item);
            }

            return Ok(items)
        }
        Err(format!("Not enough bytes (len: {})", data.len()))
    }

    fn to_bytes(self) -> Vec<u8> {
        let data_bytes: Vec<u8> = self
            .into_iter()
            .flat_map(|t| t.to_bytes())
            .collect();
        let mut bytes = (data_bytes.len() as u16).to_bytes();
        bytes.extend(data_bytes);
        bytes
    }
}
