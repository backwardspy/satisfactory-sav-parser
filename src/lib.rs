//! https://satisfactory.fandom.com/wiki/Save_files
use std::io::{Read, Seek};

use adastring::ADAString;
use binrw::{BinRead, BinReaderExt, BinResult};
use thiserror::Error;

mod adastring;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read save file")]
    BinRead(#[from] binrw::Error),
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct SaveFileHeader {
    pub version: i32,
    pub save_version: i32,
    pub build_version: i32,
    pub map_name: ADAString,
    pub map_options: ADAString,
    pub session_name: ADAString,
    pub seconds_played: i32,
    pub save_timestamp: i64,
    pub session_visibility: i8,
    pub editor_object_version: i32,
    pub mod_metadata: ADAString,
    pub mod_flags: i32,
    pub save_identifier: ADAString,
    #[br(map(|x: i32| x != 0))]
    pub is_partitioned_world: bool,
    pub md5_hash: [u8; 20],
    #[br(map(|x: i32| x != 0))]
    pub is_creative_mode_enabled: bool,
}

#[derive(Debug, BinRead)]
#[br(little, magic = 0x9E2A83C1u32)]
pub struct CompressedSaveFileBody {
    pub archive_header: u32,
    #[br(assert(max_chunk_size == 128 * 1024))]
    pub max_chunk_size: i64,
    #[br(if(archive_header == 0x22222222))]
    pub compression_algorithm: i8,
    pub compressed_size: i64,
    pub uncompressed_size: i64,
    pub compressed_size_2: i64,
    pub uncompressed_size_2: i64,
    #[br(count = compressed_size)]
    pub chunk_bytes: Vec<u8>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct SaveFileBody {
    pub uncompressed_size: i32,
    pub sublevel_count: i32,
    #[br(count = sublevel_count)]
    pub sub_levels: Vec<Level>,
    pub persistent_level: Level,
    pub object_reference_count: i32,
    #[br(count = object_reference_count)]
    pub object_references: Vec<ObjectReference>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct Level {
    pub sublevel_name: ADAString,
    pub object_header_and_collectables_size: i32,
    pub object_header_count: i32,
    #[br(count = object_header_count)]
    pub object_headers: Vec<ObjectHeader>,
    pub collectables_count: i32,
    #[br(count = collectables_count)]
    pub collectables: Vec<ObjectReference>,
    pub objects_size: i32,
    pub object_count: i32,
    #[br(parse_with = parse_objects, args(&object_headers))]
    pub objects: Vec<Object>,
    pub collectables_count_2: i32,
    #[br(count = collectables_count_2)]
    pub collections_2: Vec<ObjectReference>,
}

#[binrw::parser(reader, endian)]
fn parse_objects(headers: &[ObjectHeader]) -> BinResult<Vec<Object>> {
    let mut objects = Vec::new();

    for header in headers {
        match header {
            ObjectHeader::Actor(_) => {
                let object = ActorObject::read_options(reader, endian, ())?;
                objects.push(Object::Actor(object));
            }
            ObjectHeader::Component(_) => {
                let object = ComponentObject::read_options(reader, endian, ())?;
                objects.push(Object::Component(object));
            }
        }
    }

    Ok(objects)
}

#[derive(Debug, BinRead)]
#[br(little)]
pub enum ObjectHeader {
    #[br(magic = 1i32)]
    Actor(ActorHeader),
    #[br(magic = 0i32)]
    Component(ComponentHeader),
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Actor,
    Component,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ActorHeader {
    pub type_path: ADAString,
    pub root_object: ADAString,
    pub instance_name: ADAString,
    pub need_transform: i32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotation_w: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub was_placed_in_level: i32,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ComponentHeader {
    pub type_path: ADAString,
    pub root_object: ADAString,
    pub instance_name: ADAString,
    pub parent_actor_name: ADAString,
}

#[derive(Debug, BinRead)]
#[br(little, import { object_type: ObjectType })]
pub enum Object {
    #[br(pre_assert(matches!(object_type, ObjectType::Actor)))]
    Actor(ActorObject),
    #[br(pre_assert(matches!(object_type, ObjectType::Component)))]
    Component(ComponentObject),
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ActorObject {
    pub size: i32,
    pub parent_object_root: ADAString,
    pub parent_object_name: ADAString,
    pub component_count: i32,
    #[br(count = component_count)]
    pub components: Vec<ObjectReference>,
    pub properties: PropertyList,
    pub trailing: [u8; 16],
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ComponentObject {
    pub size: i32,
    pub properties: PropertyList,
    pub trailing: [u8; 16],
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ObjectReference {
    pub level_name: ADAString,
    pub path_name: ADAString,
}

#[derive(Debug, BinRead)]
#[br(little, import { prop_type: ADAString })]
pub enum Property {
    #[br(pre_assert(prop_type == "ArrayProperty"))]
    Array(ArrayProperty),
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ArrayProperty {
    pub size: i32,
    pub index: i32,
    pub element_type: ADAString,
    #[br(pad_before = 1)]
    pub length: i32,
    #[br(count = length, args { inner: PropertyBinReadArgs { prop_type: element_type.clone() } })]
    pub elements: Vec<Property>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct BoolProperty {
    #[br(pad_before = 4)]
    pub index: i32,
    #[br(map(|x: i32| x != 0), pad_after = 1)]
    pub value: bool,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ByteProperty {
    pub size: i32,
    pub index: i32,
    pub prop_type: ADAString,
    #[br(args { prop_type: prop_type.clone() }, pad_before = 1)]
    pub value: BytePropertyValue,
}

#[derive(Debug, BinRead)]
#[br(little, import { prop_type: ADAString })]
pub enum BytePropertyValue {
    #[br(pre_assert(prop_type == "None"))]
    Byte(i8),
    #[br(pre_assert(prop_type != "None"))]
    String(ADAString),
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct EnumProperty {
    pub size: i32,
    pub index: i32,
    pub prop_type: ADAString,
    #[br(pad_before = 1)]
    pub value: ADAString,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct FloatProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub value: f32,
}

#[derive(Debug, BinRead, Default)]
#[br(little)]
pub struct IntProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub value: i32,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct Int64Property {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub value: i64,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct MapProperty {
    pub size: i32,
    pub index: i32,
    pub key_type: ADAString,
    pub value_type: ADAString,
    #[br(pad_before = 1)]
    pub mode_type: i32,
    pub element_count: i32,
    #[br(count = element_count, args { inner: KVPairBinReadArgs { key_type: key_type.clone(), value_type: value_type.clone() } })]
    pub elements: Vec<KVPair>,
}

#[derive(Debug, BinRead)]
#[br(little, import { key_type: ADAString, value_type: ADAString })]
pub struct KVPair {
    #[br(args { prop_type: key_type.clone() })]
    pub key: Property,
    #[br(args { prop_type: value_type.clone() })]
    pub value: Property,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct NameProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub value: ADAString,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ObjectProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub level_name: ADAString,
    pub path_name: ADAString,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct SetProperty {
    pub size: i32,
    pub index: i32,
    pub element_type: ADAString,
    #[br(pad_before = 1 + 4)]
    pub element_count: i32,
    #[br(count = element_count, args { inner: PropertyBinReadArgs { prop_type: element_type.clone() } })]
    pub elements: Vec<Property>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct StrProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub value: ADAString,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct StructProperty {
    pub size: i32,
    pub index: i32,
    pub struct_type: ADAString,
    #[br(pad_before = 8 + 8 + 1, args { is_struct_property_payload: true })]
    pub typed_data: TypedData,
}

#[derive(Debug, BinRead)]
#[br(little, import { is_struct_property_payload: bool })]
pub enum TypedData {
    PropertyList(PropertyList),
    Box {
        min_x: f32,
        min_y: f32,
        min_z: f32,
        max_x: f32,
        max_y: f32,
        max_z: f32,
        #[br(map(|x: i32| x != 0))]
        is_value: bool,
    },
    FluidBox(f32),
    InventoryItem {
        #[br(pad_before = 4)]
        item_type: ADAString,
        level_name: ADAString,
        path_name: ADAString,
        #[br(if(is_struct_property_payload))]
        extra: IntProperty,
    },
    LinearColor {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    Quat {
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    },
    RailroadTrackPosition {
        level_name: ADAString,
        path_name: ADAString,
        offset: f32,
        forward: f32,
    },
    Vector {
        x: f32,
        y: f32,
        z: f32,
    },
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct TextProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub flags: i32,
    pub history_type: i8,
    #[br(map(|x: i32| x != 0))]
    pub is_culture_invariant: bool,
    pub value: ADAString,
}

#[derive(Debug)]
pub struct PropertyList(pub Vec<Property>);

impl BinRead for PropertyList {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (): Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        // read properties until a special "None" property is encountered.
        // properties start with a name string, then a type string, then the property data.
        let mut properties = Vec::new();

        loop {
            let name = ADAString::read_options(reader, endian, ())?;
            if name == "None" {
                break;
            }

            let prop_type = ADAString::read_options(reader, endian, ())?;
            properties.push(Property::read_options(
                reader,
                endian,
                PropertyBinReadArgs { prop_type },
            )?);
        }

        Ok(PropertyList(properties))
    }
}

pub struct Parser<R> {
    data: R,
}

impl<R> Parser<R>
where
    R: Read + Seek,
{
    pub fn new(data: R) -> Self {
        Parser { data }
    }

    pub fn read_header(&mut self) -> Result<SaveFileHeader, Error> {
        self.data.read_le().map_err(Error::BinRead)
    }

    pub fn read_compressed_body_chunk(&mut self) -> Result<Option<CompressedSaveFileBody>, Error> {
        match self.data.read_le() {
            Ok(chunk) => Ok(Some(chunk)),
            Err(e) => match e {
                binrw::Error::Io(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                    Ok(None)
                }
                e => Err(e.into()),
            },
        }
    }
}
