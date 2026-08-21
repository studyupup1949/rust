use std::{
    borrow::Cow,
    io::{Error, Read, Write},
};

use super::{
    serialise::Serialise,
    deserialise::{Deserialise, ParseError},
};

#[derive(Copy, Clone, Debug)]
pub struct VarInt(pub i32);
#[derive(Copy, Clone, Debug)]
pub struct VarLong(pub i64);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position(pub i64, pub i64, pub i64);

impl std::ops::Deref for VarInt {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for VarLong {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! impl_ser_de {
    ($(
        pub struct $name:ident$(<$($life:lifetime),+>)* {
            $(
            $(#[meta = $meta:expr])*
            $field_name:ident: $field_type:ty,
            )*
        }
    )*) => {
        $(
        #[derive(Clone)]    // so we can put it in a Cow
        pub struct $name$(<$($life),*>)* {
            $(
            $field_name: $field_type,
            )*
        }

        impl$(<$($life),*>)* Serialise for $name$(<$($life),*>)* {
            fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
                $(
                self.$field_name.serialise(&mut writer)?;
                )*
                Ok(())
            }
        }

        impl$(<$($life),*>)* Deserialise<()> for $name$(<$($life),*>)* {
            fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
                #![allow(unused_parens)]
                $(
                let $field_name = <$field_type>::deserialise(&mut reader, ($($meta),*))?;
                )*
                Ok($name {
                    $($field_name,)*
                })
            }
        }
        )*
    };
}

macro_rules! impl_ser_de_enum {
    ($(
        pub enum $name:ident$(<$($life:lifetime),+>)* {
            $(
            $variant:ident ($($id:pat)|*) {
                $(
                $(#[meta = $meta:expr])*
                $field_name:ident: $field_type:ty,
                )*
            },
            )*
        }
    )*) => {
        $(
        #[derive(Clone)]    // so we can put it in a Cow
        pub enum $name$(<$($life),*>)* {
            $(
            $variant {
                $(
                $field_name: $field_type,
                )*
            },
            )*
        }

        impl$(<$($life),*>)* Serialise for $name$(<$($life),*>)* {
            fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
                match self {
                    $(
                    $name::$variant { $($field_name,)* } => {
                        $(
                        $field_name.serialise(&mut writer)?;
                        )*
                    },
                    )*
                }
                Ok(())
            }
        }

        impl$(<$($life),*>)* Deserialise<i32> for $name$(<$($life),*>)*{
            fn deserialise<R: Read>(mut reader: R, meta: i32) -> Result<Self, ParseError> {
                #![allow(unused_parens)]
                match meta {
                    $(
                    $($id)|* => {
                        $(
                        let $field_name = <$field_type as Deserialise<_>>::deserialise(&mut reader, ($($meta),*))?;
                        )*
                        Ok($name::$variant { $($field_name,)* })
                    }
                    )*
                    _ => Err(ParseError::InvalidEnumDiscriminant(stringify!($name), meta)),
                }
            }
        }
        )*
    };
}

impl_ser_de! {
    pub struct Statistic {
        category_id: VarInt,
        statistic_id: VarInt,
        value: VarInt,
    }

    pub struct MultiBlockChangeRecord {
        horiz_pos: u8,
        y: u8,
        block_id: VarInt,
    }

    pub struct TabCompleteMatch<'a, 'b> {
        valid_match: Cow<'a, str>,
        has_tooltip: bool,
        #[meta = has_tooltip]
        tooltip: Option<Cow<'b, str>>,
    }

    pub struct Slot {
        present: bool,
        #[meta = present]
        item_id: Option<VarInt>,
        #[meta = present]
        item_count: Option<u8>,
        #[meta = present]
        nbt: Option<nbt::Value>,
    }

    pub struct ExplosionBlockOffset {
        x: i8,
        y: i8,
        z: i8,
    }

    pub struct Icon<'a> {
        icon_type: VarInt,
        x: u8,
        z: u8,
        direction: u8,
        has_display_name: bool,
        #[meta = has_display_name]
        display_name: Option<Cow<'a, str>>,
    }

    pub struct EndCombat {
        duration: VarInt,
        entity_id: i32,
    }

    pub struct EntityDead<'a> {
        player_id: VarInt,
        entity_id: i32,
        message: Cow<'a, str>,
    }

    pub struct PlayerListProperty<'a, 'b, 'c> {
        name: Cow<'a, str>,
        value: Cow<'b, str>,
        is_signed: bool,
        signature: Cow<'c, str>,
    }

    pub struct EntityMetadata<'a, 'b, 'c> {
        index: u8,
        #[meta = index != 0xff]
        value_type: Option<VarInt>,
        #[meta = (index != 0xff, index as i32)]
        value: Option<EntityMetadataValue<'a, 'b, 'c>>,
    }

    pub struct Particle {
        id: VarInt,
        #[meta = *id]
        data: ParticleData,
    }

    // these are all going to be initialised to 'static lol
    pub struct AdvancementMapping<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j> {
        identifier: Cow<'a, str>,
        advancement: Advancement<'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j>,
    }

    pub struct Advancement<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i> {
        has_parent: bool,
        #[meta = has_parent]
        parent_id: Option<Cow<'a, str>>,
        has_display: bool,
        #[meta = has_display]
        display: Option<AdvancementDisplay<'b, 'c, 'd>>,
        criteria_count: VarInt,
        #[meta = *criteria_count as usize]
        criteria: Cow<'e, [Cow<'f, str>]>,
        requirements_count: VarInt,
        // don't ask, I don't know either
        #[meta = *requirements_count as usize]
        requirements: Cow<'g, [Requirement<'h, 'i>]>,
    }

    pub struct AdvancementDisplay<'a, 'b, 'c> {
        title: Cow<'a, str>,
        description: Cow<'b, str>,
        icon: Slot,
        frame_type: VarInt,
        flags: i32,
        #[meta = flags & 1 == 1]
        background_texture: Option<Cow<'c, str>>,
        x: f32,
        y: f32,
    }

    pub struct Requirement<'a, 'b> {
        count: VarInt,
        #[meta = *count as usize]
        requirements: Cow<'a, [Cow<'b, str>]>,
    }

    pub struct ProgressMapping<'a, 'b, 'c> {
        identifier: Cow<'a, str>,
        progress: AdvancementProgress<'b, 'c>,
    }

    pub struct AdvancementProgress<'a, 'b> {
        count: VarInt,
        #[meta = *count as usize]
        criteria: Cow<'a, [Criterion<'b>]>,
    }

    pub struct Criterion<'a> {
        identifier: Cow<'a, str>,
        achieved: bool,
        #[meta = achieved]
        achieved_date: Option<i64>,
    }

    pub struct EntityProperty<'a, 'b> {
        key: Cow<'a, str>,
        value: f64,
        modifier_count: VarInt,
        #[meta = *modifier_count as usize]
        modifiers: Cow<'b, [Modifier]>,
    }

    pub struct Modifier {
        uuid: u128,
        amount: f64,
        operation: u8,
    }

    pub struct Recipe<'a, 'b> {
        recipe_id: Cow<'a, str>,
        recipe_type: Cow<'b, str>,
    }

    pub struct Ingredient<'a> {
        count: VarInt,
        #[meta = *count as usize]
        items: Cow<'a, [Slot]>,
    }

    pub struct Tags<'a, 'b, 'c> {
        count: VarInt,
        #[meta = *count as usize]
        tags: Cow<'a, [Tag<'b, 'c>]>,
    }

    pub struct Tag<'a, 'b> {
        tag_name: Cow<'a, str>,
        count: VarInt,
        #[meta = *count as usize]
        entries: Cow<'b, [VarInt]>,
    }
}

impl_ser_de_enum! {
    pub enum BossBarAction<'a> {
        Add (0) {
            title: Cow<'a, str>,
            health: f32,
            colour: VarInt,
            division: VarInt,
            flags: u8,
        },
        Remove (1) {},
        UpdateHealth (2) {
            health: f32,
        },
        UpdateTitle (3) {
            title: Cow<'a, str>,
        },
        UpdateStyle (4) {
            colour: VarInt,
            division: VarInt,
        },
        UpdateFlags (5) {
            flags: u8,
        },
    }

    pub enum PlayerListAction<'a, 'b, 'c, 'd, 'e, 'f> {
        AddPlayer (0) {
            name: Cow<'a, str>,
            num_props: VarInt,
            #[meta = *num_props as usize]
            props: Cow<'b, [PlayerListProperty<'c, 'd, 'e>]>,
            gamemode: VarInt,
            ping: VarInt,
            has_display_name: bool,
            #[meta = has_display_name]
            display_name: Option<Cow<'f, str>>,
        },
        UpdateGamemode (1) {
            gamemode: VarInt,
        },
        UpdateLatency (2) {
            ping: VarInt,
        },
        UpdateDisplayName (3) {
            has_display_name: bool,
            #[meta = has_display_name]
            display_name: Option<Cow<'a, str>>,
        },
        RemovePlayer (4) {},
    }

    pub enum WorldBorderAction {
        SetSize (0) {
            diameter: f64,
        },
        LerpSize (1) {
            old_diameter: f64,
            new_diameter: f64,
            speed: VarLong,
        },
        SetCenter (2) {
            x: f64,
            z: f64,
        },
        Initialise (3) {
            x: f64,
            z: f64,
            old_diameter: f64,
            new_diameter: f64,
            speed: VarLong,
            portal_teleport_boundary: VarInt,
            warning_time: VarInt,
            warning_blocks: VarInt,
        },
        SetWarningTime (4) {
            warning_time: VarInt,
        },
        SetWarningBlocks (5) {
            warning_blocks: VarInt,
        },
    }

    pub enum EntityMetadataValue<'a, 'b, 'c> {
        Byte (0) { v: u8, },
        VarInt (1) { v: VarInt, },
        Float (2) { v: f32, },
        String (3) { v: Cow<'a, str>, },
        Chat (4) { v: Cow<'b, str>, },
        OptChat (5) {
            present: bool,
            #[meta = present]
            v: Option<Cow<'c, str>>,
        },
        Slot (6) { v: Slot, },
        Bool (7) { v: bool, },
        Rotation (8) {
            x: f32,
            y: f32,
            z: f32,
        },
        Position (9) { v: f64, },
        OptPosition (10) {
            present: bool,
            #[meta = present]
            v: Option<u64>,
        },
        Direction (11) {
            v: VarInt,
        },
        OptUuid (12) {
            present: bool,
            #[meta = present]
            v: Option<u128>,
        },
        OptBlockId (13) { v: VarInt, },
        Nbt (14) { v: nbt::Value, },
        Particle (15) { v: Particle, },
    }

    pub enum ParticleData {
        NoData (0..=2 | 4..=10 | 12..=19 | 21..=26 | 28..=49) {},
        Block (3) {
            block_state: VarInt,
        },
        Dust (11) {
            red: f32,
            green: f32,
            blue: f32,
            scale: f32,
        },
        FallingDust (20) {
            block_state: VarInt,
        },
        Item (27) {
            item: Slot,
        },
    }

    pub enum TeamsAction<'a, 'b, 'c, 'd, 'e, 'f, 'g> {
        CreateTeam (0) {
            display_name: Cow<'a, str>,
            flags: u8,
            name_tag_vis: Cow<'b, str>,
            collision_rule: Cow<'c, str>,
            formatting: VarInt,
            team_prefix: Cow<'d, str>,
            team_suffix: Cow<'e, str>,
            entity_count: VarInt,
            #[meta = *entity_count as usize]
            entities: Cow<'f, [Cow<'g, str>]>,
        },
        RemoveTeam (1) {},
        UpdateTeamInfo (2) {
            display_name: Cow<'a, str>,
            flags: u8,
            name_tag_vis: Cow<'b, str>,
            collision_rule: Cow<'c, str>,
            formatting: VarInt,
            team_prefix: Cow<'d, str>,
            team_suffix: Cow<'e, str>,
        },
        AddEntities (3) {
            count: VarInt,
            #[meta = *count as usize]
            entities: Cow<'a, [Cow<'b, str>]>,
        },
        RemoveEntities (4) {
            count: VarInt,
            #[meta = *count as usize]
            entities: Cow<'a, [Cow<'b, str>]>,
        },
    }

    pub enum TitleAction<'a> {
        SetTitle (0) {
            text: Cow<'a, str>,
        },
        SetSubtitle (1) {
            text: Cow<'a, str>,
        },
        SetActionBar (2) {
            text: Cow<'a, str>,
        },
        SetTimes (3) {
            fade_in_len: i32,
            stay_for: i32,
            fade_out_len: i32,
        },
        Hide (4) {},
        Reset (5) {},
    }

    pub enum RecipeData<'a, 'b, 'c> {
        // The numbers here are internal to this library
        CraftingShapeless (0) {
            group: Cow<'a, str>,
            ingredient_count: VarInt,
            #[meta = *ingredient_count as usize]
            ingredients: Cow<'a, [Ingredient<'b>]>,
        },
        CraftingShaped (1) {
            width: VarInt,
            height: VarInt,
            group: Cow<'a, str>,
            #[meta = (*width * *height) as usize]
            ingredients: Cow<'b, [Ingredient<'c>]>,
            result: Slot,
        },
        Smelting (2) {
            group: Cow<'a, str>,
            ingredient: Ingredient<'b>,
            result: Slot,
            xp: f32,
            cooking_time: VarInt,
        },
    }

    pub enum RecipeBookUpdate<'a> {
        DisplayedRecipe (0) {
            recipe_id: Cow<'a, str>,
        },
        RecipeBookStates (1) {
            crafting_book_open: bool,
            crafting_filter_active: bool,
            smelting_book_open: bool,
            smelting_filter_active: bool,
        },
    }
}

#[derive(Clone)]
pub struct PlayerListEntry<'a, 'b, 'c, 'd, 'e, 'f> {
    uuid: u128,
    action: PlayerListAction<'a, 'b, 'c, 'd, 'e, 'f>,
}

impl Serialise for PlayerListEntry<'_, '_, '_, '_, '_, '_> {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.uuid.serialise(&mut writer)?;
        self.action.serialise(&mut writer)?;
        Ok(())
    }
}

impl Deserialise<i32> for PlayerListEntry<'_, '_, '_, '_, '_, '_> {
    fn deserialise<R: Read>(mut reader: R, meta: i32) -> Result<Self, ParseError> {
        let uuid = u128::deserialise(&mut reader, ())?;
        let action = PlayerListAction::deserialise(&mut reader, meta)?;
        Ok(PlayerListEntry { uuid, action })
    }
}
