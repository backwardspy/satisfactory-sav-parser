//! https://satisfactory.fandom.com/wiki/Save_files
//! https://github.com/moritz-h/satisfactory-3d-map/blob/master/docs/SATISFACTORY_SAVE.md#type-and-object-reference
use std::io::{Read, Seek};

use binrw::{BinRead, BinReaderExt, BinResult};
use thiserror::Error;
use types::{array::Array, string::String};

mod types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read save file")]
    BinRead(#[from] binrw::Error),
}

fn adabool(value: u32) -> bool {
    value != 0
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct SaveFileHeader {
    pub save_header_version: i32,
    pub save_version: i32,
    pub build_version: i32,
    pub map_name: String,
    pub map_options: String,
    pub session_name: String,
    pub play_duration_seconds: i32,
    pub save_date_time: i64,
    pub session_visibility: i8,
    pub editor_object_version: i32,
    pub mod_metadata: String,
    #[br(map(adabool))]
    pub is_modded_save: bool,
    pub save_identifier: String,
    #[br(map(adabool))]
    pub is_partitioned_world: bool,
    pub md5_hash: [u8; 20],
    #[br(map(adabool))]
    pub is_creative_mode_enabled: bool,
}

#[derive(Debug, BinRead)]
#[br(little, magic = 0x9E2A83C1u32)]
pub struct CompressedSaveFileBody {
    pub archive_header: u32,
    #[br(assert(max_chunk_size == 128 * 1024))]
    pub max_chunk_size: i64,
    // 3 = zlib
    #[br(if(archive_header == 0x22222222), assert(compressor_num == 3))]
    pub compressor_num: u8,
    pub compressed_size_summary: i64,
    pub uncompressed_size_summary: i64,
    pub compressed_size: i64,
    pub uncompressed_size: i64,
    #[br(count = compressed_size)]
    pub chunk_bytes: Vec<u8>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct SaveFileBody {
    pub uncompressed_size: i64,
    // pub sublevel_count: i32,
    #[br(args { inner: LevelBinReadArgs { is_sublevel: true } })]
    pub sub_levels: Array<Level>,
    #[br(args { is_sublevel: false })]
    pub persistent_level: Level,
    pub object_references: Array<ObjectReference>,
}

#[derive(Debug, BinRead)]
#[br(little, import { is_sublevel: bool })]
pub struct Level {
    #[br(if(is_sublevel))]
    pub sublevel_name: String,
    pub object_header_and_collectables_size: i32,
    pub object_headers: Array<ObjectHeader>,
    pub collectables: Array<ObjectReference>,
    pub objects_size: i32,
    pub object_count: i32,
    #[br(parse_with = parse_objects, args(&object_headers.0))]
    pub objects: Vec<Object>,
    pub collections_2: Array<ObjectReference>,
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
    pub type_path: String,
    pub root_object: String,
    pub instance_name: String,
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
    pub type_path: String,
    pub root_object: String,
    pub instance_name: String,
    pub parent_actor_name: String,
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
    pub parent_object_root: String,
    pub parent_object_name: String,
    pub components: Array<ObjectReference>,
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
    pub level_name: String,
    pub path_name: String,
}

#[derive(Debug, BinRead)]
#[br(little, import { prop_type: String })]
pub enum Property {
    #[br(pre_assert(prop_type == "ArrayProperty"))]
    Array(ArrayProperty),
    #[br(pre_assert(prop_type == "BoolProperty"))]
    Bool(BoolProperty),
    #[br(pre_assert(prop_type == "ByteProperty"))]
    Byte(ByteProperty),
    #[br(pre_assert(prop_type == "EnumProperty"))]
    Enum(EnumProperty),
    #[br(pre_assert(prop_type == "FloatProperty"))]
    Float(FloatProperty),
    #[br(pre_assert(prop_type == "IntProperty"))]
    Int(IntProperty),
    #[br(pre_assert(prop_type == "Int64Property"))]
    Int64(Int64Property),
    #[br(pre_assert(prop_type == "MapProperty"))]
    Map(MapProperty),
    #[br(pre_assert(prop_type == "NameProperty"))]
    Name(NameProperty),
    #[br(pre_assert(prop_type == "ObjectProperty"))]
    Object(ObjectProperty),
    #[br(pre_assert(prop_type == "SetProperty"))]
    Set(SetProperty),
    #[br(pre_assert(prop_type == "StrProperty"))]
    Str(StrProperty),
    #[br(pre_assert(prop_type == "StructProperty"))]
    Struct(StructProperty),
    #[br(pre_assert(prop_type == "TextProperty"))]
    Text(TextProperty),
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ArrayProperty {
    pub size: i32,
    pub index: i32,
    pub element_type: String,
    #[br(pad_before = 1)]
    #[br(args { inner: PropertyBinReadArgs { prop_type: element_type.clone() } })]
    pub elements: Array<Property>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct BoolProperty {
    #[br(pad_before = 4)]
    pub index: i32,
    #[br(map(adabool), pad_after = 1)]
    pub value: bool,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ByteProperty {
    pub size: i32,
    pub index: i32,
    pub prop_type: String,
    #[br(args { prop_type: prop_type.clone() }, pad_before = 1)]
    pub value: BytePropertyValue,
}

#[derive(Debug, BinRead)]
#[br(little, import { prop_type: String })]
pub enum BytePropertyValue {
    #[br(pre_assert(prop_type == "None"))]
    Byte(i8),
    #[br(pre_assert(prop_type != "None"))]
    String(String),
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct EnumProperty {
    pub size: i32,
    pub index: i32,
    pub prop_type: String,
    #[br(pad_before = 1)]
    pub value: String,
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
    pub key_type: String,
    pub value_type: String,
    #[br(pad_before = 1)]
    pub mode_type: i32,
    #[br(args { inner: KVPairBinReadArgs { key_type: key_type.clone(), value_type: value_type.clone() } })]
    pub elements: Array<KVPair>,
}

#[derive(Debug, BinRead)]
#[br(little, import { key_type: String, value_type: String })]
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
    pub value: String,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct ObjectProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub level_name: String,
    pub path_name: String,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct SetProperty {
    pub size: i32,
    pub index: i32,
    pub element_type: String,
    #[br(pad_before = 1 + 4)]
    pub element_count: i32,
    #[br(args { inner: PropertyBinReadArgs { prop_type: element_type.clone() } })]
    pub elements: Array<Property>,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct StrProperty {
    pub size: i32,
    pub index: i32,
    #[br(pad_before = 1)]
    pub value: String,
}

#[derive(Debug, BinRead)]
#[br(little)]
pub struct StructProperty {
    pub size: i32,
    pub index: i32,
    pub struct_type: String,
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
        #[br(map(adabool))]
        is_value: bool,
    },
    FluidBox(f32),
    InventoryItem {
        #[br(pad_before = 4)]
        item_type: String,
        level_name: String,
        path_name: String,
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
        level_name: String,
        path_name: String,
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
    #[br(map(adabool))]
    pub is_culture_invariant: bool,
    pub value: String,
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
            let name = String::read_options(reader, endian, ())?;
            if name == "None" {
                break;
            }

            let prop_type = String::read_options(reader, endian, ())?;
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
