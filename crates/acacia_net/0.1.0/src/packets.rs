use std::{
    borrow::Cow,
    io::{Cursor, Error, Write},
};

use super::{
    deserialise::{Deserialise, ParseError},
    serialise::Serialise,
    types::*,
};

pub trait Packet: Sized {
    fn write_to(&self, out_buf: &mut Vec<u8>) -> Result<(), Error>;
    fn read_from(from: &[u8]) -> Result<Self, ParseError>;
}

macro_rules! impl_packets {
    (
        $(
        //TODO: use ? instead of * for repetition when my toolchains stop having identity crises
        pub enum $name:ident$(<$($life:lifetime),+>)* {
            $(
            $packet_name:ident ($packet_id:expr) {
                $(
                $(#[meta = $meta:expr])*
                $field_name:ident: $field_type:ty,
                )*
            },
            )*
        }
        )*
    ) => {
        $(
        pub enum $name$(<$($life),*>)* {
            $(
            $packet_name {
                $(
                $field_name: $field_type,
                )*
            },
            )*
        }

        impl$(<$($life),*>)* Packet for $name$(<$($life),*>)* {
            fn write_to(&self, out_buf: &mut Vec<u8>) -> Result<(), Error> {
                out_buf.clear();
                match self {
                    $(
                    $name::$packet_name { $($field_name),* } => {
                        let mut out_buf = Cursor::new(out_buf);
                        let mut temp_buf = Cursor::new(Vec::new());

                        VarInt($packet_id).serialise(&mut temp_buf)?;
                        $(
                        $field_name.serialise(&mut temp_buf)?;
                        )*

                        VarInt(temp_buf.get_ref().len() as i32).serialise(&mut out_buf)?;
                        out_buf.write_all(&temp_buf.get_ref())?;
                        Ok(())
                    },
                    )*
                }
            }

            fn read_from(from: &[u8]) -> Result<Self, ParseError> {
                #![allow(unused_parens)]
                let mut reader = Cursor::new(from);
                let packet_len = VarInt::deserialise(&mut reader, ())?.0 as usize;
                let needed = packet_len - (reader.get_ref().len() - reader.position() as usize);
                if needed != 0 { return Err(ParseError::MoreNeeded(needed)); }

                let VarInt(packet_id) = VarInt::deserialise(&mut reader, ())?;
                match packet_id {
                    $(
                    $packet_id => {
                        $(
                        let $field_name = <$field_type>::deserialise(&mut reader, ($($meta),*))?;
                        )*
                        Ok($name::$packet_name { $($field_name),* })
                    },
                    )*
                    _ => return Err(ParseError::InvalidPacketId(packet_id)),
                }
            }
        }
        )*
    }
}

impl_packets! {

    // Begin clientbound

    pub enum CbStatusMessage<'a> {
        Response (0x00) {
            server_info: Cow<'a, str>,
        },
        Pong (0x01) {
            payload: u64,
        },
    }

    pub enum CbLoginMessage<'a, 'b> {
        Disconnect (0x00) {
            reason: Cow<'a, str>,
        },
        EncryptionRequest (0x01) {
            _server_id: Cow<'a, str>,
            pub_key_len: VarInt,
            #[meta = *pub_key_len as usize]
            pub_key: Cow<'a, [u8]>,
            verify_token_len: VarInt,
            #[meta = *verify_token_len as usize]
            verify_token: Cow<'a, [u8]>,
        },
        LoginSuccess (0x02) {
            uuid: Cow<'a, str>,
            username: Cow<'b, str>,
        },
        SetCompression (0x03) {
            threshold: VarInt,
        },
        LoginPluginRequest (0x04) {
            message_id: VarInt,
            channel: Cow<'a, str>,
            data: Cow<'b, [u8]>,
        },
    }

    pub enum CbPlayMessage<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j, 'k, 'l, 'm, 'n, 'o, 'p, 'q> {
        SpawnObject (0x00) {
            entity_id: VarInt,
            object_uuid: u128,
            object_type: u8,
            x: f64,
            y: f64,
            z: f64,
            pitch: u8,
            yaw: u8,
            vel_x: i16,
            vel_y: i16,
            vel_z: i16,
        },
        SpawnExperienceOrb (0x01) {
            entity_id: VarInt,
            x: f64,
            y: f64,
            z: f64,
            xp_count: i16,
        },
        SpawnGlobalEntity (0x02) {
            entity_id: VarInt,
            entity_type: u8,
            x: f64,
            y: f64,
            z: f64,
        },
        SpawnMob (0x03) {
            entity_id: VarInt,
            entity_uuid: u128,
            mob_type: VarInt,
            x: f64,
            y: f64,
            z: f64,
            yaw: u8,
            pitch: u8,
            head_pitch: u8,
            vel_x: i16,
            vel_y: i16,
            vel_z: i16,
            entity_metadata: Cow<'a, [u8]>,
        },
        SpawnPainting (0x04) {
            entity_id: VarInt,
            entity_uuid: u128,
            motif: VarInt,
        },
        SpawnPlayer (0x05) {
            entity_id: VarInt,
            player_uuid: u128,
            x: f64,
            y: f64,
            z: f64,
            yaw: u8,
            pitch: u8,
            entity_metadata: Cow<'a, [u8]>,
        },
        Animation (0x06) {
            entity_id: VarInt,
            animation: u8,
        },
        Statistics (0x07) {
            stats_len: VarInt,
            #[meta = *stats_len as usize]
            stats: Cow<'a, [Statistic]>,
        },
        BlockBreakAnimation (0x08) {
            entity_id: VarInt,
            position: Position,
            stage: u8,
        },
        UpdateBlockEntity (0x09) {
            position: Position,
            action: u8,
            nbt_data: Cow<'a, [u8]>,
        },
        BlockAction (0x0A) {
            position: Position,
            action_id: u8,
            action_param: u8,
            block_type: VarInt,
        },
        BlockChange (0x0B) {
            position: Position,
            block_id: VarInt,
        },
        BossBar (0x0C) {
            bar_uuid: u128,
            action_type: VarInt,
            #[meta = *action_type]
            action: BossBarAction<'a>,
        },
        ServerDifficulty (0x0D) {
            difficulty: u8,
        },
        ChatMessage (0x0E) {
            message_json: Cow<'a, str>,
            position: u8,
        },
        MultiBlockChange (0x0F) {
            chunk_x: i32,
            chunk_z: i32,
            record_count: VarInt,
            #[meta = *record_count as usize]
            records: Cow<'a, [MultiBlockChangeRecord]>,
        },
        TabComplete (0x10) {
            start: VarInt,
            length: VarInt,
            count: VarInt,
            #[meta = *count as usize]
            matches: Cow<'a, [TabCompleteMatch<'b, 'c>]>,
        },
        DeclareCommands (0x11) {
            count: VarInt,
            rest_of_the_packet: Cow<'a, [u8]>,    // for another day
        },
        ConfirmTransaction (0x12) {
            window_id: u8,
            action_number: i16,
            accepted: bool,
        },
        CloseWindow (0x13) {
            window_id: u8,
        },
        OpenWindow (0x14) {
            window_id: u8,
            window_type: Cow<'a, str>,
            window_title: Cow<'b, str>,
            num_slots: u8,
            #[meta = window_type == "EntityHorse"]
            entity_id: Option<i32>,
        },
        WindowItems (0x15) {
            window_id: u8,
            count: i16,
            #[meta = count as usize]
            slots: Cow<'a, [Slot]>,
        },
        WindowProperty (0x16) {
            window_id: u8,
            property: i16,
            value: i16,
        },
        SetSlot (0x17) {
            window_id: u8,
            slot: i16,
            slot_data: Slot,
        },
        SetCooldown (0x18) {
            item_id: VarInt,
            cooldown_ticks: VarInt,
        },
        PluginMessage (0x19) {
            channel: Cow<'a, str>,
            data: Cow<'b, [u8]>,
        },
        NamedSoundEffect (0x1A) {
            sound_name: Cow<'a, str>,
            sound_category: VarInt,
            effect_x: i32,
            effect_y: i32,
            effect_z: i32,
            volume: f32,
            pitch: f32,
        },
        Disconnect (0x1B) {
            reason: Cow<'a, str>,
        },
        EntityStatus (0x1C) {
            entity_id: i32,
            entity_status: u8,
        },
        NbtQueryResponse (0x1D) {
            transaction_id: VarInt,
            nbt_data: Cow<'a, [u8]>,
        },
        Explosion (0x1E) {
            x: f32,
            y: f32,
            z: f32,
            radius: f32,
            record_count: i32,
            #[meta = record_count as usize]
            affected_blocks: Cow<'a, [ExplosionBlockOffset]>,
            player_vel_x: f32,
            player_vel_y: f32,
            player_vel_z: f32,
        },
        UnloadChunk (0x1F) {
            x: i32,
            z: i32,
        },
        ChangeGameState (0x20) {
            reason: u8,
            value: f32,
        },
        KeepAlive (0x21) {
            payload: u64,
        },
        ChunkData (0x22) {
            chunk_x: i32,
            chunk_z: i32,
            ground_up_continuous: bool,
            primary_bit_mask: VarInt,
            data_len: VarInt,
            #[meta = *data_len as usize]
            chunk_data: Cow<'a, [u8]>,
            num_block_entities: VarInt,
            block_entities_nbt: Cow<'b, [u8]>,
        },
        Effect (0x23) {
            effect_id: i32,
            position: Position,
            data: i32,
            disable_relative_volume: bool,
        },
        Particle (0x24) {
            particle_id: i32,
            long_distance: bool,
            x: f32,
            y: f32,
            z: f32,
            offset_x: f32,
            offset_y: f32,
            offset_z: f32,
            particle_data: f32,
            particle_count: i32,
            data: Cow<'a, [u8]>,
        },
        JoinGame (0x25) {
            entity_id: i32,
            gamemode: u8,
            dimension: i32,
            difficulty: u8,
            max_players: u8,
            level_type: Cow<'a, str>,
            reduced_debug_info: bool,
        },
        Map (0x26) {
            map_id: VarInt,
            scale: u8,
            tracking_position: bool,
            icon_count: VarInt,
            #[meta = *icon_count as usize]
            icons: Cow<'a, [Icon<'b>]>,
            columns: u8,
            #[meta = columns > 0]
            rows: Option<u8>,
            #[meta = columns > 0]
            x: Option<u8>,
            #[meta = columns > 0]
            z: Option<u8>,
            #[meta = columns > 0]
            length: Option<VarInt>,
            #[meta = (columns > 0, *length.unwrap() as usize)]
            data: Option<Cow<'a, [u8]>>,
        },
        Entity (0x27) {
            entity_id: VarInt,
        },
        EntityRelativeMove (0x28) {
            entity_id: VarInt,
            delta_x: i16,
            delta_y: i16,
            delta_z: i16,
            on_ground: bool,
        },
        EntityLookAndRelativeMove (0x29) {
            entity_id: VarInt,
            delta_x: i16,
            delta_y: i16,
            delta_z: i16,
            yaw: u8,
            pitch: u8,
            on_ground: bool,
        },
        EntityLook (0x2A) {
            entity_id: VarInt,
            yaw: u8,
            pitch: u8,
            on_ground: bool,
        },
        VehicleMove (0x2B) {
            x: f64,
            y: f64,
            z: f64,
            yaw_degrees: f32,
            pitch_degrees: f32,
        },
        OpenSignEditor (0x2C) {
            position: Position,
        },
        CraftRecipeResponse (0x2D) {
            window_id: u8,
            recipe: Cow<'a, str>,
        },
        PlayerAbilities (0x2E) {
            flags: u8,
            flying_speed: f32,
            fov_modifier: f32,
        },
        CombatEvent (0x2F) {
            event: VarInt,
            #[meta = *event == 1]
            end_combat: Option<EndCombat>,
            #[meta = *event == 2]
            entity_dead: Option<EntityDead<'a>>,
        },
        PlayerListItem (0x30) {
            action: VarInt,
            num_players: VarInt,
            #[meta = (*num_players as usize, *action)]
            players: Cow<'a, [PlayerListEntry<'b, 'c, 'd, 'e, 'f, 'g>]>,
        },
        FacePlayer (0x31) {
            feet_or_eyes: VarInt,
            target_x: f64,
            target_y: f64,
            target_z: f64,
            is_entity: bool,
            #[meta = is_entity]
            entity_id: Option<VarInt>,
            #[meta = is_entity]
            entity_feet_or_eyes: Option<VarInt>,
        },
        PlayerPositionAndLook (0x32) {
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            pitch: f32,
            flags: u8,
            teleport_id: VarInt,
        },
        UseBed (0x33) {
            entity_id: VarInt,
            position: Position,
        },
        UnlockRecipes (0x34) {
            action: VarInt,
            crafting_recipe_book_open: bool,
            crafting_recipe_book_filter_active: bool,
            smelting_recipe_book_open: bool,
            smelting_recipe_book_filter_active: bool,
            ids_1_len: VarInt,
            ids_1: Cow<'a, [Cow<'b, str>]>,
            #[meta = *action == 0]
            ids_2_len: Option<VarInt>,
            #[meta = (*action == 0, *ids_2_len.unwrap() as usize)]
            ids_2: Option<Cow<'c, [Cow<'d, str>]>>,
        },
        DestroyEntities (0x35) {
            count: VarInt,
            #[meta = *count as usize]
            entity_ids: Cow<'a, [VarInt]>,
        },
        RemoveEntityEffect (0x36) {
            entity_id: VarInt,
            effect_id: u8,
        },
        ResourcePackSend (0x37) {
            url: Cow<'a, str>,
            hash: Cow<'b, str>,
        },
        Respawn (0x38) {
            dimension: i32,
            difficulty: u8,
            gamemode: u8,
            level_type: Cow<'a, str>,
        },
        EntityHeadLook (0x39) {
            entity_id: VarInt,
            head_yaw: u8,
        },
        SelectAdvancementTab (0x3A) {
            has_id: bool,
            #[meta = has_id]
            id: Option<Cow<'a, str>>,
        },
        WorldBorder (0x3B) {
            action_id: VarInt,
            #[meta = *action_id]
            action: WorldBorderAction,
        },
        Camera (0x3C) {
            entity_id: VarInt,
        },
        HeldItemChange (0x3D) {
            slot: u8,
        },
        DisplayScoreboard (0x3E) {
            position: u8,
            score_name: Cow<'a, str>,
        },
        EntityMetadata (0x3F) {
            entity_id: VarInt,
            metadata: Cow<'a, [EntityMetadata<'b, 'c, 'd>]>,
        },
        AttachEntity (0x40) {
            attached_entity_id: VarInt,
            holding_entity_id: VarInt,
        },
        EntityVelocity (0x41) {
            entity_id: VarInt,
            x: u16,
            y: u16,
            z: u16,
        },
        EntityEquipment (0x42) {
            entity_id: VarInt,
            in_slot: VarInt,
            item: Slot,
        },
        SetExperience (0x43) {
            xp_bar: f32,
            level: VarInt,
            total: VarInt,
        },
        UpdateHealth (0x44) {
            health: f32,
            food: VarInt,
            saturation: f32,
        },
        ScoreboardObjective (0x45) {
            objective_name: Cow<'a, str>,
            action: u8,
            #[meta = action == 0 || action == 2]
            objective_value: Option<Cow<'b, str>>,
            #[meta = action == 0 || action == 2]
            objective_type: Option<VarInt>,
        },
        SetPassengers (0x46) {
            entity_id: VarInt,
            count: VarInt,
            #[meta = *count as usize]
            passengers: Cow<'a, [VarInt]>,
        },
        Teams (0x47) {
            team_name: Cow<'a, str>,
            action_type: u8,
            #[meta = action_type as i32]
            action: TeamsAction<'b, 'c, 'd, 'e, 'f, 'g, 'h>,
        },
        UpdateScore (0x48) {
            entity: Cow<'a, str>,
            action: u8,
            objective_name: Cow<'b, str>,
            #[meta = action == 0]
            value: Option<VarInt>,
        },
        SpawnPosition (0x49) {
            position: Position,
        },
        TimeUpdate (0x4A) {
            world_age: i64,
            time: i64,
        },
        Title (0x4B) {
            action_type: VarInt,
            #[meta = *action_type]
            action: TitleAction<'a>,
        },
        StopSound (0x4C) {
            flags: u8,
            #[meta = flags & 1 == 1]
            source: Option<VarInt>,
            #[meta = flags & 2 == 2]
            sound: Option<Cow<'a, str>>,
        },
        SoundEffect (0x4D) {
            sound_id: VarInt,
            sound_category: VarInt,
            effect_x: i32,
            effect_y: i32,
            effect_z: i32,
            volume: f32,
            pitch: f32,
        },
        PlayerListHeaderAndFooter (0x4E) {
            header: Cow<'a, str>,
            footer: Cow<'b, str>,
        },
        CollectItem (0x4F) {
            collected_entity_id: VarInt,
            collector_entity_id: VarInt,
            item_count: VarInt,
        },
        EntityTeleport (0x50) {
            entity_id: VarInt,
            x: f64,
            y: f64,
            z: f64,
            yaw: u8,
            pitch: u8,
            on_ground: bool,
        },
        Advancements (0x51) {
            reset: bool,
            mapping_len: VarInt,
            #[meta = *mapping_len as usize]
            mappings: Cow<'a, [AdvancementMapping<'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j, 'k>]>,
            ident_count: VarInt,
            #[meta = *ident_count as usize]
            identifiers: Cow<'l, [Cow<'m, str>]>,
            progress_count: VarInt,
            #[meta = *progress_count as usize]
            progress_mappings: Cow<'n, [ProgressMapping<'o, 'p, 'q>]>,
        },
        EntityProperties (0x52) {
            entity_id: VarInt,
            property_count: i32,
            #[meta = property_count as usize]
            properties: Cow<'a, [EntityProperty<'b, 'c>]>,
        },
        EntityEffect (0x53) {
            entity_id: VarInt,
            effect_id: u8,
            amplifier: u8,
            duration_secs: VarInt,
            flags: u8,
        },
        DeclareRecipes (0x54) {
            recipe_count: VarInt,
            #[meta = *recipe_count as usize]
            recipes: Cow<'a, [Recipe<'b, 'c>]>,
        },
        Tags (0x55) {
            block_tags: Tags<'a, 'b, 'c>,
            item_tags: Tags<'d, 'e, 'f>,
            fluid_tags: Tags<'h, 'i, 'j>,
        },
    }

    // Begin serverbound

    pub enum SbHandshakeMessage<'a> {
        Handshake (0x00) {
            proto_ver: VarInt,
            server_address: Cow<'a, str>,
            server_port: u16,
            next_state: VarInt,
        },
    }

    pub enum SbStatusMessage {
        Request (0x00) {},
        Ping (0x01) {
            payload: u64,
        },
    }

    pub enum SbLoginMessage<'a, 'b> {
        LoginStart (0x00) {
            username: Cow<'a, str>,
        },
        EncryptionResponse (0x01) {
            shared_secret_len: VarInt,
            #[meta = shared_secret_len.0 as usize]
            shared_secret: Cow<'a, [u8]>,
            verify_token_len: VarInt,
            #[meta = verify_token_len.0 as usize]
            verify_token: Cow<'b, [u8]>,
        },
        LoginPluginResponse (0x02) {
            message_id: VarInt,
            successful: bool,
            data: Cow<'a, [u8]>,
        },
    }

    pub enum SbPlayMessage<'a, 'b, 'c, 'd> {
        TeleportConfirm (0x00) {
            teleport_id: VarInt,
        },
        QueryBlockNbt (0x01) {
            transaction_id: VarInt,
            position: Position,
        },
        ChatMessage (0x02) {
            message: Cow<'a, str>,
        },
        ClientStatus (0x03) {
            action_id: VarInt,
        },
        ClientSettings (0x04) {
            locale: Cow<'a, str>,
            view_distance: u8,
            chat_mode: VarInt,
            chat_colours: bool,
            displayed_skin_parts: u8,
            main_hand: VarInt,
        },
        TabComplete (0x05) {
            transaction_id: VarInt,
            text: Cow<'a, str>,
        },
        ConfirmTransaction (0x06) {
            window_id: u8,
            action_number: i16,
            accepted: bool,
        },
        EnchantItem (0x07) {
            window_id: u8,
            enchantment: u8,
        },
        ClickWindow (0x08) {
            window_id: u8,
            slot: i16,
            button: u8,
            action_number: i16,
            mode: VarInt,
            clicked_item: Slot,
        },
        CloseWindow (0x09) {
            window_id: u8,
        },
        PluginMessage (0x0A) {
            channel: Cow<'a, str>,
            data: Cow<'b, [u8]>,
        },
        EditBook (0x0B) {
            new_book: Slot,
            is_signing: bool,
            hand: VarInt,
        },
        QueryEntityNbt (0x0C) {
            transaction_id: VarInt,
            entity_id: VarInt,
        },
        UseEntity (0x0D) {
            target: VarInt,
            action: VarInt,
            #[meta = *action == 2]
            target_x: Option<f32>,
            #[meta = *action == 2]
            target_y: Option<f32>,
            #[meta = *action == 2]
            target_z: Option<f32>,
            #[meta = *action == 0 || *action == 2]
            hand: Option<VarInt>,
        },
        KeepAlive (0x0E) {
            payload: i64,
        },
        Player (0x0F) {
            on_ground: bool,
        },
        PlayerPosition (0x10) {
            x: f64,
            y: f64,
            z: f64,
            on_ground: bool,
        },
        PlayerPositionAndLook (0x11) {
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            pitch: f32,
            on_ground: bool,
        },
        PlayerLook (0x12) {
            yaw: f32,
            pitch: f32,
            on_ground: bool,
        },
        VehicleMove (0x13) {
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            pitch: f32,
        },
        SteerBoat (0x14) {
            left_paddle: bool,
            right_paddle: bool,
        },
        PickItem (0x15) {
            from_slot: VarInt,
        },
        CraftRecipeRequest (0x16) {
            window_id: u8,
            recipe: Cow<'a, str>,
            make_all: bool,
        },
        PlayerAbilities (0x17) {
            flags: u8,
            flying_speed: f32,
            walking_speed: f32,
        },
        PlayerDigging (0x18) {
            status: VarInt,
            position: Position,
            face: u8,
        },
        EntityAction (0x19) {
            entity_id: VarInt,
            action_id: VarInt,
            jump_boost: VarInt,
        },
        SteerVehicle (0x1A) {
            left: f32,
            forward: f32,
            flags: u8,
        },
        RecipeBookData (0x1B) {
            update_type: VarInt,
            #[meta = *update_type]
            update: RecipeBookUpdate<'a>,
        },
        NameItem (0x1C) {
            name: Cow<'a, str>,
        },
        ResourcePackStatus (0x1D) {
            result: VarInt,
        },
        AdvancementTab (0x1E) {
            action: VarInt,
            #[meta = *action == 0]
            tab_id: Option<Cow<'a, str>>,
        },
        SelectTrade (0x1F) {
            selected_slot: VarInt,
        },
        SetBeaconEffect (0x20) {
            primary: VarInt,
            secondary: VarInt,
        },
        HeldItemChange (0x21) {
            slot: i16,
        },
        UpdateCommandBlock (0x22) {
            position: Position,
            command: Cow<'a, str>,
            mode: VarInt,
            flags: u8,
        },
        UpdateCommandBlockMinecart (0x23) {
            entity_id: VarInt,
            command: Cow<'a, str>,
            track_output: bool,
        },
        CreativeInventoryAction (0x24) {
            slot: i16,
            clicked_item: Slot,
        },
        UpdateStructureBlock (0x25) {
            position: Position,
            action: VarInt,
            mode: VarInt,
            name: Cow<'a, str>,
            offset_x: u8,
            offset_y: u8,
            offset_z: u8,
            size_x: u8,
            size_y: u8,
            size_z: u8,
            mirror: VarInt,
            rotation: VarInt,
            metadata: Cow<'b, str>,
            integrity: f32,
            seed: VarLong,
            flags: u8,
        },
        UpdateSign (0x26) {
            position: Position,
            line_1: Cow<'a, str>,
            line_2: Cow<'b, str>,
            line_3: Cow<'c, str>,
            line_4: Cow<'d, str>,
        },
        HandAnimation (0x27) {
            hand: VarInt,
        },
        Spectate (0x28) {
            player: u128,
        },
        PlayerBlockPlacement (0x29) {
            position: Position,
            face: VarInt,
            hand: VarInt,
            cursor_x: f32,
            cursor_y: f32,
            cursor_z: f32,
        },
        UseItem (0x2A) {
            hand: VarInt,
        },
    }
}
