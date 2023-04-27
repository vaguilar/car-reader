use std::fmt;

use binrw::BinRead;
use binrw::BinResult;
use binrw::NullString;
use bitfield_struct::bitfield;
use hex::ToHex;
use num_derive::FromPrimitive;
use serde::Serialize;
use serde::Serializer;

use crate::common::dynamic_string::dynamic_length_string_parser;
use crate::coregraphics;
use crate::coreui::rendition::Rendition;
use crate::coreui::tlv::parse_tlv_data;
use crate::coreui::tlv::RenditionType;

#[derive(Debug, BinRead)]
#[brw(little)]
pub struct CarHeader {
    _magic: u32,
    pub core_ui_version: u32,
    pub storage_version: u32,
    pub storage_timestamp: u32,
    pub rendition_count: u32,
    #[br(pad_size_to = 128)]
    pub main_version_string: MyNullString,
    #[br(pad_size_to = 256)]
    pub version_string: MyNullString,
    pub uuid: [u8; 16],
    pub associated_checksum: u32,
    pub schema_version: u32,
    pub color_space_id: u32,
    pub key_semantics: u32,
}

#[derive(Debug, BinRead)]
#[brw(little)]
pub struct CarExtendedMetadata {
    _magic: u32,
    #[br(pad_size_to = 256)]
    pub thinning_arguments: MyNullString,
    #[br(pad_size_to = 256)]
    pub deployment_platform_version: MyNullString,
    #[br(pad_size_to = 256)]
    pub deployment_platform: MyNullString,
    #[br(pad_size_to = 256)]
    pub authoring_tool: MyNullString,
}

#[derive(Debug, BinRead)]
#[brw(little, magic = b"tmfk")]
pub struct KeyFormat {
    pub _version: u32,
    pub _max_count: u32,
    #[br(count = _max_count)]
    pub attribute_types: Vec<RenditionAttributeType>,
}

#[derive(Debug, BinRead, FromPrimitive, Clone, Copy, PartialEq, Eq, Hash)]
#[br(repr(u32))]
pub enum RenditionAttributeType2 {
    Look = 0,
    Element,
    Part,
    Size,
    Direction,
    PlaceHolder,
    Value,
    Appearance,
    Dimension1,
    Dimension2,
    State,
    Layer,
    Scale,
    Unknown13,
    PresentationState,
    Idiom,
    Subtype,
    Identifier,
    PreviousValue,
    PreviousState,
    SizeClassHorizontal,
    SizeClassVertical,
    MemoryClass,
    GraphicsClass,
    DisplayGamut,
    DeploymentTarget,
}

#[derive(Debug, BinRead, FromPrimitive, Clone, Copy, PartialEq, Eq, Hash)]
#[br(repr(u32))]
pub enum RenditionAttributeType {
    Look = 0,
    Element,
    Part,
    Size,
    Direction,
    PlaceHolder,
    Value,
    Appearance,
    Dimension1,
    Dimension2,
    State,
    Layer,
    Scale,
    Unknown13,
    PresentationState,
    Idiom,
    Subtype,
    Identifier,
    PreviousValue,
    PreviousState,
    SizeClassHorizontal,
    SizeClassVertical,
    MemoryClass,
    GraphicsClass,
    DisplayGamut,
    DeploymentTarget,
}

impl Serialize for RenditionAttributeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("kCRTheme{}Name", self.to_string());
        serializer.serialize_str(&s)
    }
}

#[derive(BinRead, Debug)]
#[brw(little, magic = b"tmfk")]
pub struct RenditionKeyFmt {
    pub _version: u32,
    pub _maximum_rendition_key_token_count: u32,
    #[br(count = _maximum_rendition_key_token_count)]
    #[br(dbg)]
    pub _rendition_key_tokens: Vec<RenditionKeyToken>,
}

#[derive(BinRead, Debug)]
#[brw(little)]
pub struct RenditionKeyToken {
    _cursor_hotspot: (u16, u16),
    _number_of_attributes: u16,
    #[br(count = _number_of_attributes)]
    pub attributes: Vec<RenditionAttribute>,
}

#[derive(BinRead, Debug)]
pub struct RenditionAttribute {
    #[br(parse_with = parse_rendition_attribute_type_u16)]
    pub name: RenditionAttributeType,
    // pub name: u16, // RenditionAttributeType
    pub value: u16,
}

#[derive(BinRead, Debug, Clone)]
#[brw(little)]
pub struct CSIMetadata {
    _mod_time: u32,
    pub layout: RenditionLayoutType,
    _zero: u16,
    #[br(parse_with = dynamic_length_string_parser(128))]
    pub name: String,
}

#[derive(BinRead, Debug, Clone)]
pub struct CSIBitmapList {
    pub tlv_length: u32,
    pub unknown: u32,
    pub zero: u32,
    pub rendition_length: u32,
}

#[derive(BinRead, Debug, Clone)]
#[brw(little)]
pub struct CSIHeader {
    #[br(parse_with = dynamic_length_string_parser(4))]
    pub magic: String,
    pub version: u32,
    pub rendition_flags: RenditionFlags,
    pub width: u32,
    pub height: u32,
    pub scale_factor: Scale,
    pub pixel_format: PixelFormat,
    pub color_space: coregraphics::ColorSpace,
    pub csimetadata: CSIMetadata,
    pub csibitmaplist: CSIBitmapList,
    #[br(args(csibitmaplist.tlv_length))]
    #[br(parse_with = parse_tlv_data)]
    pub tlv_data: Vec<RenditionType>,
    pub rendition_data: Rendition,
}

/*
CUI::NamedImageProperties
"{_cuiniproperties=\"isVectorBased\"b1\"hasSliceInformation\"b1\"hasAlignmentInformation\"b1\"resizingMode\"b2\"templateRenderingMode\"b3\"exifOrientation\"b4\"isAlphaCropped\"b1\"isFlippable\"b1\"isTintable\"b1\"preservedVectorRepresentation\"b1\"_reserved\"b16}", 0
*/
#[bitfield(u32)]
pub struct NamedImageProperties {
    #[bits(1)]
    is_vector_based: bool,
    #[bits(1)]
    has_slice_information: bool,
    #[bits(1)]
    has_alignment_information: bool,
    #[bits(2)]
    resizing_mode: u8,
    #[bits(3)]
    template_rendering_mode: u8,
    #[bits(4)]
    exif_orientation: u8,
    #[bits(1)]
    is_alpha_cropped: bool,
    #[bits(1)]
    is_flippable: bool,
    #[bits(1)]
    is_tintable: bool,
    #[bits(1)]
    preserved_vector_representation: bool,
    #[bits(16)]
    _reserved: u16,
}

/*
"{cuithemerenditionrenditionflags=\"isVectorBased\"b1\"isOpaque\"b1\"bitmapEncoding\"b4\"optOutOfThinning\"b1\"isFlippable\"b1\"isTintable\"b1\"preservedVectorRepresentation\"b1\"reserved\"b22}", 0
*/
#[derive(BinRead, Debug, Clone)]
pub struct RenditionFlags {
    // these values are all packed into one u32
    // is_header_flagged_fpo: u32,
    // is_excluded_from_contrast_filter: u32,
    // is_vector_based: u32,
    // is_opaque: u32,
    // bitmap_encoding: u32,
    // opt_out_of_thinning: u32,
    // is_flippable: u32,
    // is_tintable: u32,
    // preserved_vector_representation: u32,
    // reserved: u32,
    flags: u32,
}

impl RenditionFlags {
    pub fn is_opaque(&self) -> bool {
        ((self.flags >> 3) & 1) != 0
    }

    pub fn bitmap_encoding(&self) -> &str {
        match (self.flags >> 4) & 0b1111 {
            1 => "RGB",
            _ => "???",
        }
    }
}

// #[derive(BinRead, Clone, Debug, Serialize)]
// #[br(repr(u32))]
// pub enum ColorSpace {
//     #[serde(rename = "srgb")]
//     SRGB = 0,
//     #[serde(rename = "gray gamma 22")]
//     GrayGamma2_2,
//     #[serde(rename = "p3")]
//     DisplayP3,
//     #[serde(rename = "extended srgb")]
//     ExtendedRangeSRGB,
//     #[serde(rename = "extended linear srgb")]
//     ExtendedLinearSRGB,
//     #[serde(rename = "extended gray")]
//     ExtendedGray,
//     Unknown = 14,
// }

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(repr(u32))]
pub enum PixelFormat {
    None = 0,
    ARGB = 0x41524742,
    Data = 0x44415441,
    Gray = 0x47413820,
    JPEG = 0x4A504547,
}

#[derive(BinRead, Clone, FromPrimitive)]
#[br(repr(u32))]
pub enum Scale {
    None = 0,
    X1 = 100,
    X2 = 200,
    X3 = 300,
}

impl fmt::Debug for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scale::None => write!(f, "None"),
            Scale::X1 => write!(f, "1x"),
            Scale::X2 => write!(f, "2x"),
            Scale::X3 => write!(f, "3x"),
        }
    }
}

impl Serialize for Scale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Scale::None => serializer.serialize_u32(1),
            Scale::X1 => serializer.serialize_u32(1),
            Scale::X2 => serializer.serialize_u32(2),
            Scale::X3 => serializer.serialize_u32(3),
        }
    }
}

#[derive(BinRead, Debug, PartialOrd, PartialEq, Serialize, Clone, Copy)]
#[br(repr(u16))]
pub enum RenditionLayoutType {
    TextEffect = 0x007,
    Vector = 0x009,
    Image = 0x00C, // ???
    Data = 0x3E8,
    ExternalLink = 0x3E9,
    LayerStack = 0x3EA,
    InternalReference = 0x3EB,
    PackedImage = 0x3EC,
    NameList = 0x3ED,
    UnknownAddObject = 0x3EE,
    Texture = 0x3EF,
    TextureImage = 0x3F0,
    Color = 0x3F1,
    MultisizeImage = 0x3F2,
    LayerReference = 0x3F4,
    ContentRendition = 0x3F5,
    RecognitionObject = 0x3F6,
}

#[derive(Debug, BinRead, FromPrimitive, Clone, Copy, PartialEq)]
#[br(repr(u32))]
pub enum CoreThemeImageSubtype {
    CoreThemeOnePartFixedSize = 10,
    CoreThemeOnePartTile = 11,
    CoreThemeOnePartScale = 12,
    CoreThemeThreePartHTile = 20,
    CoreThemeThreePartHScale = 21,
    CoreThemeThreePartHUniform = 22,
    CoreThemeThreePartVTile = 23,
    CoreThemeThreePartVScale = 24,
    CoreThemeThreePartVUniform = 25,
    CoreThemeNinePartTile = 30,
    CoreThemeNinePartScale = 31,
    CoreThemeNinePartHorizontalUniformVerticalScale = 32,
    CoreThemeNinePartHorizontalScaleVerticalUniform = 33,
    CoreThemeNinePartEdgesOnly = 34,
    CoreThemeManyPartLayoutUnknown = 40,
    CoreThemeAnimationFilmstrip = 50,
}

#[derive(Debug)]
pub struct MyNullString(pub NullString);

impl Serialize for MyNullString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl BinRead for MyNullString {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        NullString::read_options(reader, endian, args).map(|s| MyNullString(s))
    }
}

impl fmt::Display for RenditionAttributeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[binrw::parser(reader, endian)]
fn parse_rendition_attribute_type_u16() -> BinResult<RenditionAttributeType> {
    let raw = u16::read_options(reader, endian, ())?;
    let attribute = num::FromPrimitive::from_u16(raw);
    dbg!(raw);
    attribute.ok_or(binrw::Error::NoVariantMatch {
        pos: reader.stream_position().unwrap(),
    })
}

#[derive(BinRead)]
#[brw(little)]
pub struct HexString36(pub [u8; 36]);
impl fmt::Debug for HexString36 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.serialize_str(&self.0.encode_hex::<String>())
    }
}

#[derive(BinRead)]
#[brw(little)]
pub struct HexString22(pub [u8; 22]);
impl fmt::Debug for HexString22 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.serialize_str(&self.0.encode_hex::<String>())
    }
}
