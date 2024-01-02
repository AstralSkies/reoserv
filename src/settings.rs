use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub max_connections: i32,
    pub password_salt: String,
    pub ping_rate: i32,
    pub enforce_sequence: bool,
    pub num_of_maps: i32,
    pub min_version: String,
    pub max_version: String,
    pub save_rate: i32,
    pub generate_pub: bool,
    pub lang: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub host: String,
    pub port: String,
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCharacter {
    pub spawn_map: i32,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_direction: i32,
    pub home: String,
}

#[derive(Debug, Deserialize)]
pub struct Character {
    pub max_hair_color: i32,
    pub max_hair_style: i32,
}

#[derive(Debug, Deserialize)]
pub struct Jail {
    pub map: i32,
    pub x: i32,
    pub y: i32,
    pub free_map: i32,
    pub free_x: i32,
    pub free_y: i32,
}

#[derive(Debug, Deserialize)]
pub struct Rescue {
    pub map: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize)]
pub struct World {
    pub see_distance: i32,
    pub drop_distance: i32,
    pub recover_rate: i32,
    pub npc_recover_rate: i32,
    pub chest_spawn_rate: i32,
    pub exp_multiplier: i32,
    pub stat_points_per_level: i32,
    pub skill_points_per_level: i32,
    pub tick_rate: i32,
    pub arena_spawn_rate: i32,
    pub door_close_rate: i32,
    pub quake_rate: i32,
    pub spike_rate: i32,
    pub spike_damage: f32,
    pub drain_rate: i32,
    pub drain_hp_damage: f32,
    pub drain_tp_damage: f32,
    pub warp_suck_rate: i32,
}

#[derive(Debug, Deserialize)]
pub struct NPCs {
    pub instant_spawn: bool,
    pub respawn_rate: i32,
    pub freeze_on_empty_map: bool,
    pub freeze_out_of_range: bool,
    pub chase_distance: i32,
    pub bored_timer: i32,
    pub act_rate: i32,
    pub speed_0: i32,
    pub speed_1: i32,
    pub speed_2: i32,
    pub speed_3: i32,
    pub speed_4: i32,
    pub speed_5: i32,
    pub speed_6: i32,
    pub talk_rate: i32,
}

#[derive(Debug, Deserialize)]
pub struct Sln {
    pub enabled: bool,
    pub url: String,
    pub site: String,
    pub hostname: String,
    pub server_name: String,
    pub rate: i32,
    pub zone: String,
}

#[derive(Debug, Deserialize)]
pub struct Bank {
    pub max_item_amount: i32,
    pub base_size: i32,
    pub size_step: i32,
    pub max_upgrades: i32,
    pub upgrade_base_cost: i32,
    pub upgrade_cost_step: i32,
}

#[derive(Debug, Deserialize)]
pub struct Limits {
    pub max_bank_gold: i32,
    pub max_item: i32,
    pub max_trade: i32,
    pub max_chest: i32,
    pub max_party_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct Combat {
    pub weapon_ranges: Vec<WeaponRange>,
}

#[derive(Debug, Deserialize)]
pub struct WeaponRange {
    pub weapon: i32,
    pub range: i32,
    pub arrows: bool,
}

#[derive(Debug, Deserialize)]
pub struct Board {
    pub max_posts: i32,
    pub max_user_posts: i32,
    pub max_recent_posts: i32,
    pub recent_post_time: i32,
    pub max_subject_length: i32,
    pub max_post_length: i32,
    pub date_posts: bool,
    pub admin_board: i32,
    pub admin_max_posts: i32,
}

#[derive(Debug, Deserialize)]
pub struct Chest {
    pub slots: i32,
}

#[derive(Debug, Deserialize)]
pub struct Quake {
    pub min_ticks: i32,
    pub max_ticks: i32,
    pub min_strength: i32,
    pub max_strength: i32,
}

#[derive(Debug, Deserialize)]
pub struct Map {
    pub quakes: Vec<Quake>,
    pub door_close_rate: i32,
}

#[derive(Debug, Deserialize)]
pub struct Jukebox {
    pub cost: i32,
    pub max_track_id: i32,
    pub track_timer: i32,
}

#[derive(Debug, Deserialize)]
pub struct Barber {
    pub cost_per_level: i32,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub new_character: NewCharacter,
    pub jail: Jail,
    pub rescue: Rescue,
    pub world: World,
    pub character: Character,
    pub npcs: NPCs,
    pub sln: Sln,
    pub bank: Bank,
    pub limits: Limits,
    pub board: Board,
    pub chest: Chest,
    pub map: Map,
    pub combat: Combat,
    pub jukebox: Jukebox,
    pub barber: Barber,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("Config.toml"))
            .add_source(File::with_name("Config.local.toml").required(false))
            .build()?;

        s.try_deserialize()
    }
}
