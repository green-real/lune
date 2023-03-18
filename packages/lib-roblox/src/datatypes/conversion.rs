use mlua::prelude::*;

use rbx_dom_weak::types::{Variant as RbxVariant, VariantType as RbxVariantType};

use crate::datatypes::extension::RbxVariantExt;

use super::*;

pub(crate) trait LuaToRbxVariant<'lua> {
    fn lua_to_rbx_variant(
        &self,
        lua: &'lua Lua,
        variant_type: RbxVariantType,
    ) -> DatatypeConversionResult<RbxVariant>;
}

pub(crate) trait RbxVariantToLua<'lua>: Sized {
    fn rbx_variant_to_lua(lua: &'lua Lua, variant: &RbxVariant) -> DatatypeConversionResult<Self>;
}

/*

    Blanket trait implementations for converting between LuaValue and rbx_dom Variant values

    These should be considered stable and done, already containing all of the known primitives

    See bottom of module for implementations between our custom datatypes and lua userdata

*/

impl<'lua> RbxVariantToLua<'lua> for LuaValue<'lua> {
    fn rbx_variant_to_lua(lua: &'lua Lua, variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;

        use rbx_dom_weak::types as rbx;
        use RbxVariant as Rbx;

        match LuaAnyUserData::rbx_variant_to_lua(lua, variant) {
            Ok(value) => Ok(LuaValue::UserData(value)),
            Err(e) => match variant {
                Rbx::Bool(b) => Ok(LuaValue::Boolean(*b)),
                Rbx::Int64(i) => Ok(LuaValue::Number(*i as f64)),
                Rbx::Int32(i) => Ok(LuaValue::Number(*i as f64)),
                Rbx::Float64(n) => Ok(LuaValue::Number(*n)),
                Rbx::Float32(n) => Ok(LuaValue::Number(*n as f64)),
                Rbx::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
                Rbx::Content(s) => Ok(LuaValue::String(
                    lua.create_string(AsRef::<str>::as_ref(s))?,
                )),
                Rbx::BinaryString(s) => {
                    let encoded = STANDARD_NO_PAD.encode(AsRef::<[u8]>::as_ref(s));
                    Ok(LuaValue::String(lua.create_string(&encoded)?))
                }

                // NOTE: We need this special case here to handle default (nil)
                // physical properties since our PhysicalProperties datatype
                // implementation does not handle default at all, only custom
                Rbx::PhysicalProperties(rbx::PhysicalProperties::Default) => Ok(LuaValue::Nil),

                _ => Err(e),
            },
        }
    }
}

impl<'lua> LuaToRbxVariant<'lua> for LuaValue<'lua> {
    fn lua_to_rbx_variant(
        &self,
        lua: &'lua Lua,
        variant_type: RbxVariantType,
    ) -> DatatypeConversionResult<RbxVariant> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;

        use rbx_dom_weak::types as rbx;
        use RbxVariant as Rbx;
        use RbxVariantType as RbxType;

        match (self, variant_type) {
            (LuaValue::Boolean(b), RbxType::Bool) => Ok(Rbx::Bool(*b)),

            (LuaValue::Integer(i), RbxType::Int64) => Ok(Rbx::Int64(*i as i64)),
            (LuaValue::Integer(i), RbxType::Int32) => Ok(Rbx::Int32(*i)),
            (LuaValue::Integer(i), RbxType::Float64) => Ok(Rbx::Float64(*i as f64)),
            (LuaValue::Integer(i), RbxType::Float32) => Ok(Rbx::Float32(*i as f32)),

            (LuaValue::Number(n), RbxType::Int64) => Ok(Rbx::Int64(*n as i64)),
            (LuaValue::Number(n), RbxType::Int32) => Ok(Rbx::Int32(*n as i32)),
            (LuaValue::Number(n), RbxType::Float64) => Ok(Rbx::Float64(*n)),
            (LuaValue::Number(n), RbxType::Float32) => Ok(Rbx::Float32(*n as f32)),

            (LuaValue::String(s), RbxType::String) => Ok(Rbx::String(s.to_str()?.to_string())),
            (LuaValue::String(s), RbxType::Content) => {
                Ok(Rbx::Content(s.to_str()?.to_string().into()))
            }
            (LuaValue::String(s), RbxType::BinaryString) => {
                Ok(Rbx::BinaryString(STANDARD_NO_PAD.decode(s)?.into()))
            }

            // NOTE: We need this special case here to handle default (nil)
            // physical properties since our PhysicalProperties datatype
            // implementation does not handle default at all, only custom
            (LuaValue::Nil, RbxType::PhysicalProperties) => {
                Ok(Rbx::PhysicalProperties(rbx::PhysicalProperties::Default))
            }

            (LuaValue::UserData(u), d) => u.lua_to_rbx_variant(lua, d),

            (v, d) => Err(DatatypeConversionError::ToRbxVariant {
                to: d.variant_name(),
                from: v.type_name(),
                detail: None,
            }),
        }
    }
}

/*

    Trait implementations for converting between all of
    our custom datatypes and generic Lua userdata values

    NOTE: When adding a new datatype, make sure to add it below to _both_
    of the traits and not just one to allow for bidirectional conversion

*/

impl<'lua> RbxVariantToLua<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn rbx_variant_to_lua(lua: &'lua Lua, variant: &RbxVariant) -> DatatypeConversionResult<Self> {
		use super::types::*;

        use rbx_dom_weak::types as rbx;
        use RbxVariant as Rbx;

        /*
            NOTES:

            1. Enum is intentionally left out here, it has a custom
               conversion going from instance property > datatype instead,
               check `EnumItem::from_instance_property` for specifics

            2. PhysicalProperties can only be converted if they are custom
               physical properties, since default physical properties values
               depend on what other related properties an instance might have

        */
        Ok(match variant.clone() {
            Rbx::Axes(value)  => lua.create_userdata(Axes::from(value))?,
            Rbx::Faces(value) => lua.create_userdata(Faces::from(value))?,

            Rbx::CFrame(value) => lua.create_userdata(CFrame::from(value))?,

            Rbx::BrickColor(value)    => lua.create_userdata(BrickColor::from(value))?,
            Rbx::Color3(value)        => lua.create_userdata(Color3::from(value))?,
            Rbx::Color3uint8(value)   => lua.create_userdata(Color3::from(value))?,
            Rbx::ColorSequence(value) => lua.create_userdata(ColorSequence::from(value))?,

            Rbx::Font(value) => lua.create_userdata(Font::from(value))?,

            Rbx::NumberRange(value)    => lua.create_userdata(NumberRange::from(value))?,
            Rbx::NumberSequence(value) => lua.create_userdata(NumberSequence::from(value))?,

            Rbx::Ray(value) => lua.create_userdata(Ray::from(value))?,

            Rbx::Rect(value)  => lua.create_userdata(Rect::from(value))?,
            Rbx::UDim(value)  => lua.create_userdata(UDim::from(value))?,
            Rbx::UDim2(value) => lua.create_userdata(UDim2::from(value))?,

            Rbx::Region3(value)      => lua.create_userdata(Region3::from(value))?,
            Rbx::Region3int16(value) => lua.create_userdata(Region3int16::from(value))?,
            Rbx::Vector2(value)      => lua.create_userdata(Vector2::from(value))?,
            Rbx::Vector2int16(value) => lua.create_userdata(Vector2int16::from(value))?,
            Rbx::Vector3(value)      => lua.create_userdata(Vector3::from(value))?,
            Rbx::Vector3int16(value) => lua.create_userdata(Vector3int16::from(value))?,

            Rbx::OptionalCFrame(value) => match value {
                Some(value) => lua.create_userdata(CFrame::from(value))?,
                None => lua.create_userdata(CFrame::IDENTITY)?
            },

            Rbx::PhysicalProperties(rbx::PhysicalProperties::Custom(value)) => {
                lua.create_userdata(PhysicalProperties::from(value))?
            },

            v => {
                return Err(DatatypeConversionError::FromRbxVariant {
                    from: v.variant_name(),
                    to: "userdata",
                    detail: Some("Type not supported".to_string()),
                })
            }
        })
    }
}

impl<'lua> LuaToRbxVariant<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn lua_to_rbx_variant(
        &self,
        _: &'lua Lua,
        variant_type: RbxVariantType,
    ) -> DatatypeConversionResult<RbxVariant> {
        use super::types::*;

        use rbx_dom_weak::types as rbx;

        let f = match variant_type {
            RbxVariantType::Axes  => convert::<Axes,  rbx::Axes>,
            RbxVariantType::Faces => convert::<Faces, rbx::Faces>,

            RbxVariantType::CFrame => convert::<CFrame, rbx::CFrame>,

            RbxVariantType::BrickColor    => convert::<BrickColor,    rbx::BrickColor>,
            RbxVariantType::Color3        => convert::<Color3,        rbx::Color3>,
            RbxVariantType::Color3uint8   => convert::<Color3,        rbx::Color3uint8>,
            RbxVariantType::ColorSequence => convert::<ColorSequence, rbx::ColorSequence>,

            RbxVariantType::Enum => convert::<EnumItem, rbx::Enum>,

            RbxVariantType::Font => convert::<Font, rbx::Font>,

            RbxVariantType::NumberRange    => convert::<NumberRange,    rbx::NumberRange>,
            RbxVariantType::NumberSequence => convert::<NumberSequence, rbx::NumberSequence>,

            RbxVariantType::Rect  => convert::<Rect,  rbx::Rect>,
            RbxVariantType::UDim  => convert::<UDim,  rbx::UDim>,
            RbxVariantType::UDim2 => convert::<UDim2, rbx::UDim2>,

            RbxVariantType::Ray => convert::<Ray, rbx::Ray>,

            RbxVariantType::Region3      => convert::<Region3,      rbx::Region3>,
            RbxVariantType::Region3int16 => convert::<Region3int16, rbx::Region3int16>,
            RbxVariantType::Vector2      => convert::<Vector2,      rbx::Vector2>,
            RbxVariantType::Vector2int16 => convert::<Vector2int16, rbx::Vector2int16>,
            RbxVariantType::Vector3      => convert::<Vector3,      rbx::Vector3>,
            RbxVariantType::Vector3int16 => convert::<Vector3int16, rbx::Vector3int16>,

            RbxVariantType::OptionalCFrame => return match self.borrow::<CFrame>() {
                Ok(value) => Ok(RbxVariant::OptionalCFrame(Some(rbx::CFrame::from(*value)))),
                Err(e) => Err(lua_userdata_error_to_conversion_error(variant_type, e)),
            },

            RbxVariantType::PhysicalProperties => return match self.borrow::<PhysicalProperties>() {
                Ok(value) => {
                    let props = rbx::CustomPhysicalProperties::from(*value);
                    let custom = rbx::PhysicalProperties::Custom(props);
                    Ok(RbxVariant::PhysicalProperties(custom))
                },
                Err(e) => Err(lua_userdata_error_to_conversion_error(variant_type, e)),
            },

            _ => return Err(DatatypeConversionError::ToRbxVariant {
                to: variant_type.variant_name(),
                from: "userdata",
                detail: Some("Type not supported".to_string()),
            }),
        };

        f(self, variant_type)
    }
}

fn convert<Datatype, RbxType>(
    userdata: &LuaAnyUserData,
    variant_type: RbxVariantType,
) -> DatatypeConversionResult<RbxVariant>
where
    Datatype: LuaUserData + Clone + 'static,
    RbxType: From<Datatype> + Into<RbxVariant>,
{
    match userdata.borrow::<Datatype>() {
        Ok(value) => Ok(RbxType::from(value.clone()).into()),
        Err(e) => Err(lua_userdata_error_to_conversion_error(variant_type, e)),
    }
}

fn lua_userdata_error_to_conversion_error(
    variant_type: RbxVariantType,
    error: LuaError,
) -> DatatypeConversionError {
    match error {
        LuaError::UserDataTypeMismatch => DatatypeConversionError::ToRbxVariant {
            to: variant_type.variant_name(),
            from: "userdata",
            detail: Some("Type mismatch".to_string()),
        },
        e => DatatypeConversionError::ToRbxVariant {
            to: variant_type.variant_name(),
            from: "userdata",
            detail: Some(format!("Internal error: {e}")),
        },
    }
}