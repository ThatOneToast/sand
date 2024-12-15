#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#ifdef _MSC_VER
#pragma optimize("", off)
#elif defined(__clang__)
#pragma clang optimize off
#elif defined(__GNUC__)
#pragma GCC optimize ("O0")
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 267
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 227
#define ALIAS_COUNT 0
#define TOKEN_COUNT 155
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 41
#define MAX_ALIAS_SEQUENCE_LENGTH 7
#define PRODUCTION_ID_COUNT 45

enum ts_symbol_identifiers {
  anon_sym_fn = 1,
  anon_sym_LBRACE = 2,
  anon_sym_SEMI = 3,
  aux_sym_block_token1 = 4,
  anon_sym_RBRACE = 5,
  anon_sym_execute = 6,
  anon_sym_as = 7,
  anon_sym_at = 8,
  anon_sym_align = 9,
  anon_sym_anchored = 10,
  anon_sym_eyes = 11,
  anon_sym_feet = 12,
  anon_sym_facing = 13,
  anon_sym_entity = 14,
  anon_sym_in = 15,
  anon_sym_positioned = 16,
  anon_sym_rotated = 17,
  anon_sym_if = 18,
  anon_sym_unless = 19,
  anon_sym_store = 20,
  anon_sym_result = 21,
  anon_sym_success = 22,
  anon_sym_byte = 23,
  anon_sym_short = 24,
  anon_sym_int = 25,
  anon_sym_long = 26,
  anon_sym_float = 27,
  anon_sym_double = 28,
  anon_sym_run = 29,
  anon_sym_TILDE = 30,
  anon_sym_DASH = 31,
  anon_sym_CARET = 32,
  sym_align_axes = 33,
  anon_sym_minecraft_COLONoverworld = 34,
  anon_sym_minecraft_COLONthe_nether = 35,
  anon_sym_minecraft_COLONthe_end = 36,
  anon_sym_block = 37,
  anon_sym_minecraft_COLON = 38,
  anon_sym_LBRACK = 39,
  anon_sym_COMMA = 40,
  anon_sym_RBRACK = 41,
  anon_sym_blocks = 42,
  anon_sym_masked = 43,
  anon_sym_data = 44,
  anon_sym_storage = 45,
  anon_sym_predicate = 46,
  anon_sym_score = 47,
  anon_sym_matches = 48,
  anon_sym_LT = 49,
  anon_sym_LT_EQ = 50,
  anon_sym_EQ = 51,
  anon_sym_GT_EQ = 52,
  anon_sym_GT = 53,
  anon_sym_bossbar = 54,
  anon_sym_value = 55,
  anon_sym_max = 56,
  sym_nbt_path = 57,
  anon_sym_ATs = 58,
  anon_sym_ATa = 59,
  anon_sym_ATp = 60,
  anon_sym_ATr = 61,
  anon_sym_ATe = 62,
  anon_sym_levels = 63,
  anon_sym_points = 64,
  anon_sym_DOT_DOT = 65,
  anon_sym_xpadd = 66,
  anon_sym_xpset = 67,
  anon_sym_xpquery = 68,
  aux_sym_quoted_string_token1 = 69,
  anon_sym_say = 70,
  anon_sym_clear = 71,
  anon_sym_eclear = 72,
  anon_sym_tellraw = 73,
  anon_sym_effect = 74,
  anon_sym_give = 75,
  anon_sym_enchant = 76,
  anon_sym_minecraft_COLONspeed = 77,
  anon_sym_minecraft_COLONslowness = 78,
  anon_sym_minecraft_COLONhaste = 79,
  anon_sym_minecraft_COLONmining_fatigue = 80,
  anon_sym_minecraft_COLONstrength = 81,
  anon_sym_minecraft_COLONinstant_health = 82,
  anon_sym_minecraft_COLONinstant_damage = 83,
  anon_sym_minecraft_COLONjump_boost = 84,
  anon_sym_minecraft_COLONnausea = 85,
  anon_sym_minecraft_COLONregeneration = 86,
  anon_sym_minecraft_COLONresistance = 87,
  anon_sym_minecraft_COLONfire_resistance = 88,
  anon_sym_minecraft_COLONwater_breathing = 89,
  anon_sym_minecraft_COLONinvisibility = 90,
  anon_sym_minecraft_COLONblindness = 91,
  anon_sym_minecraft_COLONnight_vision = 92,
  anon_sym_minecraft_COLONhunger = 93,
  anon_sym_minecraft_COLONweakness = 94,
  anon_sym_minecraft_COLONpoison = 95,
  anon_sym_minecraft_COLONwither = 96,
  anon_sym_minecraft_COLONhealth_boost = 97,
  anon_sym_minecraft_COLONabsorption = 98,
  anon_sym_minecraft_COLONsaturation = 99,
  anon_sym_minecraft_COLONglowing = 100,
  anon_sym_minecraft_COLONlevitation = 101,
  anon_sym_minecraft_COLONluck = 102,
  anon_sym_minecraft_COLONunluck = 103,
  anon_sym_minecraft_COLONslow_falling = 104,
  anon_sym_minecraft_COLONconduit_power = 105,
  anon_sym_minecraft_COLONdolphins_grace = 106,
  anon_sym_minecraft_COLONbad_omen = 107,
  anon_sym_minecraft_COLONhero_of_the_village = 108,
  anon_sym_minecraft_COLONprotection = 109,
  anon_sym_minecraft_COLONfire_protection = 110,
  anon_sym_minecraft_COLONfeather_falling = 111,
  anon_sym_minecraft_COLONblast_protection = 112,
  anon_sym_minecraft_COLONprojectile_protection = 113,
  anon_sym_minecraft_COLONrespiration = 114,
  anon_sym_minecraft_COLONaqua_affinity = 115,
  anon_sym_minecraft_COLONthorns = 116,
  anon_sym_minecraft_COLONdepth_strider = 117,
  anon_sym_minecraft_COLONfrost_walker = 118,
  anon_sym_minecraft_COLONbinding_curse = 119,
  anon_sym_minecraft_COLONsharpness = 120,
  anon_sym_minecraft_COLONsmite = 121,
  anon_sym_minecraft_COLONbane_of_arthropods = 122,
  anon_sym_minecraft_COLONknockback = 123,
  anon_sym_minecraft_COLONfire_aspect = 124,
  anon_sym_minecraft_COLONlooting = 125,
  anon_sym_minecraft_COLONsweeping = 126,
  anon_sym_minecraft_COLONefficiency = 127,
  anon_sym_minecraft_COLONsilk_touch = 128,
  anon_sym_minecraft_COLONunbreaking = 129,
  anon_sym_minecraft_COLONfortune = 130,
  anon_sym_minecraft_COLONpower = 131,
  anon_sym_minecraft_COLONpunch = 132,
  anon_sym_minecraft_COLONflame = 133,
  anon_sym_minecraft_COLONinfinity = 134,
  anon_sym_minecraft_COLONluck_of_the_sea = 135,
  anon_sym_minecraft_COLONlure = 136,
  anon_sym_minecraft_COLONmending = 137,
  anon_sym_minecraft_COLONvanishing_curse = 138,
  anon_sym_minecraft_COLONsoul_speed = 139,
  anon_sym_minecraft_COLONswift_sneak = 140,
  sym_text = 141,
  anon_sym_time = 142,
  anon_sym_set = 143,
  anon_sym_query = 144,
  anon_sym_gms = 145,
  anon_sym_gma = 146,
  anon_sym_gmsp = 147,
  anon_sym_gmc = 148,
  anon_sym_day = 149,
  anon_sym_night = 150,
  anon_sym_noon = 151,
  anon_sym_midnight = 152,
  sym_identifier = 153,
  sym_number = 154,
  sym_source_file = 155,
  sym__definition = 156,
  sym_function_definition = 157,
  sym_block = 158,
  sym__command = 159,
  sym_execute_command = 160,
  sym_execute_subcommand = 161,
  sym_execute_as = 162,
  sym_execute_at = 163,
  sym_execute_align = 164,
  sym_execute_anchored = 165,
  sym_execute_facing = 166,
  sym_execute_facing_entity = 167,
  sym_execute_in = 168,
  sym_execute_positioned = 169,
  sym_execute_positioned_as = 170,
  sym_execute_rotated = 171,
  sym_execute_rotated_as = 172,
  sym_execute_if = 173,
  sym_execute_unless = 174,
  sym_execute_store = 175,
  sym_execute_run = 176,
  sym__coordinate = 177,
  sym_relative_coordinate = 178,
  sym_relative_coordinate_plain = 179,
  sym_relative_coordinate_offset = 180,
  sym_local_coordinate = 181,
  sym_local_coordinate_plain = 182,
  sym_local_coordinate_offset = 183,
  sym_position = 184,
  sym_rotation = 185,
  sym_dimension = 186,
  sym_execute_condition = 187,
  sym_condition_block = 188,
  sym_condition_blocks = 189,
  sym_condition_data = 190,
  sym_condition_entity = 191,
  sym_condition_predicate = 192,
  sym_condition_score = 193,
  sym_store_target = 194,
  sym_block_state = 195,
  sym_target_selector = 196,
  sym_selector_type = 197,
  sym_selector_arguments = 198,
  sym_selector_argument = 199,
  sym_xp_type = 200,
  sym_range_value = 201,
  sym_xp_add_command = 202,
  sym_xp_set_command = 203,
  sym_xp_query_command = 204,
  sym_quoted_string = 205,
  sym_say_command = 206,
  sym_inv_clear_command = 207,
  sym_effect_clear_command = 208,
  sym_tellraw_command = 209,
  sym_effect_command = 210,
  sym_enchant_command = 211,
  sym_vanilla_effect = 212,
  sym_vanilla_enchant = 213,
  sym_custom_effect = 214,
  sym_custom_enchant = 215,
  sym_time_command = 216,
  sym_gm_survival_command = 217,
  sym_gm_adventure_command = 218,
  sym_gm_spectator_command = 219,
  sym_gm_creative_command = 220,
  sym_time_unit = 221,
  aux_sym_source_file_repeat1 = 222,
  aux_sym_block_repeat1 = 223,
  aux_sym_execute_command_repeat1 = 224,
  aux_sym_condition_block_repeat1 = 225,
  aux_sym_selector_arguments_repeat1 = 226,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [anon_sym_fn] = "fn",
  [anon_sym_LBRACE] = "{",
  [anon_sym_SEMI] = ";",
  [aux_sym_block_token1] = "block_token1",
  [anon_sym_RBRACE] = "}",
  [anon_sym_execute] = "execute",
  [anon_sym_as] = "as",
  [anon_sym_at] = "at",
  [anon_sym_align] = "align",
  [anon_sym_anchored] = "anchored",
  [anon_sym_eyes] = "eyes",
  [anon_sym_feet] = "feet",
  [anon_sym_facing] = "facing",
  [anon_sym_entity] = "entity",
  [anon_sym_in] = "in",
  [anon_sym_positioned] = "positioned",
  [anon_sym_rotated] = "rotated",
  [anon_sym_if] = "if",
  [anon_sym_unless] = "unless",
  [anon_sym_store] = "store",
  [anon_sym_result] = "result",
  [anon_sym_success] = "success",
  [anon_sym_byte] = "byte",
  [anon_sym_short] = "short",
  [anon_sym_int] = "int",
  [anon_sym_long] = "long",
  [anon_sym_float] = "float",
  [anon_sym_double] = "double",
  [anon_sym_run] = "run",
  [anon_sym_TILDE] = "~",
  [anon_sym_DASH] = "-",
  [anon_sym_CARET] = "^",
  [sym_align_axes] = "align_axes",
  [anon_sym_minecraft_COLONoverworld] = "minecraft:overworld",
  [anon_sym_minecraft_COLONthe_nether] = "minecraft:the_nether",
  [anon_sym_minecraft_COLONthe_end] = "minecraft:the_end",
  [anon_sym_block] = "block",
  [anon_sym_minecraft_COLON] = "minecraft:",
  [anon_sym_LBRACK] = "[",
  [anon_sym_COMMA] = ",",
  [anon_sym_RBRACK] = "]",
  [anon_sym_blocks] = "blocks",
  [anon_sym_masked] = "masked",
  [anon_sym_data] = "data",
  [anon_sym_storage] = "storage",
  [anon_sym_predicate] = "predicate",
  [anon_sym_score] = "score",
  [anon_sym_matches] = "matches",
  [anon_sym_LT] = "<",
  [anon_sym_LT_EQ] = "<=",
  [anon_sym_EQ] = "=",
  [anon_sym_GT_EQ] = ">=",
  [anon_sym_GT] = ">",
  [anon_sym_bossbar] = "bossbar",
  [anon_sym_value] = "value",
  [anon_sym_max] = "max",
  [sym_nbt_path] = "nbt_path",
  [anon_sym_ATs] = "@s",
  [anon_sym_ATa] = "@a",
  [anon_sym_ATp] = "@p",
  [anon_sym_ATr] = "@r",
  [anon_sym_ATe] = "@e",
  [anon_sym_levels] = "levels",
  [anon_sym_points] = "points",
  [anon_sym_DOT_DOT] = "..",
  [anon_sym_xpadd] = "xpadd",
  [anon_sym_xpset] = "xpset",
  [anon_sym_xpquery] = "xpquery",
  [aux_sym_quoted_string_token1] = "quoted_string_token1",
  [anon_sym_say] = "say",
  [anon_sym_clear] = "clear",
  [anon_sym_eclear] = "eclear",
  [anon_sym_tellraw] = "tellraw",
  [anon_sym_effect] = "effect",
  [anon_sym_give] = "give",
  [anon_sym_enchant] = "enchant",
  [anon_sym_minecraft_COLONspeed] = "minecraft:speed",
  [anon_sym_minecraft_COLONslowness] = "minecraft:slowness",
  [anon_sym_minecraft_COLONhaste] = "minecraft:haste",
  [anon_sym_minecraft_COLONmining_fatigue] = "minecraft:mining_fatigue",
  [anon_sym_minecraft_COLONstrength] = "minecraft:strength",
  [anon_sym_minecraft_COLONinstant_health] = "minecraft:instant_health",
  [anon_sym_minecraft_COLONinstant_damage] = "minecraft:instant_damage",
  [anon_sym_minecraft_COLONjump_boost] = "minecraft:jump_boost",
  [anon_sym_minecraft_COLONnausea] = "minecraft:nausea",
  [anon_sym_minecraft_COLONregeneration] = "minecraft:regeneration",
  [anon_sym_minecraft_COLONresistance] = "minecraft:resistance",
  [anon_sym_minecraft_COLONfire_resistance] = "minecraft:fire_resistance",
  [anon_sym_minecraft_COLONwater_breathing] = "minecraft:water_breathing",
  [anon_sym_minecraft_COLONinvisibility] = "minecraft:invisibility",
  [anon_sym_minecraft_COLONblindness] = "minecraft:blindness",
  [anon_sym_minecraft_COLONnight_vision] = "minecraft:night_vision",
  [anon_sym_minecraft_COLONhunger] = "minecraft:hunger",
  [anon_sym_minecraft_COLONweakness] = "minecraft:weakness",
  [anon_sym_minecraft_COLONpoison] = "minecraft:poison",
  [anon_sym_minecraft_COLONwither] = "minecraft:wither",
  [anon_sym_minecraft_COLONhealth_boost] = "minecraft:health_boost",
  [anon_sym_minecraft_COLONabsorption] = "minecraft:absorption",
  [anon_sym_minecraft_COLONsaturation] = "minecraft:saturation",
  [anon_sym_minecraft_COLONglowing] = "minecraft:glowing",
  [anon_sym_minecraft_COLONlevitation] = "minecraft:levitation",
  [anon_sym_minecraft_COLONluck] = "minecraft:luck",
  [anon_sym_minecraft_COLONunluck] = "minecraft:unluck",
  [anon_sym_minecraft_COLONslow_falling] = "minecraft:slow_falling",
  [anon_sym_minecraft_COLONconduit_power] = "minecraft:conduit_power",
  [anon_sym_minecraft_COLONdolphins_grace] = "minecraft:dolphins_grace",
  [anon_sym_minecraft_COLONbad_omen] = "minecraft:bad_omen",
  [anon_sym_minecraft_COLONhero_of_the_village] = "minecraft:hero_of_the_village",
  [anon_sym_minecraft_COLONprotection] = "minecraft:protection",
  [anon_sym_minecraft_COLONfire_protection] = "minecraft:fire_protection",
  [anon_sym_minecraft_COLONfeather_falling] = "minecraft:feather_falling",
  [anon_sym_minecraft_COLONblast_protection] = "minecraft:blast_protection",
  [anon_sym_minecraft_COLONprojectile_protection] = "minecraft:projectile_protection",
  [anon_sym_minecraft_COLONrespiration] = "minecraft:respiration",
  [anon_sym_minecraft_COLONaqua_affinity] = "minecraft:aqua_affinity",
  [anon_sym_minecraft_COLONthorns] = "minecraft:thorns",
  [anon_sym_minecraft_COLONdepth_strider] = "minecraft:depth_strider",
  [anon_sym_minecraft_COLONfrost_walker] = "minecraft:frost_walker",
  [anon_sym_minecraft_COLONbinding_curse] = "minecraft:binding_curse",
  [anon_sym_minecraft_COLONsharpness] = "minecraft:sharpness",
  [anon_sym_minecraft_COLONsmite] = "minecraft:smite",
  [anon_sym_minecraft_COLONbane_of_arthropods] = "minecraft:bane_of_arthropods",
  [anon_sym_minecraft_COLONknockback] = "minecraft:knockback",
  [anon_sym_minecraft_COLONfire_aspect] = "minecraft:fire_aspect",
  [anon_sym_minecraft_COLONlooting] = "minecraft:looting",
  [anon_sym_minecraft_COLONsweeping] = "minecraft:sweeping",
  [anon_sym_minecraft_COLONefficiency] = "minecraft:efficiency",
  [anon_sym_minecraft_COLONsilk_touch] = "minecraft:silk_touch",
  [anon_sym_minecraft_COLONunbreaking] = "minecraft:unbreaking",
  [anon_sym_minecraft_COLONfortune] = "minecraft:fortune",
  [anon_sym_minecraft_COLONpower] = "minecraft:power",
  [anon_sym_minecraft_COLONpunch] = "minecraft:punch",
  [anon_sym_minecraft_COLONflame] = "minecraft:flame",
  [anon_sym_minecraft_COLONinfinity] = "minecraft:infinity",
  [anon_sym_minecraft_COLONluck_of_the_sea] = "minecraft:luck_of_the_sea",
  [anon_sym_minecraft_COLONlure] = "minecraft:lure",
  [anon_sym_minecraft_COLONmending] = "minecraft:mending",
  [anon_sym_minecraft_COLONvanishing_curse] = "minecraft:vanishing_curse",
  [anon_sym_minecraft_COLONsoul_speed] = "minecraft:soul_speed",
  [anon_sym_minecraft_COLONswift_sneak] = "minecraft:swift_sneak",
  [sym_text] = "text",
  [anon_sym_time] = "time",
  [anon_sym_set] = "set",
  [anon_sym_query] = "query",
  [anon_sym_gms] = "gms",
  [anon_sym_gma] = "gma",
  [anon_sym_gmsp] = "gmsp",
  [anon_sym_gmc] = "gmc",
  [anon_sym_day] = "day",
  [anon_sym_night] = "night",
  [anon_sym_noon] = "noon",
  [anon_sym_midnight] = "midnight",
  [sym_identifier] = "identifier",
  [sym_number] = "number",
  [sym_source_file] = "source_file",
  [sym__definition] = "_definition",
  [sym_function_definition] = "function_definition",
  [sym_block] = "block",
  [sym__command] = "_command",
  [sym_execute_command] = "execute_command",
  [sym_execute_subcommand] = "execute_subcommand",
  [sym_execute_as] = "execute_as",
  [sym_execute_at] = "execute_at",
  [sym_execute_align] = "execute_align",
  [sym_execute_anchored] = "execute_anchored",
  [sym_execute_facing] = "execute_facing",
  [sym_execute_facing_entity] = "execute_facing_entity",
  [sym_execute_in] = "execute_in",
  [sym_execute_positioned] = "execute_positioned",
  [sym_execute_positioned_as] = "execute_positioned_as",
  [sym_execute_rotated] = "execute_rotated",
  [sym_execute_rotated_as] = "execute_rotated_as",
  [sym_execute_if] = "execute_if",
  [sym_execute_unless] = "execute_unless",
  [sym_execute_store] = "execute_store",
  [sym_execute_run] = "execute_run",
  [sym__coordinate] = "_coordinate",
  [sym_relative_coordinate] = "relative_coordinate",
  [sym_relative_coordinate_plain] = "relative_coordinate_plain",
  [sym_relative_coordinate_offset] = "relative_coordinate_offset",
  [sym_local_coordinate] = "local_coordinate",
  [sym_local_coordinate_plain] = "local_coordinate_plain",
  [sym_local_coordinate_offset] = "local_coordinate_offset",
  [sym_position] = "position",
  [sym_rotation] = "rotation",
  [sym_dimension] = "dimension",
  [sym_execute_condition] = "execute_condition",
  [sym_condition_block] = "condition_block",
  [sym_condition_blocks] = "condition_blocks",
  [sym_condition_data] = "condition_data",
  [sym_condition_entity] = "condition_entity",
  [sym_condition_predicate] = "condition_predicate",
  [sym_condition_score] = "condition_score",
  [sym_store_target] = "store_target",
  [sym_block_state] = "block_state",
  [sym_target_selector] = "target_selector",
  [sym_selector_type] = "selector_type",
  [sym_selector_arguments] = "selector_arguments",
  [sym_selector_argument] = "selector_argument",
  [sym_xp_type] = "xp_type",
  [sym_range_value] = "range_value",
  [sym_xp_add_command] = "xp_add_command",
  [sym_xp_set_command] = "xp_set_command",
  [sym_xp_query_command] = "xp_query_command",
  [sym_quoted_string] = "quoted_string",
  [sym_say_command] = "say_command",
  [sym_inv_clear_command] = "inv_clear_command",
  [sym_effect_clear_command] = "effect_clear_command",
  [sym_tellraw_command] = "tellraw_command",
  [sym_effect_command] = "effect_command",
  [sym_enchant_command] = "enchant_command",
  [sym_vanilla_effect] = "vanilla_effect",
  [sym_vanilla_enchant] = "vanilla_enchant",
  [sym_custom_effect] = "custom_effect",
  [sym_custom_enchant] = "custom_enchant",
  [sym_time_command] = "time_command",
  [sym_gm_survival_command] = "gm_survival_command",
  [sym_gm_adventure_command] = "gm_adventure_command",
  [sym_gm_spectator_command] = "gm_spectator_command",
  [sym_gm_creative_command] = "gm_creative_command",
  [sym_time_unit] = "time_unit",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_block_repeat1] = "block_repeat1",
  [aux_sym_execute_command_repeat1] = "execute_command_repeat1",
  [aux_sym_condition_block_repeat1] = "condition_block_repeat1",
  [aux_sym_selector_arguments_repeat1] = "selector_arguments_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [anon_sym_fn] = anon_sym_fn,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [aux_sym_block_token1] = aux_sym_block_token1,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_execute] = anon_sym_execute,
  [anon_sym_as] = anon_sym_as,
  [anon_sym_at] = anon_sym_at,
  [anon_sym_align] = anon_sym_align,
  [anon_sym_anchored] = anon_sym_anchored,
  [anon_sym_eyes] = anon_sym_eyes,
  [anon_sym_feet] = anon_sym_feet,
  [anon_sym_facing] = anon_sym_facing,
  [anon_sym_entity] = anon_sym_entity,
  [anon_sym_in] = anon_sym_in,
  [anon_sym_positioned] = anon_sym_positioned,
  [anon_sym_rotated] = anon_sym_rotated,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_unless] = anon_sym_unless,
  [anon_sym_store] = anon_sym_store,
  [anon_sym_result] = anon_sym_result,
  [anon_sym_success] = anon_sym_success,
  [anon_sym_byte] = anon_sym_byte,
  [anon_sym_short] = anon_sym_short,
  [anon_sym_int] = anon_sym_int,
  [anon_sym_long] = anon_sym_long,
  [anon_sym_float] = anon_sym_float,
  [anon_sym_double] = anon_sym_double,
  [anon_sym_run] = anon_sym_run,
  [anon_sym_TILDE] = anon_sym_TILDE,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_CARET] = anon_sym_CARET,
  [sym_align_axes] = sym_align_axes,
  [anon_sym_minecraft_COLONoverworld] = anon_sym_minecraft_COLONoverworld,
  [anon_sym_minecraft_COLONthe_nether] = anon_sym_minecraft_COLONthe_nether,
  [anon_sym_minecraft_COLONthe_end] = anon_sym_minecraft_COLONthe_end,
  [anon_sym_block] = anon_sym_block,
  [anon_sym_minecraft_COLON] = anon_sym_minecraft_COLON,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_blocks] = anon_sym_blocks,
  [anon_sym_masked] = anon_sym_masked,
  [anon_sym_data] = anon_sym_data,
  [anon_sym_storage] = anon_sym_storage,
  [anon_sym_predicate] = anon_sym_predicate,
  [anon_sym_score] = anon_sym_score,
  [anon_sym_matches] = anon_sym_matches,
  [anon_sym_LT] = anon_sym_LT,
  [anon_sym_LT_EQ] = anon_sym_LT_EQ,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_GT_EQ] = anon_sym_GT_EQ,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_bossbar] = anon_sym_bossbar,
  [anon_sym_value] = anon_sym_value,
  [anon_sym_max] = anon_sym_max,
  [sym_nbt_path] = sym_nbt_path,
  [anon_sym_ATs] = anon_sym_ATs,
  [anon_sym_ATa] = anon_sym_ATa,
  [anon_sym_ATp] = anon_sym_ATp,
  [anon_sym_ATr] = anon_sym_ATr,
  [anon_sym_ATe] = anon_sym_ATe,
  [anon_sym_levels] = anon_sym_levels,
  [anon_sym_points] = anon_sym_points,
  [anon_sym_DOT_DOT] = anon_sym_DOT_DOT,
  [anon_sym_xpadd] = anon_sym_xpadd,
  [anon_sym_xpset] = anon_sym_xpset,
  [anon_sym_xpquery] = anon_sym_xpquery,
  [aux_sym_quoted_string_token1] = aux_sym_quoted_string_token1,
  [anon_sym_say] = anon_sym_say,
  [anon_sym_clear] = anon_sym_clear,
  [anon_sym_eclear] = anon_sym_eclear,
  [anon_sym_tellraw] = anon_sym_tellraw,
  [anon_sym_effect] = anon_sym_effect,
  [anon_sym_give] = anon_sym_give,
  [anon_sym_enchant] = anon_sym_enchant,
  [anon_sym_minecraft_COLONspeed] = anon_sym_minecraft_COLONspeed,
  [anon_sym_minecraft_COLONslowness] = anon_sym_minecraft_COLONslowness,
  [anon_sym_minecraft_COLONhaste] = anon_sym_minecraft_COLONhaste,
  [anon_sym_minecraft_COLONmining_fatigue] = anon_sym_minecraft_COLONmining_fatigue,
  [anon_sym_minecraft_COLONstrength] = anon_sym_minecraft_COLONstrength,
  [anon_sym_minecraft_COLONinstant_health] = anon_sym_minecraft_COLONinstant_health,
  [anon_sym_minecraft_COLONinstant_damage] = anon_sym_minecraft_COLONinstant_damage,
  [anon_sym_minecraft_COLONjump_boost] = anon_sym_minecraft_COLONjump_boost,
  [anon_sym_minecraft_COLONnausea] = anon_sym_minecraft_COLONnausea,
  [anon_sym_minecraft_COLONregeneration] = anon_sym_minecraft_COLONregeneration,
  [anon_sym_minecraft_COLONresistance] = anon_sym_minecraft_COLONresistance,
  [anon_sym_minecraft_COLONfire_resistance] = anon_sym_minecraft_COLONfire_resistance,
  [anon_sym_minecraft_COLONwater_breathing] = anon_sym_minecraft_COLONwater_breathing,
  [anon_sym_minecraft_COLONinvisibility] = anon_sym_minecraft_COLONinvisibility,
  [anon_sym_minecraft_COLONblindness] = anon_sym_minecraft_COLONblindness,
  [anon_sym_minecraft_COLONnight_vision] = anon_sym_minecraft_COLONnight_vision,
  [anon_sym_minecraft_COLONhunger] = anon_sym_minecraft_COLONhunger,
  [anon_sym_minecraft_COLONweakness] = anon_sym_minecraft_COLONweakness,
  [anon_sym_minecraft_COLONpoison] = anon_sym_minecraft_COLONpoison,
  [anon_sym_minecraft_COLONwither] = anon_sym_minecraft_COLONwither,
  [anon_sym_minecraft_COLONhealth_boost] = anon_sym_minecraft_COLONhealth_boost,
  [anon_sym_minecraft_COLONabsorption] = anon_sym_minecraft_COLONabsorption,
  [anon_sym_minecraft_COLONsaturation] = anon_sym_minecraft_COLONsaturation,
  [anon_sym_minecraft_COLONglowing] = anon_sym_minecraft_COLONglowing,
  [anon_sym_minecraft_COLONlevitation] = anon_sym_minecraft_COLONlevitation,
  [anon_sym_minecraft_COLONluck] = anon_sym_minecraft_COLONluck,
  [anon_sym_minecraft_COLONunluck] = anon_sym_minecraft_COLONunluck,
  [anon_sym_minecraft_COLONslow_falling] = anon_sym_minecraft_COLONslow_falling,
  [anon_sym_minecraft_COLONconduit_power] = anon_sym_minecraft_COLONconduit_power,
  [anon_sym_minecraft_COLONdolphins_grace] = anon_sym_minecraft_COLONdolphins_grace,
  [anon_sym_minecraft_COLONbad_omen] = anon_sym_minecraft_COLONbad_omen,
  [anon_sym_minecraft_COLONhero_of_the_village] = anon_sym_minecraft_COLONhero_of_the_village,
  [anon_sym_minecraft_COLONprotection] = anon_sym_minecraft_COLONprotection,
  [anon_sym_minecraft_COLONfire_protection] = anon_sym_minecraft_COLONfire_protection,
  [anon_sym_minecraft_COLONfeather_falling] = anon_sym_minecraft_COLONfeather_falling,
  [anon_sym_minecraft_COLONblast_protection] = anon_sym_minecraft_COLONblast_protection,
  [anon_sym_minecraft_COLONprojectile_protection] = anon_sym_minecraft_COLONprojectile_protection,
  [anon_sym_minecraft_COLONrespiration] = anon_sym_minecraft_COLONrespiration,
  [anon_sym_minecraft_COLONaqua_affinity] = anon_sym_minecraft_COLONaqua_affinity,
  [anon_sym_minecraft_COLONthorns] = anon_sym_minecraft_COLONthorns,
  [anon_sym_minecraft_COLONdepth_strider] = anon_sym_minecraft_COLONdepth_strider,
  [anon_sym_minecraft_COLONfrost_walker] = anon_sym_minecraft_COLONfrost_walker,
  [anon_sym_minecraft_COLONbinding_curse] = anon_sym_minecraft_COLONbinding_curse,
  [anon_sym_minecraft_COLONsharpness] = anon_sym_minecraft_COLONsharpness,
  [anon_sym_minecraft_COLONsmite] = anon_sym_minecraft_COLONsmite,
  [anon_sym_minecraft_COLONbane_of_arthropods] = anon_sym_minecraft_COLONbane_of_arthropods,
  [anon_sym_minecraft_COLONknockback] = anon_sym_minecraft_COLONknockback,
  [anon_sym_minecraft_COLONfire_aspect] = anon_sym_minecraft_COLONfire_aspect,
  [anon_sym_minecraft_COLONlooting] = anon_sym_minecraft_COLONlooting,
  [anon_sym_minecraft_COLONsweeping] = anon_sym_minecraft_COLONsweeping,
  [anon_sym_minecraft_COLONefficiency] = anon_sym_minecraft_COLONefficiency,
  [anon_sym_minecraft_COLONsilk_touch] = anon_sym_minecraft_COLONsilk_touch,
  [anon_sym_minecraft_COLONunbreaking] = anon_sym_minecraft_COLONunbreaking,
  [anon_sym_minecraft_COLONfortune] = anon_sym_minecraft_COLONfortune,
  [anon_sym_minecraft_COLONpower] = anon_sym_minecraft_COLONpower,
  [anon_sym_minecraft_COLONpunch] = anon_sym_minecraft_COLONpunch,
  [anon_sym_minecraft_COLONflame] = anon_sym_minecraft_COLONflame,
  [anon_sym_minecraft_COLONinfinity] = anon_sym_minecraft_COLONinfinity,
  [anon_sym_minecraft_COLONluck_of_the_sea] = anon_sym_minecraft_COLONluck_of_the_sea,
  [anon_sym_minecraft_COLONlure] = anon_sym_minecraft_COLONlure,
  [anon_sym_minecraft_COLONmending] = anon_sym_minecraft_COLONmending,
  [anon_sym_minecraft_COLONvanishing_curse] = anon_sym_minecraft_COLONvanishing_curse,
  [anon_sym_minecraft_COLONsoul_speed] = anon_sym_minecraft_COLONsoul_speed,
  [anon_sym_minecraft_COLONswift_sneak] = anon_sym_minecraft_COLONswift_sneak,
  [sym_text] = sym_text,
  [anon_sym_time] = anon_sym_time,
  [anon_sym_set] = anon_sym_set,
  [anon_sym_query] = anon_sym_query,
  [anon_sym_gms] = anon_sym_gms,
  [anon_sym_gma] = anon_sym_gma,
  [anon_sym_gmsp] = anon_sym_gmsp,
  [anon_sym_gmc] = anon_sym_gmc,
  [anon_sym_day] = anon_sym_day,
  [anon_sym_night] = anon_sym_night,
  [anon_sym_noon] = anon_sym_noon,
  [anon_sym_midnight] = anon_sym_midnight,
  [sym_identifier] = sym_identifier,
  [sym_number] = sym_number,
  [sym_source_file] = sym_source_file,
  [sym__definition] = sym__definition,
  [sym_function_definition] = sym_function_definition,
  [sym_block] = sym_block,
  [sym__command] = sym__command,
  [sym_execute_command] = sym_execute_command,
  [sym_execute_subcommand] = sym_execute_subcommand,
  [sym_execute_as] = sym_execute_as,
  [sym_execute_at] = sym_execute_at,
  [sym_execute_align] = sym_execute_align,
  [sym_execute_anchored] = sym_execute_anchored,
  [sym_execute_facing] = sym_execute_facing,
  [sym_execute_facing_entity] = sym_execute_facing_entity,
  [sym_execute_in] = sym_execute_in,
  [sym_execute_positioned] = sym_execute_positioned,
  [sym_execute_positioned_as] = sym_execute_positioned_as,
  [sym_execute_rotated] = sym_execute_rotated,
  [sym_execute_rotated_as] = sym_execute_rotated_as,
  [sym_execute_if] = sym_execute_if,
  [sym_execute_unless] = sym_execute_unless,
  [sym_execute_store] = sym_execute_store,
  [sym_execute_run] = sym_execute_run,
  [sym__coordinate] = sym__coordinate,
  [sym_relative_coordinate] = sym_relative_coordinate,
  [sym_relative_coordinate_plain] = sym_relative_coordinate_plain,
  [sym_relative_coordinate_offset] = sym_relative_coordinate_offset,
  [sym_local_coordinate] = sym_local_coordinate,
  [sym_local_coordinate_plain] = sym_local_coordinate_plain,
  [sym_local_coordinate_offset] = sym_local_coordinate_offset,
  [sym_position] = sym_position,
  [sym_rotation] = sym_rotation,
  [sym_dimension] = sym_dimension,
  [sym_execute_condition] = sym_execute_condition,
  [sym_condition_block] = sym_condition_block,
  [sym_condition_blocks] = sym_condition_blocks,
  [sym_condition_data] = sym_condition_data,
  [sym_condition_entity] = sym_condition_entity,
  [sym_condition_predicate] = sym_condition_predicate,
  [sym_condition_score] = sym_condition_score,
  [sym_store_target] = sym_store_target,
  [sym_block_state] = sym_block_state,
  [sym_target_selector] = sym_target_selector,
  [sym_selector_type] = sym_selector_type,
  [sym_selector_arguments] = sym_selector_arguments,
  [sym_selector_argument] = sym_selector_argument,
  [sym_xp_type] = sym_xp_type,
  [sym_range_value] = sym_range_value,
  [sym_xp_add_command] = sym_xp_add_command,
  [sym_xp_set_command] = sym_xp_set_command,
  [sym_xp_query_command] = sym_xp_query_command,
  [sym_quoted_string] = sym_quoted_string,
  [sym_say_command] = sym_say_command,
  [sym_inv_clear_command] = sym_inv_clear_command,
  [sym_effect_clear_command] = sym_effect_clear_command,
  [sym_tellraw_command] = sym_tellraw_command,
  [sym_effect_command] = sym_effect_command,
  [sym_enchant_command] = sym_enchant_command,
  [sym_vanilla_effect] = sym_vanilla_effect,
  [sym_vanilla_enchant] = sym_vanilla_enchant,
  [sym_custom_effect] = sym_custom_effect,
  [sym_custom_enchant] = sym_custom_enchant,
  [sym_time_command] = sym_time_command,
  [sym_gm_survival_command] = sym_gm_survival_command,
  [sym_gm_adventure_command] = sym_gm_adventure_command,
  [sym_gm_spectator_command] = sym_gm_spectator_command,
  [sym_gm_creative_command] = sym_gm_creative_command,
  [sym_time_unit] = sym_time_unit,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_block_repeat1] = aux_sym_block_repeat1,
  [aux_sym_execute_command_repeat1] = aux_sym_execute_command_repeat1,
  [aux_sym_condition_block_repeat1] = aux_sym_condition_block_repeat1,
  [aux_sym_selector_arguments_repeat1] = aux_sym_selector_arguments_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_fn] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_block_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_execute] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_as] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_at] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_align] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_anchored] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_eyes] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_feet] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_facing] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_entity] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_in] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_positioned] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_rotated] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_unless] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_store] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_result] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_success] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_byte] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_short] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_long] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_double] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_run] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_CARET] = {
    .visible = true,
    .named = false,
  },
  [sym_align_axes] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_minecraft_COLONoverworld] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONthe_nether] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONthe_end] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_block] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_blocks] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_masked] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_data] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_storage] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_predicate] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_score] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_matches] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_bossbar] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_value] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_max] = {
    .visible = true,
    .named = false,
  },
  [sym_nbt_path] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_ATs] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATa] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATp] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATr] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATe] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_levels] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_points] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_xpadd] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_xpset] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_xpquery] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_quoted_string_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_say] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_clear] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_eclear] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_tellraw] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_effect] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_give] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_enchant] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONspeed] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONslowness] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONhaste] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONmining_fatigue] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONstrength] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONinstant_health] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONinstant_damage] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONjump_boost] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONnausea] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONregeneration] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONresistance] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONfire_resistance] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONwater_breathing] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONinvisibility] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONblindness] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONnight_vision] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONhunger] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONweakness] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONpoison] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONwither] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONhealth_boost] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONabsorption] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONsaturation] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONglowing] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONlevitation] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONluck] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONunluck] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONslow_falling] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONconduit_power] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONdolphins_grace] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONbad_omen] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONhero_of_the_village] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONprotection] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONfire_protection] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONfeather_falling] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONblast_protection] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONprojectile_protection] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONrespiration] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONaqua_affinity] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONthorns] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONdepth_strider] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONfrost_walker] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONbinding_curse] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONsharpness] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONsmite] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONbane_of_arthropods] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONknockback] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONfire_aspect] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONlooting] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONsweeping] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONefficiency] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONsilk_touch] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONunbreaking] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONfortune] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONpower] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONpunch] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONflame] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONinfinity] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONluck_of_the_sea] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONlure] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONmending] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONvanishing_curse] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONsoul_speed] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLONswift_sneak] = {
    .visible = true,
    .named = false,
  },
  [sym_text] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_time] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_set] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_query] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_gms] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_gma] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_gmsp] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_gmc] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_day] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_night] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_noon] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_midnight] = {
    .visible = true,
    .named = false,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym__definition] = {
    .visible = false,
    .named = true,
  },
  [sym_function_definition] = {
    .visible = true,
    .named = true,
  },
  [sym_block] = {
    .visible = true,
    .named = true,
  },
  [sym__command] = {
    .visible = false,
    .named = true,
  },
  [sym_execute_command] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_subcommand] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_as] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_at] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_align] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_anchored] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_facing] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_facing_entity] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_in] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_positioned] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_positioned_as] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_rotated] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_rotated_as] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_if] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_unless] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_store] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_run] = {
    .visible = true,
    .named = true,
  },
  [sym__coordinate] = {
    .visible = false,
    .named = true,
  },
  [sym_relative_coordinate] = {
    .visible = true,
    .named = true,
  },
  [sym_relative_coordinate_plain] = {
    .visible = true,
    .named = true,
  },
  [sym_relative_coordinate_offset] = {
    .visible = true,
    .named = true,
  },
  [sym_local_coordinate] = {
    .visible = true,
    .named = true,
  },
  [sym_local_coordinate_plain] = {
    .visible = true,
    .named = true,
  },
  [sym_local_coordinate_offset] = {
    .visible = true,
    .named = true,
  },
  [sym_position] = {
    .visible = true,
    .named = true,
  },
  [sym_rotation] = {
    .visible = true,
    .named = true,
  },
  [sym_dimension] = {
    .visible = true,
    .named = true,
  },
  [sym_execute_condition] = {
    .visible = true,
    .named = true,
  },
  [sym_condition_block] = {
    .visible = true,
    .named = true,
  },
  [sym_condition_blocks] = {
    .visible = true,
    .named = true,
  },
  [sym_condition_data] = {
    .visible = true,
    .named = true,
  },
  [sym_condition_entity] = {
    .visible = true,
    .named = true,
  },
  [sym_condition_predicate] = {
    .visible = true,
    .named = true,
  },
  [sym_condition_score] = {
    .visible = true,
    .named = true,
  },
  [sym_store_target] = {
    .visible = true,
    .named = true,
  },
  [sym_block_state] = {
    .visible = true,
    .named = true,
  },
  [sym_target_selector] = {
    .visible = true,
    .named = true,
  },
  [sym_selector_type] = {
    .visible = true,
    .named = true,
  },
  [sym_selector_arguments] = {
    .visible = true,
    .named = true,
  },
  [sym_selector_argument] = {
    .visible = true,
    .named = true,
  },
  [sym_xp_type] = {
    .visible = true,
    .named = true,
  },
  [sym_range_value] = {
    .visible = true,
    .named = true,
  },
  [sym_xp_add_command] = {
    .visible = true,
    .named = true,
  },
  [sym_xp_set_command] = {
    .visible = true,
    .named = true,
  },
  [sym_xp_query_command] = {
    .visible = true,
    .named = true,
  },
  [sym_quoted_string] = {
    .visible = true,
    .named = true,
  },
  [sym_say_command] = {
    .visible = true,
    .named = true,
  },
  [sym_inv_clear_command] = {
    .visible = true,
    .named = true,
  },
  [sym_effect_clear_command] = {
    .visible = true,
    .named = true,
  },
  [sym_tellraw_command] = {
    .visible = true,
    .named = true,
  },
  [sym_effect_command] = {
    .visible = true,
    .named = true,
  },
  [sym_enchant_command] = {
    .visible = true,
    .named = true,
  },
  [sym_vanilla_effect] = {
    .visible = true,
    .named = true,
  },
  [sym_vanilla_enchant] = {
    .visible = true,
    .named = true,
  },
  [sym_custom_effect] = {
    .visible = true,
    .named = true,
  },
  [sym_custom_enchant] = {
    .visible = true,
    .named = true,
  },
  [sym_time_command] = {
    .visible = true,
    .named = true,
  },
  [sym_gm_survival_command] = {
    .visible = true,
    .named = true,
  },
  [sym_gm_adventure_command] = {
    .visible = true,
    .named = true,
  },
  [sym_gm_spectator_command] = {
    .visible = true,
    .named = true,
  },
  [sym_gm_creative_command] = {
    .visible = true,
    .named = true,
  },
  [sym_time_unit] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_execute_command_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_condition_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_selector_arguments_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_amount = 1,
  field_amplifier = 2,
  field_anchor = 3,
  field_axes = 4,
  field_block = 5,
  field_body = 6,
  field_command = 7,
  field_condition = 8,
  field_destination = 9,
  field_dimension = 10,
  field_duration = 11,
  field_effect_type = 12,
  field_enchantment = 13,
  field_end = 14,
  field_etype = 15,
  field_id = 16,
  field_key = 17,
  field_level = 18,
  field_message = 19,
  field_mode = 20,
  field_name = 21,
  field_objective = 22,
  field_operator = 23,
  field_path = 24,
  field_pitch = 25,
  field_pos = 26,
  field_query_type = 27,
  field_range = 28,
  field_rot = 29,
  field_scale = 30,
  field_selector = 31,
  field_source = 32,
  field_source_objective = 33,
  field_start = 34,
  field_target = 35,
  field_type = 36,
  field_value = 37,
  field_x = 38,
  field_y = 39,
  field_yaw = 40,
  field_z = 41,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_amount] = "amount",
  [field_amplifier] = "amplifier",
  [field_anchor] = "anchor",
  [field_axes] = "axes",
  [field_block] = "block",
  [field_body] = "body",
  [field_command] = "command",
  [field_condition] = "condition",
  [field_destination] = "destination",
  [field_dimension] = "dimension",
  [field_duration] = "duration",
  [field_effect_type] = "effect_type",
  [field_enchantment] = "enchantment",
  [field_end] = "end",
  [field_etype] = "etype",
  [field_id] = "id",
  [field_key] = "key",
  [field_level] = "level",
  [field_message] = "message",
  [field_mode] = "mode",
  [field_name] = "name",
  [field_objective] = "objective",
  [field_operator] = "operator",
  [field_path] = "path",
  [field_pitch] = "pitch",
  [field_pos] = "pos",
  [field_query_type] = "query_type",
  [field_range] = "range",
  [field_rot] = "rot",
  [field_scale] = "scale",
  [field_selector] = "selector",
  [field_source] = "source",
  [field_source_objective] = "source_objective",
  [field_start] = "start",
  [field_target] = "target",
  [field_type] = "type",
  [field_value] = "value",
  [field_x] = "x",
  [field_y] = "y",
  [field_yaw] = "yaw",
  [field_z] = "z",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 2},
  [2] = {.index = 2, .length = 1},
  [3] = {.index = 3, .length = 1},
  [4] = {.index = 4, .length = 1},
  [5] = {.index = 5, .length = 1},
  [6] = {.index = 6, .length = 1},
  [7] = {.index = 7, .length = 1},
  [8] = {.index = 8, .length = 1},
  [9] = {.index = 9, .length = 1},
  [10] = {.index = 10, .length = 1},
  [11] = {.index = 11, .length = 1},
  [12] = {.index = 12, .length = 2},
  [13] = {.index = 14, .length = 2},
  [14] = {.index = 16, .length = 1},
  [15] = {.index = 17, .length = 1},
  [16] = {.index = 18, .length = 1},
  [17] = {.index = 19, .length = 2},
  [18] = {.index = 21, .length = 1},
  [19] = {.index = 22, .length = 2},
  [20] = {.index = 24, .length = 3},
  [21] = {.index = 27, .length = 3},
  [22] = {.index = 30, .length = 2},
  [23] = {.index = 32, .length = 3},
  [24] = {.index = 35, .length = 2},
  [25] = {.index = 37, .length = 4},
  [26] = {.index = 41, .length = 3},
  [27] = {.index = 44, .length = 3},
  [28] = {.index = 47, .length = 2},
  [29] = {.index = 49, .length = 2},
  [30] = {.index = 51, .length = 2},
  [31] = {.index = 53, .length = 2},
  [32] = {.index = 55, .length = 2},
  [33] = {.index = 57, .length = 2},
  [34] = {.index = 59, .length = 2},
  [35] = {.index = 61, .length = 4},
  [36] = {.index = 65, .length = 2},
  [37] = {.index = 67, .length = 4},
  [38] = {.index = 71, .length = 5},
  [39] = {.index = 76, .length = 4},
  [40] = {.index = 80, .length = 4},
  [41] = {.index = 84, .length = 5},
  [42] = {.index = 89, .length = 5},
  [43] = {.index = 94, .length = 5},
  [44] = {.index = 99, .length = 6},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_body, 2},
    {field_name, 1},
  [2] =
    {field_selector, 0},
  [3] =
    {field_message, 1},
  [4] =
    {field_target, 1},
  [5] =
    {field_axes, 1},
  [6] =
    {field_anchor, 1},
  [7] =
    {field_pos, 1},
  [8] =
    {field_dimension, 1},
  [9] =
    {field_rot, 1},
  [10] =
    {field_condition, 1},
  [11] =
    {field_command, 1},
  [12] =
    {field_etype, 2},
    {field_target, 1},
  [14] =
    {field_message, 2},
    {field_target, 1},
  [16] =
    {field_value, 2},
  [17] =
    {field_query_type, 2},
  [18] =
    {field_target, 2},
  [19] =
    {field_pitch, 1},
    {field_yaw, 0},
  [21] =
    {field_id, 1},
  [22] =
    {field_mode, 1},
    {field_target, 2},
  [24] =
    {field_amount, 1},
    {field_etype, 3},
    {field_target, 2},
  [27] =
    {field_enchantment, 2},
    {field_level, 3},
    {field_target, 1},
  [30] =
    {field_anchor, 3},
    {field_target, 2},
  [32] =
    {field_x, 0},
    {field_y, 1},
    {field_z, 2},
  [35] =
    {field_block, 2},
    {field_pos, 1},
  [37] =
    {field_amplifier, 4},
    {field_duration, 3},
    {field_effect_type, 2},
    {field_target, 1},
  [41] =
    {field_block, 2},
    {field_block, 3},
    {field_pos, 1},
  [44] =
    {field_destination, 3},
    {field_end, 2},
    {field_start, 1},
  [47] =
    {field_path, 3},
    {field_target, 2},
  [49] =
    {field_path, 3},
    {field_pos, 2},
  [51] =
    {field_path, 3},
    {field_source, 2},
  [53] =
    {field_path, 2},
    {field_target, 1},
  [55] =
    {field_path, 2},
    {field_pos, 1},
  [57] =
    {field_path, 2},
    {field_source, 1},
  [59] =
    {field_objective, 2},
    {field_target, 1},
  [61] =
    {field_mode, 1},
    {field_scale, 4},
    {field_target, 2},
    {field_type, 3},
  [65] =
    {field_key, 0},
    {field_value, 2},
  [67] =
    {field_amplifier, 5},
    {field_duration, 4},
    {field_effect_type, 3},
    {field_target, 2},
  [71] =
    {field_amplifier, 5},
    {field_duration, 4},
    {field_effect_type, 2},
    {field_effect_type, 3},
    {field_target, 1},
  [76] =
    {field_block, 2},
    {field_block, 3},
    {field_block, 4},
    {field_pos, 1},
  [80] =
    {field_objective, 2},
    {field_operator, 3},
    {field_range, 4},
    {field_target, 1},
  [84] =
    {field_amplifier, 6},
    {field_duration, 5},
    {field_effect_type, 3},
    {field_effect_type, 4},
    {field_target, 2},
  [89] =
    {field_block, 2},
    {field_block, 3},
    {field_block, 4},
    {field_block, 5},
    {field_pos, 1},
  [94] =
    {field_objective, 2},
    {field_operator, 3},
    {field_source, 4},
    {field_source_objective, 5},
    {field_target, 1},
  [99] =
    {field_block, 2},
    {field_block, 3},
    {field_block, 4},
    {field_block, 5},
    {field_block, 6},
    {field_pos, 1},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 2,
  [10] = 10,
  [11] = 3,
  [12] = 12,
  [13] = 13,
  [14] = 5,
  [15] = 6,
  [16] = 4,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 23,
  [25] = 22,
  [26] = 26,
  [27] = 26,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 115,
  [118] = 118,
  [119] = 119,
  [120] = 116,
  [121] = 116,
  [122] = 115,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
  [136] = 136,
  [137] = 137,
  [138] = 138,
  [139] = 139,
  [140] = 140,
  [141] = 141,
  [142] = 142,
  [143] = 143,
  [144] = 144,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 149,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 156,
  [157] = 157,
  [158] = 158,
  [159] = 159,
  [160] = 156,
  [161] = 161,
  [162] = 162,
  [163] = 163,
  [164] = 35,
  [165] = 34,
  [166] = 159,
  [167] = 156,
  [168] = 159,
  [169] = 156,
  [170] = 159,
  [171] = 50,
  [172] = 172,
  [173] = 173,
  [174] = 96,
  [175] = 175,
  [176] = 2,
  [177] = 35,
  [178] = 34,
  [179] = 179,
  [180] = 180,
  [181] = 2,
  [182] = 182,
  [183] = 183,
  [184] = 94,
  [185] = 185,
  [186] = 186,
  [187] = 187,
  [188] = 188,
  [189] = 189,
  [190] = 190,
  [191] = 38,
  [192] = 41,
  [193] = 39,
  [194] = 40,
  [195] = 43,
  [196] = 44,
  [197] = 197,
  [198] = 3,
  [199] = 3,
  [200] = 200,
  [201] = 201,
  [202] = 202,
  [203] = 203,
  [204] = 204,
  [205] = 36,
  [206] = 206,
  [207] = 207,
  [208] = 208,
  [209] = 209,
  [210] = 210,
  [211] = 211,
  [212] = 212,
  [213] = 213,
  [214] = 214,
  [215] = 215,
  [216] = 216,
  [217] = 38,
  [218] = 41,
  [219] = 4,
  [220] = 39,
  [221] = 40,
  [222] = 222,
  [223] = 5,
  [224] = 43,
  [225] = 44,
  [226] = 226,
  [227] = 6,
  [228] = 228,
  [229] = 229,
  [230] = 230,
  [231] = 231,
  [232] = 232,
  [233] = 233,
  [234] = 234,
  [235] = 235,
  [236] = 236,
  [237] = 237,
  [238] = 238,
  [239] = 239,
  [240] = 240,
  [241] = 241,
  [242] = 207,
  [243] = 243,
  [244] = 36,
  [245] = 245,
  [246] = 246,
  [247] = 247,
  [248] = 4,
  [249] = 5,
  [250] = 6,
  [251] = 251,
  [252] = 252,
  [253] = 253,
  [254] = 254,
  [255] = 255,
  [256] = 256,
  [257] = 215,
  [258] = 254,
  [259] = 259,
  [260] = 254,
  [261] = 207,
  [262] = 262,
  [263] = 263,
  [264] = 264,
  [265] = 265,
  [266] = 266,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(781);
      ADVANCE_MAP(
        '"', 5,
        ',', 824,
        '-', 813,
        '.', 8,
        ';', 784,
        '<', 833,
        '=', 835,
        '>', 837,
        '@', 43,
        '[', 822,
        ']', 825,
        '^', 814,
        'a', 411,
        'b', 412,
        'c', 413,
        'd', 44,
        'e', 146,
        'f', 56,
        'g', 339,
        'i', 273,
        'l', 246,
        'm', 45,
        'n', 340,
        'p', 511,
        'q', 745,
        'r', 201,
        's', 47,
        't', 199,
        'u', 489,
        'v', 58,
        '{', 783,
        '}', 786,
        '~', 812,
      );
      if (('x' <= lookahead && lookahead <= 'z')) ADVANCE(815);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(968);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(5);
      if (lookahead == ',') ADVANCE(824);
      if (lookahead == '.') ADVANCE(8);
      if (lookahead == ']') ADVANCE(825);
      if (lookahead == 'm') ADVANCE(956);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(968);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 2:
      ADVANCE_MAP(
        '"', 5,
        '-', 813,
        '.', 8,
        ';', 784,
        '[', 822,
        '^', 814,
        'a', 411,
        'c', 413,
        'e', 148,
        'f', 55,
        'g', 439,
        'i', 274,
        'l', 245,
        'm', 78,
        'p', 510,
        'r', 513,
        's', 48,
        't', 199,
        'u', 489,
        'x', 564,
        '~', 812,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(968);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(5);
      if (lookahead == '[') ADVANCE(822);
      if (lookahead == 'm') ADVANCE(955);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(3);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 4:
      if (lookahead == '"') ADVANCE(5);
      if (lookahead == 'm') ADVANCE(394);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(4);
      END_STATE();
    case 5:
      if (lookahead == '"') ADVANCE(854);
      if (lookahead != 0) ADVANCE(5);
      END_STATE();
    case 6:
      if (lookahead == '-') ADVANCE(813);
      if (lookahead == '[') ADVANCE(822);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(6);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(841);
      if (lookahead == '.' ||
          ('A' <= lookahead && lookahead <= '\\') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= '{') ||
          lookahead == '}') ADVANCE(842);
      END_STATE();
    case 7:
      if (lookahead == '-') ADVANCE(813);
      if (lookahead == 'm') ADVANCE(957);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(7);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(968);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 8:
      if (lookahead == '.') ADVANCE(850);
      END_STATE();
    case 9:
      if (lookahead == ':') ADVANCE(821);
      END_STATE();
    case 10:
      if (lookahead == ':') ADVANCE(53);
      END_STATE();
    case 11:
      if (lookahead == '_') ADVANCE(167);
      END_STATE();
    case 12:
      if (lookahead == '_') ADVANCE(81);
      END_STATE();
    case 13:
      if (lookahead == '_') ADVANCE(241);
      END_STATE();
    case 14:
      if (lookahead == '_') ADVANCE(282);
      if (lookahead == 'n') ADVANCE(257);
      END_STATE();
    case 15:
      if (lookahead == '_') ADVANCE(765);
      END_STATE();
    case 16:
      if (lookahead == '_') ADVANCE(110);
      END_STATE();
    case 17:
      if (lookahead == '_') ADVANCE(568);
      END_STATE();
    case 18:
      if (lookahead == '_') ADVANCE(284);
      END_STATE();
    case 19:
      if (lookahead == '_') ADVANCE(771);
      END_STATE();
    case 20:
      if (lookahead == '_') ADVANCE(109);
      END_STATE();
    case 21:
      if (lookahead == '_') ADVANCE(313);
      END_STATE();
    case 22:
      if (lookahead == '_') ADVANCE(576);
      END_STATE();
    case 23:
      if (lookahead == '_') ADVANCE(549);
      END_STATE();
    case 24:
      if (lookahead == '_') ADVANCE(141);
      END_STATE();
    case 25:
      if (lookahead == '_') ADVANCE(520);
      END_STATE();
    case 26:
      if (lookahead == '_') ADVANCE(87);
      END_STATE();
    case 27:
      if (lookahead == '_') ADVANCE(647);
      END_STATE();
    case 28:
      if (lookahead == '_') ADVANCE(656);
      END_STATE();
    case 29:
      if (lookahead == '_') ADVANCE(716);
      END_STATE();
    case 30:
      if (lookahead == '_') ADVANCE(762);
      END_STATE();
    case 31:
      if (lookahead == '_') ADVANCE(552);
      END_STATE();
    case 32:
      if (lookahead == '_') ADVANCE(721);
      END_STATE();
    case 33:
      if (lookahead == '_') ADVANCE(725);
      END_STATE();
    case 34:
      if (lookahead == '_') ADVANCE(90);
      END_STATE();
    case 35:
      if (lookahead == '_') ADVANCE(618);
      END_STATE();
    case 36:
      if (lookahead == '_') ADVANCE(659);
      END_STATE();
    case 37:
      if (lookahead == '_') ADVANCE(667);
      END_STATE();
    case 38:
      if (lookahead == '_') ADVANCE(145);
      END_STATE();
    case 39:
      if (lookahead == '_') ADVANCE(561);
      END_STATE();
    case 40:
      if (lookahead == '_') ADVANCE(115);
      END_STATE();
    case 41:
      if (lookahead == '_') ADVANCE(289);
      END_STATE();
    case 42:
      if (lookahead == '_') ADVANCE(577);
      END_STATE();
    case 43:
      if (lookahead == 'a') ADVANCE(844);
      if (lookahead == 'e') ADVANCE(847);
      if (lookahead == 'p') ADVANCE(845);
      if (lookahead == 'r') ADVANCE(846);
      if (lookahead == 's') ADVANCE(843);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(687);
      if (lookahead == 'o') ADVANCE(742);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(626);
      if (lookahead == 'i') ADVANCE(163);
      END_STATE();
    case 46:
      if (lookahead == 'a') ADVANCE(773);
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(773);
      if (lookahead == 'c') ADVANCE(514);
      if (lookahead == 'e') ADVANCE(672);
      if (lookahead == 'h') ADVANCE(516);
      if (lookahead == 't') ADVANCE(521);
      if (lookahead == 'u') ADVANCE(126);
      END_STATE();
    case 48:
      if (lookahead == 'a') ADVANCE(773);
      if (lookahead == 't') ADVANCE(558);
      END_STATE();
    case 49:
      if (lookahead == 'a') ADVANCE(933);
      if (lookahead == 'c') ADVANCE(935);
      if (lookahead == 's') ADVANCE(932);
      END_STATE();
    case 50:
      if (lookahead == 'a') ADVANCE(828);
      END_STATE();
    case 51:
      if (lookahead == 'a') ADVANCE(766);
      END_STATE();
    case 52:
      if (lookahead == 'a') ADVANCE(870);
      END_STATE();
    case 53:
      ADVANCE_MAP(
        'a', 578,
        'b', 99,
        'd', 249,
        'e', 275,
        'f', 262,
        'i', 467,
        'k', 491,
        'l', 537,
        'm', 268,
        'p', 553,
        'r', 234,
        's', 335,
        't', 323,
        'u', 507,
        'v', 82,
      );
      END_STATE();
    case 54:
      if (lookahead == 'a') ADVANCE(920);
      END_STATE();
    case 55:
      if (lookahead == 'a') ADVANCE(121);
      if (lookahead == 'e') ADVANCE(207);
      END_STATE();
    case 56:
      if (lookahead == 'a') ADVANCE(121);
      if (lookahead == 'e') ADVANCE(207);
      if (lookahead == 'l') ADVANCE(518);
      if (lookahead == 'n') ADVANCE(782);
      END_STATE();
    case 57:
      if (lookahead == 'a') ADVANCE(278);
      END_STATE();
    case 58:
      if (lookahead == 'a') ADVANCE(431);
      END_STATE();
    case 59:
      if (lookahead == 'a') ADVANCE(442);
      END_STATE();
    case 60:
      if (lookahead == 'a') ADVANCE(408);
      END_STATE();
    case 61:
      if (lookahead == 'a') ADVANCE(304);
      if (lookahead == 'e') ADVANCE(802);
      END_STATE();
    case 62:
      if (lookahead == 'a') ADVANCE(580);
      END_STATE();
    case 63:
      if (lookahead == 'a') ADVANCE(748);
      if (lookahead == 'i') ADVANCE(312);
      END_STATE();
    case 64:
      if (lookahead == 'a') ADVANCE(674);
      END_STATE();
    case 65:
      if (lookahead == 'a') ADVANCE(157);
      if (lookahead == 'l') ADVANCE(350);
      END_STATE();
    case 66:
      if (lookahead == 'a') ADVANCE(581);
      END_STATE();
    case 67:
      if (lookahead == 'a') ADVANCE(582);
      END_STATE();
    case 68:
      if (lookahead == 'a') ADVANCE(409);
      END_STATE();
    case 69:
      if (lookahead == 'a') ADVANCE(166);
      if (lookahead == 'q') ADVANCE(754);
      if (lookahead == 's') ADVANCE(239);
      END_STATE();
    case 70:
      if (lookahead == 'a') ADVANCE(26);
      END_STATE();
    case 71:
      if (lookahead == 'a') ADVANCE(666);
      if (lookahead == 'e') ADVANCE(85);
      if (lookahead == 'u') ADVANCE(493);
      END_STATE();
    case 72:
      if (lookahead == 'a') ADVANCE(403);
      END_STATE();
    case 73:
      if (lookahead == 'a') ADVANCE(130);
      END_STATE();
    case 74:
      if (lookahead == 'a') ADVANCE(432);
      END_STATE();
    case 75:
      if (lookahead == 'a') ADVANCE(697);
      if (lookahead == 'l') ADVANCE(517);
      if (lookahead == 'p') ADVANCE(237);
      if (lookahead == 't') ADVANCE(607);
      END_STATE();
    case 76:
      if (lookahead == 'a') ADVANCE(595);
      END_STATE();
    case 77:
      if (lookahead == 'a') ADVANCE(494);
      END_STATE();
    case 78:
      if (lookahead == 'a') ADVANCE(625);
      if (lookahead == 'i') ADVANCE(495);
      END_STATE();
    case 79:
      if (lookahead == 'a') ADVANCE(722);
      END_STATE();
    case 80:
      if (lookahead == 'a') ADVANCE(703);
      END_STATE();
    case 81:
      if (lookahead == 'a') ADVANCE(649);
      if (lookahead == 'p') ADVANCE(604);
      END_STATE();
    case 82:
      if (lookahead == 'a') ADVANCE(499);
      END_STATE();
    case 83:
      if (lookahead == 'a') ADVANCE(712);
      END_STATE();
    case 84:
      if (lookahead == 'a') ADVANCE(421);
      END_STATE();
    case 85:
      if (lookahead == 'a') ADVANCE(428);
      if (lookahead == 'r') ADVANCE(550);
      END_STATE();
    case 86:
      if (lookahead == 'a') ADVANCE(473);
      END_STATE();
    case 87:
      if (lookahead == 'a') ADVANCE(288);
      END_STATE();
    case 88:
      if (lookahead == 'a') ADVANCE(444);
      END_STATE();
    case 89:
      if (lookahead == 'a') ADVANCE(281);
      END_STATE();
    case 90:
      if (lookahead == 'a') ADVANCE(610);
      END_STATE();
    case 91:
      if (lookahead == 'a') ADVANCE(308);
      END_STATE();
    case 92:
      if (lookahead == 'a') ADVANCE(429);
      END_STATE();
    case 93:
      if (lookahead == 'a') ADVANCE(484);
      END_STATE();
    case 94:
      if (lookahead == 'a') ADVANCE(137);
      END_STATE();
    case 95:
      if (lookahead == 'a') ADVANCE(723);
      END_STATE();
    case 96:
      if (lookahead == 'a') ADVANCE(657);
      END_STATE();
    case 97:
      if (lookahead == 'a') ADVANCE(714);
      END_STATE();
    case 98:
      if (lookahead == 'a') ADVANCE(309);
      END_STATE();
    case 99:
      if (lookahead == 'a') ADVANCE(487);
      if (lookahead == 'i') ADVANCE(492);
      if (lookahead == 'l') ADVANCE(96);
      END_STATE();
    case 100:
      if (lookahead == 'a') ADVANCE(717);
      if (lookahead == 'e') ADVANCE(60);
      if (lookahead == 'i') ADVANCE(693);
      END_STATE();
    case 101:
      if (lookahead == 'a') ADVANCE(497);
      END_STATE();
    case 102:
      if (lookahead == 'a') ADVANCE(729);
      END_STATE();
    case 103:
      if (lookahead == 'a') ADVANCE(730);
      END_STATE();
    case 104:
      if (lookahead == 'a') ADVANCE(731);
      END_STATE();
    case 105:
      if (lookahead == 'a') ADVANCE(438);
      END_STATE();
    case 106:
      if (lookahead == 'a') ADVANCE(734);
      END_STATE();
    case 107:
      if (lookahead == 'b') ADVANCE(347);
      END_STATE();
    case 108:
      if (lookahead == 'b') ADVANCE(648);
      END_STATE();
    case 109:
      if (lookahead == 'b') ADVANCE(617);
      END_STATE();
    case 110:
      if (lookahead == 'b') ADVANCE(533);
      END_STATE();
    case 111:
      if (lookahead == 'b') ADVANCE(73);
      END_STATE();
    case 112:
      if (lookahead == 'b') ADVANCE(426);
      END_STATE();
    case 113:
      if (lookahead == 'b') ADVANCE(67);
      END_STATE();
    case 114:
      if (lookahead == 'b') ADVANCE(619);
      END_STATE();
    case 115:
      if (lookahead == 'b') ADVANCE(560);
      END_STATE();
    case 116:
      if (lookahead == 'c') ADVANCE(413);
      if (lookahead == 'e') ADVANCE(147);
      if (lookahead == 'g') ADVANCE(439);
      if (lookahead == 's') ADVANCE(46);
      if (lookahead == 't') ADVANCE(199);
      if (lookahead == 'x') ADVANCE(564);
      if (lookahead == '}') ADVANCE(786);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(785);
      END_STATE();
    case 117:
      if (lookahead == 'c') ADVANCE(331);
      END_STATE();
    case 118:
      if (lookahead == 'c') ADVANCE(319);
      END_STATE();
    case 119:
      if (lookahead == 'c') ADVANCE(319);
      if (lookahead == 't') ADVANCE(342);
      END_STATE();
    case 120:
      if (lookahead == 'c') ADVANCE(399);
      END_STATE();
    case 121:
      if (lookahead == 'c') ADVANCE(364);
      END_STATE();
    case 122:
      if (lookahead == 'c') ADVANCE(400);
      END_STATE();
    case 123:
      if (lookahead == 'c') ADVANCE(755);
      END_STATE();
    case 124:
      if (lookahead == 'c') ADVANCE(330);
      END_STATE();
    case 125:
      if (lookahead == 'c') ADVANCE(401);
      END_STATE();
    case 126:
      if (lookahead == 'c') ADVANCE(144);
      END_STATE();
    case 127:
      if (lookahead == 'c') ADVANCE(410);
      if (lookahead == 'r') ADVANCE(193);
      END_STATE();
    case 128:
      if (lookahead == 'c') ADVANCE(779);
      END_STATE();
    case 129:
      if (lookahead == 'c') ADVANCE(404);
      END_STATE();
    case 130:
      if (lookahead == 'c') ADVANCE(402);
      END_STATE();
    case 131:
      if (lookahead == 'c') ADVANCE(677);
      END_STATE();
    case 132:
      if (lookahead == 'c') ADVANCE(317);
      END_STATE();
    case 133:
      if (lookahead == 'c') ADVANCE(318);
      END_STATE();
    case 134:
      if (lookahead == 'c') ADVANCE(363);
      END_STATE();
    case 135:
      if (lookahead == 'c') ADVANCE(187);
      END_STATE();
    case 136:
      if (lookahead == 'c') ADVANCE(711);
      END_STATE();
    case 137:
      if (lookahead == 'c') ADVANCE(188);
      END_STATE();
    case 138:
      if (lookahead == 'c') ADVANCE(685);
      END_STATE();
    case 139:
      if (lookahead == 'c') ADVANCE(191);
      END_STATE();
    case 140:
      if (lookahead == 'c') ADVANCE(596);
      END_STATE();
    case 141:
      if (lookahead == 'c') ADVANCE(752);
      END_STATE();
    case 142:
      if (lookahead == 'c') ADVANCE(614);
      END_STATE();
    case 143:
      if (lookahead == 'c') ADVANCE(97);
      END_STATE();
    case 144:
      if (lookahead == 'c') ADVANCE(254);
      END_STATE();
    case 145:
      if (lookahead == 'c') ADVANCE(759);
      END_STATE();
    case 146:
      if (lookahead == 'c') ADVANCE(434);
      if (lookahead == 'f') ADVANCE(283);
      if (lookahead == 'n') ADVANCE(119);
      if (lookahead == 'x') ADVANCE(210);
      if (lookahead == 'y') ADVANCE(206);
      END_STATE();
    case 147:
      if (lookahead == 'c') ADVANCE(434);
      if (lookahead == 'f') ADVANCE(283);
      if (lookahead == 'n') ADVANCE(118);
      if (lookahead == 'x') ADVANCE(210);
      END_STATE();
    case 148:
      if (lookahead == 'c') ADVANCE(434);
      if (lookahead == 'f') ADVANCE(283);
      if (lookahead == 'n') ADVANCE(118);
      if (lookahead == 'x') ADVANCE(210);
      if (lookahead == 'y') ADVANCE(206);
      END_STATE();
    case 149:
      if (lookahead == 'c') ADVANCE(733);
      END_STATE();
    case 150:
      if (lookahead == 'c') ADVANCE(735);
      END_STATE();
    case 151:
      if (lookahead == 'c') ADVANCE(736);
      END_STATE();
    case 152:
      if (lookahead == 'c') ADVANCE(737);
      END_STATE();
    case 153:
      if (lookahead == 'd') ADVANCE(827);
      END_STATE();
    case 154:
      if (lookahead == 'd') ADVANCE(799);
      END_STATE();
    case 155:
      if (lookahead == 'd') ADVANCE(791);
      END_STATE();
    case 156:
      if (lookahead == 'd') ADVANCE(798);
      END_STATE();
    case 157:
      if (lookahead == 'd') ADVANCE(23);
      END_STATE();
    case 158:
      if (lookahead == 'd') ADVANCE(862);
      END_STATE();
    case 159:
      if (lookahead == 'd') ADVANCE(851);
      END_STATE();
    case 160:
      if (lookahead == 'd') ADVANCE(924);
      END_STATE();
    case 161:
      if (lookahead == 'd') ADVANCE(818);
      END_STATE();
    case 162:
      if (lookahead == 'd') ADVANCE(816);
      END_STATE();
    case 163:
      if (lookahead == 'd') ADVANCE(490);
      if (lookahead == 'n') ADVANCE(213);
      END_STATE();
    case 164:
      if (lookahead == 'd') ADVANCE(345);
      END_STATE();
    case 165:
      if (lookahead == 'd') ADVANCE(756);
      END_STATE();
    case 166:
      if (lookahead == 'd') ADVANCE(159);
      END_STATE();
    case 167:
      if (lookahead == 'd') ADVANCE(59);
      if (lookahead == 'h') ADVANCE(253);
      END_STATE();
    case 168:
      if (lookahead == 'd') ADVANCE(638);
      END_STATE();
    case 169:
      if (lookahead == 'd') ADVANCE(235);
      END_STATE();
    case 170:
      if (lookahead == 'd') ADVANCE(380);
      END_STATE();
    case 171:
      if (lookahead == 'd') ADVANCE(384);
      END_STATE();
    case 172:
      if (lookahead == 'd') ADVANCE(505);
      END_STATE();
    case 173:
      if (lookahead == 'e') ADVANCE(164);
      END_STATE();
    case 174:
      if (lookahead == 'e') ADVANCE(590);
      END_STATE();
    case 175:
      if (lookahead == 'e') ADVANCE(805);
      END_STATE();
    case 176:
      if (lookahead == 'e') ADVANCE(860);
      END_STATE();
    case 177:
      if (lookahead == 'e') ADVANCE(929);
      END_STATE();
    case 178:
      if (lookahead == 'e') ADVANCE(831);
      END_STATE();
    case 179:
      if (lookahead == 'e') ADVANCE(802);
      END_STATE();
    case 180:
      if (lookahead == 'e') ADVANCE(839);
      END_STATE();
    case 181:
      if (lookahead == 'e') ADVANCE(810);
      END_STATE();
    case 182:
      if (lookahead == 'e') ADVANCE(787);
      END_STATE();
    case 183:
      if (lookahead == 'e') ADVANCE(829);
      END_STATE();
    case 184:
      if (lookahead == 'e') ADVANCE(830);
      END_STATE();
    case 185:
      if (lookahead == 'e') ADVANCE(306);
      END_STATE();
    case 186:
      if (lookahead == 'e') ADVANCE(864);
      END_STATE();
    case 187:
      if (lookahead == 'e') ADVANCE(872);
      END_STATE();
    case 188:
      if (lookahead == 'e') ADVANCE(891);
      END_STATE();
    case 189:
      if (lookahead == 'e') ADVANCE(868);
      END_STATE();
    case 190:
      if (lookahead == 'e') ADVANCE(865);
      END_STATE();
    case 191:
      if (lookahead == 'e') ADVANCE(873);
      END_STATE();
    case 192:
      if (lookahead == 'e') ADVANCE(893);
      END_STATE();
    case 193:
      if (lookahead == 'e') ADVANCE(921);
      END_STATE();
    case 194:
      if (lookahead == 'e') ADVANCE(918);
      END_STATE();
    case 195:
      if (lookahead == 'e') ADVANCE(906);
      END_STATE();
    case 196:
      if (lookahead == 'e') ADVANCE(915);
      END_STATE();
    case 197:
      if (lookahead == 'e') ADVANCE(904);
      END_STATE();
    case 198:
      if (lookahead == 'e') ADVANCE(923);
      END_STATE();
    case 199:
      if (lookahead == 'e') ADVANCE(415);
      if (lookahead == 'i') ADVANCE(441);
      END_STATE();
    case 200:
      if (lookahead == 'e') ADVANCE(62);
      END_STATE();
    case 201:
      if (lookahead == 'e') ADVANCE(640);
      if (lookahead == 'o') ADVANCE(691);
      if (lookahead == 'u') ADVANCE(445);
      END_STATE();
    case 202:
      if (lookahead == 'e') ADVANCE(764);
      if (lookahead == 'u') ADVANCE(122);
      END_STATE();
    case 203:
      if (lookahead == 'e') ADVANCE(153);
      END_STATE();
    case 204:
      if (lookahead == 'e') ADVANCE(35);
      END_STATE();
    case 205:
      if (lookahead == 'e') ADVANCE(154);
      END_STATE();
    case 206:
      if (lookahead == 'e') ADVANCE(627);
      END_STATE();
    case 207:
      if (lookahead == 'e') ADVANCE(673);
      END_STATE();
    case 208:
      if (lookahead == 'e') ADVANCE(155);
      END_STATE();
    case 209:
      if (lookahead == 'e') ADVANCE(416);
      END_STATE();
    case 210:
      if (lookahead == 'e') ADVANCE(123);
      END_STATE();
    case 211:
      if (lookahead == 'e') ADVANCE(156);
      END_STATE();
    case 212:
      if (lookahead == 'e') ADVANCE(131);
      END_STATE();
    case 213:
      if (lookahead == 'e') ADVANCE(140);
      END_STATE();
    case 214:
      if (lookahead == 'e') ADVANCE(631);
      END_STATE();
    case 215:
      if (lookahead == 'e') ADVANCE(574);
      END_STATE();
    case 216:
      if (lookahead == 'e') ADVANCE(158);
      END_STATE();
    case 217:
      if (lookahead == 'e') ADVANCE(52);
      END_STATE();
    case 218:
      if (lookahead == 'e') ADVANCE(613);
      END_STATE();
    case 219:
      if (lookahead == 'e') ADVANCE(12);
      END_STATE();
    case 220:
      if (lookahead == 'e') ADVANCE(160);
      END_STATE();
    case 221:
      if (lookahead == 'e') ADVANCE(583);
      END_STATE();
    case 222:
      if (lookahead == 'e') ADVANCE(584);
      END_STATE();
    case 223:
      if (lookahead == 'e') ADVANCE(585);
      END_STATE();
    case 224:
      if (lookahead == 'e') ADVANCE(450);
      END_STATE();
    case 225:
      if (lookahead == 'e') ADVANCE(671);
      END_STATE();
    case 226:
      if (lookahead == 'e') ADVANCE(586);
      END_STATE();
    case 227:
      if (lookahead == 'e') ADVANCE(37);
      END_STATE();
    case 228:
      if (lookahead == 'e') ADVANCE(68);
      END_STATE();
    case 229:
      if (lookahead == 'e') ADVANCE(603);
      END_STATE();
    case 230:
      if (lookahead == 'e') ADVANCE(13);
      END_STATE();
    case 231:
      if (lookahead == 'e') ADVANCE(72);
      END_STATE();
    case 232:
      if (lookahead == 'e') ADVANCE(54);
      END_STATE();
    case 233:
      if (lookahead == 'e') ADVANCE(587);
      END_STATE();
    case 234:
      if (lookahead == 'e') ADVANCE(642);
      END_STATE();
    case 235:
      if (lookahead == 'e') ADVANCE(588);
      END_STATE();
    case 236:
      if (lookahead == 'e') ADVANCE(593);
      END_STATE();
    case 237:
      if (lookahead == 'e') ADVANCE(216);
      END_STATE();
    case 238:
      if (lookahead == 'e') ADVANCE(589);
      END_STATE();
    case 239:
      if (lookahead == 'e') ADVANCE(684);
      END_STATE();
    case 240:
      if (lookahead == 'e') ADVANCE(476);
      END_STATE();
    case 241:
      if (lookahead == 'e') ADVANCE(478);
      if (lookahead == 'n') ADVANCE(263);
      END_STATE();
    case 242:
      if (lookahead == 'e') ADVANCE(215);
      if (lookahead == 'i') ADVANCE(280);
      END_STATE();
    case 243:
      if (lookahead == 'e') ADVANCE(220);
      END_STATE();
    case 244:
      if (lookahead == 'e') ADVANCE(643);
      END_STATE();
    case 245:
      if (lookahead == 'e') ADVANCE(761);
      END_STATE();
    case 246:
      if (lookahead == 'e') ADVANCE(761);
      if (lookahead == 'o') ADVANCE(461);
      END_STATE();
    case 247:
      if (lookahead == 'e') ADVANCE(592);
      END_STATE();
    case 248:
      if (lookahead == 'e') ADVANCE(66);
      END_STATE();
    case 249:
      if (lookahead == 'e') ADVANCE(570);
      END_STATE();
    case 250:
      if (lookahead == 'e') ADVANCE(30);
      END_STATE();
    case 251:
      if (lookahead == 'e') ADVANCE(136);
      END_STATE();
    case 252:
      if (lookahead == 'e') ADVANCE(486);
      END_STATE();
    case 253:
      if (lookahead == 'e') ADVANCE(92);
      END_STATE();
    case 254:
      if (lookahead == 'e') ADVANCE(645);
      END_STATE();
    case 255:
      if (lookahead == 'e') ADVANCE(31);
      END_STATE();
    case 256:
      if (lookahead == 'e') ADVANCE(149);
      END_STATE();
    case 257:
      if (lookahead == 'e') ADVANCE(650);
      END_STATE();
    case 258:
      if (lookahead == 'e') ADVANCE(80);
      END_STATE();
    case 259:
      if (lookahead == 'e') ADVANCE(138);
      END_STATE();
    case 260:
      if (lookahead == 'e') ADVANCE(651);
      END_STATE();
    case 261:
      if (lookahead == 'e') ADVANCE(464);
      END_STATE();
    case 262:
      if (lookahead == 'e') ADVANCE(95);
      if (lookahead == 'i') ADVANCE(612);
      if (lookahead == 'l') ADVANCE(88);
      if (lookahead == 'o') ADVANCE(609);
      if (lookahead == 'r') ADVANCE(557);
      END_STATE();
    case 263:
      if (lookahead == 'e') ADVANCE(726);
      END_STATE();
    case 264:
      if (lookahead == 'e') ADVANCE(652);
      END_STATE();
    case 265:
      if (lookahead == 'e') ADVANCE(655);
      END_STATE();
    case 266:
      if (lookahead == 'e') ADVANCE(142);
      END_STATE();
    case 267:
      if (lookahead == 'e') ADVANCE(621);
      END_STATE();
    case 268:
      if (lookahead == 'e') ADVANCE(508);
      END_STATE();
    case 269:
      if (lookahead == 'e') ADVANCE(150);
      END_STATE();
    case 270:
      if (lookahead == 'e') ADVANCE(151);
      END_STATE();
    case 271:
      if (lookahead == 'e') ADVANCE(152);
      END_STATE();
    case 272:
      if (lookahead == 'e') ADVANCE(42);
      END_STATE();
    case 273:
      if (lookahead == 'f') ADVANCE(800);
      if (lookahead == 'n') ADVANCE(797);
      END_STATE();
    case 274:
      if (lookahead == 'f') ADVANCE(800);
      if (lookahead == 'n') ADVANCE(796);
      END_STATE();
    case 275:
      if (lookahead == 'f') ADVANCE(277);
      END_STATE();
    case 276:
      if (lookahead == 'f') ADVANCE(32);
      END_STATE();
    case 277:
      if (lookahead == 'f') ADVANCE(365);
      END_STATE();
    case 278:
      if (lookahead == 'f') ADVANCE(681);
      END_STATE();
    case 279:
      if (lookahead == 'f') ADVANCE(34);
      END_STATE();
    case 280:
      if (lookahead == 'f') ADVANCE(724);
      END_STATE();
    case 281:
      if (lookahead == 'f') ADVANCE(686);
      END_STATE();
    case 282:
      if (lookahead == 'f') ADVANCE(84);
      END_STATE();
    case 283:
      if (lookahead == 'f') ADVANCE(212);
      END_STATE();
    case 284:
      if (lookahead == 'f') ADVANCE(79);
      END_STATE();
    case 285:
      if (lookahead == 'f') ADVANCE(360);
      END_STATE();
    case 286:
      if (lookahead == 'f') ADVANCE(33);
      END_STATE();
    case 287:
      if (lookahead == 'f') ADVANCE(393);
      END_STATE();
    case 288:
      if (lookahead == 'f') ADVANCE(287);
      END_STATE();
    case 289:
      if (lookahead == 'f') ADVANCE(105);
      END_STATE();
    case 290:
      if (lookahead == 'g') ADVANCE(808);
      END_STATE();
    case 291:
      if (lookahead == 'g') ADVANCE(794);
      END_STATE();
    case 292:
      if (lookahead == 'g') ADVANCE(885);
      END_STATE();
    case 293:
      if (lookahead == 'g') ADVANCE(889);
      END_STATE();
    case 294:
      if (lookahead == 'g') ADVANCE(874);
      END_STATE();
    case 295:
      if (lookahead == 'g') ADVANCE(910);
      END_STATE();
    case 296:
      if (lookahead == 'g') ADVANCE(922);
      END_STATE();
    case 297:
      if (lookahead == 'g') ADVANCE(911);
      END_STATE();
    case 298:
      if (lookahead == 'g') ADVANCE(914);
      END_STATE();
    case 299:
      if (lookahead == 'g') ADVANCE(896);
      END_STATE();
    case 300:
      if (lookahead == 'g') ADVANCE(320);
      END_STATE();
    case 301:
      if (lookahead == 'g') ADVANCE(18);
      END_STATE();
    case 302:
      if (lookahead == 'g') ADVANCE(447);
      END_STATE();
    case 303:
      if (lookahead == 'g') ADVANCE(24);
      END_STATE();
    case 304:
      if (lookahead == 'g') ADVANCE(183);
      END_STATE();
    case 305:
      if (lookahead == 'g') ADVANCE(701);
      END_STATE();
    case 306:
      if (lookahead == 'g') ADVANCE(252);
      if (lookahead == 's') ADVANCE(355);
      END_STATE();
    case 307:
      if (lookahead == 'g') ADVANCE(221);
      END_STATE();
    case 308:
      if (lookahead == 'g') ADVANCE(189);
      END_STATE();
    case 309:
      if (lookahead == 'g') ADVANCE(192);
      END_STATE();
    case 310:
      if (lookahead == 'g') ADVANCE(321);
      END_STATE();
    case 311:
      if (lookahead == 'g') ADVANCE(753);
      END_STATE();
    case 312:
      if (lookahead == 'g') ADVANCE(322);
      END_STATE();
    case 313:
      if (lookahead == 'g') ADVANCE(598);
      END_STATE();
    case 314:
      if (lookahead == 'g') ADVANCE(38);
      END_STATE();
    case 315:
      if (lookahead == 'h') ADVANCE(866);
      END_STATE();
    case 316:
      if (lookahead == 'h') ADVANCE(867);
      END_STATE();
    case 317:
      if (lookahead == 'h') ADVANCE(917);
      END_STATE();
    case 318:
      if (lookahead == 'h') ADVANCE(913);
      END_STATE();
    case 319:
      if (lookahead == 'h') ADVANCE(86);
      END_STATE();
    case 320:
      if (lookahead == 'h') ADVANCE(675);
      END_STATE();
    case 321:
      if (lookahead == 'h') ADVANCE(680);
      END_STATE();
    case 322:
      if (lookahead == 'h') ADVANCE(690);
      END_STATE();
    case 323:
      if (lookahead == 'h') ADVANCE(534);
      END_STATE();
    case 324:
      if (lookahead == 'h') ADVANCE(222);
      END_STATE();
    case 325:
      if (lookahead == 'h') ADVANCE(250);
      END_STATE();
    case 326:
      if (lookahead == 'h') ADVANCE(229);
      END_STATE();
    case 327:
      if (lookahead == 'h') ADVANCE(227);
      END_STATE();
    case 328:
      if (lookahead == 'h') ADVANCE(230);
      END_STATE();
    case 329:
      if (lookahead == 'h') ADVANCE(238);
      END_STATE();
    case 330:
      if (lookahead == 'h') ADVANCE(214);
      END_STATE();
    case 331:
      if (lookahead == 'h') ADVANCE(551);
      END_STATE();
    case 332:
      if (lookahead == 'h') ADVANCE(36);
      END_STATE();
    case 333:
      if (lookahead == 'h') ADVANCE(605);
      END_STATE();
    case 334:
      if (lookahead == 'h') ADVANCE(354);
      END_STATE();
    case 335:
      if (lookahead == 'h') ADVANCE(76);
      if (lookahead == 'i') ADVANCE(418);
      if (lookahead == 'm') ADVANCE(375);
      if (lookahead == 'o') ADVANCE(749);
      if (lookahead == 'w') ADVANCE(242);
      END_STATE();
    case 336:
      if (lookahead == 'h') ADVANCE(378);
      END_STATE();
    case 337:
      if (lookahead == 'h') ADVANCE(396);
      END_STATE();
    case 338:
      if (lookahead == 'h') ADVANCE(40);
      END_STATE();
    case 339:
      if (lookahead == 'i') ADVANCE(760);
      if (lookahead == 'm') ADVANCE(49);
      END_STATE();
    case 340:
      if (lookahead == 'i') ADVANCE(300);
      if (lookahead == 'o') ADVANCE(523);
      END_STATE();
    case 341:
      if (lookahead == 'i') ADVANCE(302);
      END_STATE();
    case 342:
      if (lookahead == 'i') ADVANCE(688);
      END_STATE();
    case 343:
      if (lookahead == 'i') ADVANCE(107);
      END_STATE();
    case 344:
      if (lookahead == 'i') ADVANCE(466);
      if (lookahead == 's') ADVANCE(346);
      END_STATE();
    case 345:
      if (lookahead == 'i') ADVANCE(143);
      END_STATE();
    case 346:
      if (lookahead == 'i') ADVANCE(692);
      END_STATE();
    case 347:
      if (lookahead == 'i') ADVANCE(433);
      END_STATE();
    case 348:
      if (lookahead == 'i') ADVANCE(311);
      END_STATE();
    case 349:
      if (lookahead == 'i') ADVANCE(502);
      END_STATE();
    case 350:
      if (lookahead == 'i') ADVANCE(469);
      END_STATE();
    case 351:
      if (lookahead == 'i') ADVANCE(665);
      END_STATE();
    case 352:
      if (lookahead == 'i') ADVANCE(169);
      END_STATE();
    case 353:
      if (lookahead == 'i') ADVANCE(644);
      END_STATE();
    case 354:
      if (lookahead == 'i') ADVANCE(479);
      END_STATE();
    case 355:
      if (lookahead == 'i') ADVANCE(664);
      END_STATE();
    case 356:
      if (lookahead == 'i') ADVANCE(696);
      END_STATE();
    case 357:
      if (lookahead == 'i') ADVANCE(668);
      END_STATE();
    case 358:
      if (lookahead == 'i') ADVANCE(669);
      END_STATE();
    case 359:
      if (lookahead == 'i') ADVANCE(689);
      END_STATE();
    case 360:
      if (lookahead == 'i') ADVANCE(500);
      END_STATE();
    case 361:
      if (lookahead == 'i') ADVANCE(695);
      END_STATE();
    case 362:
      if (lookahead == 'i') ADVANCE(700);
      END_STATE();
    case 363:
      if (lookahead == 'i') ADVANCE(240);
      END_STATE();
    case 364:
      if (lookahead == 'i') ADVANCE(462);
      END_STATE();
    case 365:
      if (lookahead == 'i') ADVANCE(134);
      END_STATE();
    case 366:
      if (lookahead == 'i') ADVANCE(525);
      END_STATE();
    case 367:
      if (lookahead == 'i') ADVANCE(423);
      END_STATE();
    case 368:
      if (lookahead == 'i') ADVANCE(720);
      END_STATE();
    case 369:
      if (lookahead == 'i') ADVANCE(606);
      END_STATE();
    case 370:
      if (lookahead == 'i') ADVANCE(430);
      END_STATE();
    case 371:
      if (lookahead == 'i') ADVANCE(463);
      END_STATE();
    case 372:
      if (lookahead == 'i') ADVANCE(535);
      END_STATE();
    case 373:
      if (lookahead == 'i') ADVANCE(468);
      END_STATE();
    case 374:
      if (lookahead == 'i') ADVANCE(536);
      END_STATE();
    case 375:
      if (lookahead == 'i') ADVANCE(719);
      END_STATE();
    case 376:
      if (lookahead == 'i') ADVANCE(471);
      END_STATE();
    case 377:
      if (lookahead == 'i') ADVANCE(538);
      END_STATE();
    case 378:
      if (lookahead == 'i') ADVANCE(472);
      END_STATE();
    case 379:
      if (lookahead == 'i') ADVANCE(539);
      END_STATE();
    case 380:
      if (lookahead == 'i') ADVANCE(474);
      END_STATE();
    case 381:
      if (lookahead == 'i') ADVANCE(541);
      END_STATE();
    case 382:
      if (lookahead == 'i') ADVANCE(475);
      END_STATE();
    case 383:
      if (lookahead == 'i') ADVANCE(544);
      END_STATE();
    case 384:
      if (lookahead == 'i') ADVANCE(477);
      END_STATE();
    case 385:
      if (lookahead == 'i') ADVANCE(545);
      END_STATE();
    case 386:
      if (lookahead == 'i') ADVANCE(480);
      END_STATE();
    case 387:
      if (lookahead == 'i') ADVANCE(546);
      END_STATE();
    case 388:
      if (lookahead == 'i') ADVANCE(481);
      END_STATE();
    case 389:
      if (lookahead == 'i') ADVANCE(547);
      END_STATE();
    case 390:
      if (lookahead == 'i') ADVANCE(482);
      END_STATE();
    case 391:
      if (lookahead == 'i') ADVANCE(548);
      END_STATE();
    case 392:
      if (lookahead == 'i') ADVANCE(310);
      END_STATE();
    case 393:
      if (lookahead == 'i') ADVANCE(501);
      END_STATE();
    case 394:
      if (lookahead == 'i') ADVANCE(503);
      END_STATE();
    case 395:
      if (lookahead == 'i') ADVANCE(670);
      END_STATE();
    case 396:
      if (lookahead == 'i') ADVANCE(509);
      END_STATE();
    case 397:
      if (lookahead == 'i') ADVANCE(622);
      END_STATE();
    case 398:
      if (lookahead == 'j') ADVANCE(251);
      if (lookahead == 't') ADVANCE(256);
      END_STATE();
    case 399:
      if (lookahead == 'k') ADVANCE(819);
      END_STATE();
    case 400:
      if (lookahead == 'k') ADVANCE(887);
      END_STATE();
    case 401:
      if (lookahead == 'k') ADVANCE(888);
      END_STATE();
    case 402:
      if (lookahead == 'k') ADVANCE(908);
      END_STATE();
    case 403:
      if (lookahead == 'k') ADVANCE(925);
      END_STATE();
    case 404:
      if (lookahead == 'k') ADVANCE(111);
      END_STATE();
    case 405:
      if (lookahead == 'k') ADVANCE(203);
      END_STATE();
    case 406:
      if (lookahead == 'k') ADVANCE(233);
      END_STATE();
    case 407:
      if (lookahead == 'k') ADVANCE(29);
      END_STATE();
    case 408:
      if (lookahead == 'k') ADVANCE(504);
      END_STATE();
    case 409:
      if (lookahead == 'k') ADVANCE(388);
      END_STATE();
    case 410:
      if (lookahead == 'k') ADVANCE(39);
      END_STATE();
    case 411:
      if (lookahead == 'l') ADVANCE(341);
      if (lookahead == 'n') ADVANCE(117);
      if (lookahead == 's') ADVANCE(788);
      if (lookahead == 't') ADVANCE(789);
      END_STATE();
    case 412:
      if (lookahead == 'l') ADVANCE(519);
      if (lookahead == 'o') ADVANCE(641);
      if (lookahead == 'y') ADVANCE(698);
      END_STATE();
    case 413:
      if (lookahead == 'l') ADVANCE(200);
      END_STATE();
    case 414:
      if (lookahead == 'l') ADVANCE(566);
      END_STATE();
    case 415:
      if (lookahead == 'l') ADVANCE(417);
      END_STATE();
    case 416:
      if (lookahead == 'l') ADVANCE(628);
      END_STATE();
    case 417:
      if (lookahead == 'l') ADVANCE(594);
      END_STATE();
    case 418:
      if (lookahead == 'l') ADVANCE(407);
      END_STATE();
    case 419:
      if (lookahead == 'l') ADVANCE(747);
      END_STATE();
    case 420:
      if (lookahead == 'l') ADVANCE(515);
      END_STATE();
    case 421:
      if (lookahead == 'l') ADVANCE(436);
      END_STATE();
    case 422:
      if (lookahead == 'l') ADVANCE(244);
      END_STATE();
    case 423:
      if (lookahead == 'l') ADVANCE(435);
      END_STATE();
    case 424:
      if (lookahead == 'l') ADVANCE(678);
      END_STATE();
    case 425:
      if (lookahead == 'l') ADVANCE(27);
      END_STATE();
    case 426:
      if (lookahead == 'l') ADVANCE(181);
      END_STATE();
    case 427:
      if (lookahead == 'l') ADVANCE(162);
      END_STATE();
    case 428:
      if (lookahead == 'l') ADVANCE(699);
      END_STATE();
    case 429:
      if (lookahead == 'l') ADVANCE(704);
      END_STATE();
    case 430:
      if (lookahead == 'l') ADVANCE(272);
      END_STATE();
    case 431:
      if (lookahead == 'l') ADVANCE(750);
      END_STATE();
    case 432:
      if (lookahead == 'l') ADVANCE(406);
      END_STATE();
    case 433:
      if (lookahead == 'l') ADVANCE(359);
      END_STATE();
    case 434:
      if (lookahead == 'l') ADVANCE(248);
      END_STATE();
    case 435:
      if (lookahead == 'l') ADVANCE(98);
      END_STATE();
    case 436:
      if (lookahead == 'l') ADVANCE(376);
      END_STATE();
    case 437:
      if (lookahead == 'l') ADVANCE(390);
      END_STATE();
    case 438:
      if (lookahead == 'l') ADVANCE(437);
      END_STATE();
    case 439:
      if (lookahead == 'm') ADVANCE(49);
      END_STATE();
    case 440:
      if (lookahead == 'm') ADVANCE(565);
      END_STATE();
    case 441:
      if (lookahead == 'm') ADVANCE(177);
      END_STATE();
    case 442:
      if (lookahead == 'm') ADVANCE(91);
      END_STATE();
    case 443:
      if (lookahead == 'm') ADVANCE(224);
      END_STATE();
    case 444:
      if (lookahead == 'm') ADVANCE(194);
      END_STATE();
    case 445:
      if (lookahead == 'n') ADVANCE(811);
      END_STATE();
    case 446:
      if (lookahead == 'n') ADVANCE(938);
      END_STATE();
    case 447:
      if (lookahead == 'n') ADVANCE(790);
      END_STATE();
    case 448:
      if (lookahead == 'n') ADVANCE(663);
      END_STATE();
    case 449:
      if (lookahead == 'n') ADVANCE(880);
      END_STATE();
    case 450:
      if (lookahead == 'n') ADVANCE(892);
      END_STATE();
    case 451:
      if (lookahead == 'n') ADVANCE(883);
      END_STATE();
    case 452:
      if (lookahead == 'n') ADVANCE(886);
      END_STATE();
    case 453:
      if (lookahead == 'n') ADVANCE(884);
      END_STATE();
    case 454:
      if (lookahead == 'n') ADVANCE(877);
      END_STATE();
    case 455:
      if (lookahead == 'n') ADVANCE(871);
      END_STATE();
    case 456:
      if (lookahead == 'n') ADVANCE(894);
      END_STATE();
    case 457:
      if (lookahead == 'n') ADVANCE(899);
      END_STATE();
    case 458:
      if (lookahead == 'n') ADVANCE(895);
      END_STATE();
    case 459:
      if (lookahead == 'n') ADVANCE(897);
      END_STATE();
    case 460:
      if (lookahead == 'n') ADVANCE(898);
      END_STATE();
    case 461:
      if (lookahead == 'n') ADVANCE(290);
      END_STATE();
    case 462:
      if (lookahead == 'n') ADVANCE(291);
      END_STATE();
    case 463:
      if (lookahead == 'n') ADVANCE(301);
      END_STATE();
    case 464:
      if (lookahead == 'n') ADVANCE(305);
      END_STATE();
    case 465:
      if (lookahead == 'n') ADVANCE(165);
      END_STATE();
    case 466:
      if (lookahead == 'n') ADVANCE(694);
      END_STATE();
    case 467:
      if (lookahead == 'n') ADVANCE(285);
      END_STATE();
    case 468:
      if (lookahead == 'n') ADVANCE(292);
      END_STATE();
    case 469:
      if (lookahead == 'n') ADVANCE(172);
      END_STATE();
    case 470:
      if (lookahead == 'n') ADVANCE(419);
      END_STATE();
    case 471:
      if (lookahead == 'n') ADVANCE(293);
      END_STATE();
    case 472:
      if (lookahead == 'n') ADVANCE(294);
      END_STATE();
    case 473:
      if (lookahead == 'n') ADVANCE(679);
      END_STATE();
    case 474:
      if (lookahead == 'n') ADVANCE(303);
      END_STATE();
    case 475:
      if (lookahead == 'n') ADVANCE(295);
      END_STATE();
    case 476:
      if (lookahead == 'n') ADVANCE(128);
      END_STATE();
    case 477:
      if (lookahead == 'n') ADVANCE(296);
      END_STATE();
    case 478:
      if (lookahead == 'n') ADVANCE(161);
      END_STATE();
    case 479:
      if (lookahead == 'n') ADVANCE(646);
      END_STATE();
    case 480:
      if (lookahead == 'n') ADVANCE(297);
      END_STATE();
    case 481:
      if (lookahead == 'n') ADVANCE(298);
      END_STATE();
    case 482:
      if (lookahead == 'n') ADVANCE(299);
      END_STATE();
    case 483:
      if (lookahead == 'n') ADVANCE(211);
      END_STATE();
    case 484:
      if (lookahead == 'n') ADVANCE(702);
      END_STATE();
    case 485:
      if (lookahead == 'n') ADVANCE(636);
      END_STATE();
    case 486:
      if (lookahead == 'n') ADVANCE(267);
      END_STATE();
    case 487:
      if (lookahead == 'n') ADVANCE(255);
      END_STATE();
    case 488:
      if (lookahead == 'n') ADVANCE(196);
      END_STATE();
    case 489:
      if (lookahead == 'n') ADVANCE(422);
      END_STATE();
    case 490:
      if (lookahead == 'n') ADVANCE(392);
      END_STATE();
    case 491:
      if (lookahead == 'n') ADVANCE(530);
      END_STATE();
    case 492:
      if (lookahead == 'n') ADVANCE(170);
      END_STATE();
    case 493:
      if (lookahead == 'n') ADVANCE(307);
      END_STATE();
    case 494:
      if (lookahead == 'n') ADVANCE(135);
      END_STATE();
    case 495:
      if (lookahead == 'n') ADVANCE(213);
      END_STATE();
    case 496:
      if (lookahead == 'n') ADVANCE(132);
      END_STATE();
    case 497:
      if (lookahead == 'n') ADVANCE(139);
      END_STATE();
    case 498:
      if (lookahead == 'n') ADVANCE(231);
      END_STATE();
    case 499:
      if (lookahead == 'n') ADVANCE(358);
      END_STATE();
    case 500:
      if (lookahead == 'n') ADVANCE(361);
      END_STATE();
    case 501:
      if (lookahead == 'n') ADVANCE(362);
      END_STATE();
    case 502:
      if (lookahead == 'n') ADVANCE(371);
      END_STATE();
    case 503:
      if (lookahead == 'n') ADVANCE(266);
      END_STATE();
    case 504:
      if (lookahead == 'n') ADVANCE(260);
      END_STATE();
    case 505:
      if (lookahead == 'n') ADVANCE(264);
      END_STATE();
    case 506:
      if (lookahead == 'n') ADVANCE(265);
      END_STATE();
    case 507:
      if (lookahead == 'n') ADVANCE(114);
      END_STATE();
    case 508:
      if (lookahead == 'n') ADVANCE(171);
      END_STATE();
    case 509:
      if (lookahead == 'n') ADVANCE(314);
      END_STATE();
    case 510:
      if (lookahead == 'o') ADVANCE(344);
      END_STATE();
    case 511:
      if (lookahead == 'o') ADVANCE(344);
      if (lookahead == 'r') ADVANCE(173);
      END_STATE();
    case 512:
      if (lookahead == 'o') ADVANCE(398);
      END_STATE();
    case 513:
      if (lookahead == 'o') ADVANCE(691);
      if (lookahead == 'u') ADVANCE(445);
      END_STATE();
    case 514:
      if (lookahead == 'o') ADVANCE(599);
      END_STATE();
    case 515:
      if (lookahead == 'o') ADVANCE(772);
      END_STATE();
    case 516:
      if (lookahead == 'o') ADVANCE(597);
      END_STATE();
    case 517:
      if (lookahead == 'o') ADVANCE(767);
      END_STATE();
    case 518:
      if (lookahead == 'o') ADVANCE(64);
      END_STATE();
    case 519:
      if (lookahead == 'o') ADVANCE(120);
      END_STATE();
    case 520:
      if (lookahead == 'o') ADVANCE(276);
      END_STATE();
    case 521:
      if (lookahead == 'o') ADVANCE(579);
      END_STATE();
    case 522:
      if (lookahead == 'o') ADVANCE(769);
      END_STATE();
    case 523:
      if (lookahead == 'o') ADVANCE(446);
      END_STATE();
    case 524:
      if (lookahead == 'o') ADVANCE(414);
      END_STATE();
    case 525:
      if (lookahead == 'o') ADVANCE(483);
      END_STATE();
    case 526:
      if (lookahead == 'o') ADVANCE(351);
      END_STATE();
    case 527:
      if (lookahead == 'o') ADVANCE(591);
      END_STATE();
    case 528:
      if (lookahead == 'o') ADVANCE(465);
      END_STATE();
    case 529:
      if (lookahead == 'o') ADVANCE(757);
      END_STATE();
    case 530:
      if (lookahead == 'o') ADVANCE(129);
      END_STATE();
    case 531:
      if (lookahead == 'o') ADVANCE(449);
      END_STATE();
    case 532:
      if (lookahead == 'o') ADVANCE(168);
      END_STATE();
    case 533:
      if (lookahead == 'o') ADVANCE(540);
      END_STATE();
    case 534:
      if (lookahead == 'o') ADVANCE(615);
      END_STATE();
    case 535:
      if (lookahead == 'o') ADVANCE(451);
      END_STATE();
    case 536:
      if (lookahead == 'o') ADVANCE(452);
      END_STATE();
    case 537:
      if (lookahead == 'o') ADVANCE(543);
      if (lookahead == 'u') ADVANCE(127);
      END_STATE();
    case 538:
      if (lookahead == 'o') ADVANCE(453);
      END_STATE();
    case 539:
      if (lookahead == 'o') ADVANCE(454);
      END_STATE();
    case 540:
      if (lookahead == 'o') ADVANCE(653);
      END_STATE();
    case 541:
      if (lookahead == 'o') ADVANCE(455);
      END_STATE();
    case 542:
      if (lookahead == 'o') ADVANCE(600);
      END_STATE();
    case 543:
      if (lookahead == 'o') ADVANCE(732);
      END_STATE();
    case 544:
      if (lookahead == 'o') ADVANCE(456);
      END_STATE();
    case 545:
      if (lookahead == 'o') ADVANCE(457);
      END_STATE();
    case 546:
      if (lookahead == 'o') ADVANCE(458);
      END_STATE();
    case 547:
      if (lookahead == 'o') ADVANCE(459);
      END_STATE();
    case 548:
      if (lookahead == 'o') ADVANCE(460);
      END_STATE();
    case 549:
      if (lookahead == 'o') ADVANCE(443);
      END_STATE();
    case 550:
      if (lookahead == 'o') ADVANCE(25);
      END_STATE();
    case 551:
      if (lookahead == 'o') ADVANCE(602);
      END_STATE();
    case 552:
      if (lookahead == 'o') ADVANCE(279);
      END_STATE();
    case 553:
      if (lookahead == 'o') ADVANCE(770);
      if (lookahead == 'r') ADVANCE(512);
      if (lookahead == 'u') ADVANCE(496);
      END_STATE();
    case 554:
      if (lookahead == 'o') ADVANCE(569);
      END_STATE();
    case 555:
      if (lookahead == 'o') ADVANCE(654);
      END_STATE();
    case 556:
      if (lookahead == 'o') ADVANCE(763);
      if (lookahead == 't') ADVANCE(328);
      END_STATE();
    case 557:
      if (lookahead == 'o') ADVANCE(658);
      END_STATE();
    case 558:
      if (lookahead == 'o') ADVANCE(611);
      END_STATE();
    case 559:
      if (lookahead == 'o') ADVANCE(739);
      END_STATE();
    case 560:
      if (lookahead == 'o') ADVANCE(555);
      END_STATE();
    case 561:
      if (lookahead == 'o') ADVANCE(286);
      END_STATE();
    case 562:
      if (lookahead == 'o') ADVANCE(740);
      END_STATE();
    case 563:
      if (lookahead == 'o') ADVANCE(741);
      END_STATE();
    case 564:
      if (lookahead == 'p') ADVANCE(69);
      END_STATE();
    case 565:
      if (lookahead == 'p') ADVANCE(16);
      END_STATE();
    case 566:
      if (lookahead == 'p') ADVANCE(334);
      END_STATE();
    case 567:
      if (lookahead == 'p') ADVANCE(397);
      END_STATE();
    case 568:
      if (lookahead == 'p') ADVANCE(522);
      END_STATE();
    case 569:
      if (lookahead == 'p') ADVANCE(532);
      END_STATE();
    case 570:
      if (lookahead == 'p') ADVANCE(707);
      END_STATE();
    case 571:
      if (lookahead == 'p') ADVANCE(243);
      END_STATE();
    case 572:
      if (lookahead == 'p') ADVANCE(259);
      END_STATE();
    case 573:
      if (lookahead == 'p') ADVANCE(727);
      END_STATE();
    case 574:
      if (lookahead == 'p') ADVANCE(386);
      END_STATE();
    case 575:
      if (lookahead == 'p') ADVANCE(506);
      END_STATE();
    case 576:
      if (lookahead == 'p') ADVANCE(623);
      END_STATE();
    case 577:
      if (lookahead == 'p') ADVANCE(624);
      END_STATE();
    case 578:
      if (lookahead == 'q') ADVANCE(751);
      END_STATE();
    case 579:
      if (lookahead == 'r') ADVANCE(61);
      END_STATE();
    case 580:
      if (lookahead == 'r') ADVANCE(856);
      END_STATE();
    case 581:
      if (lookahead == 'r') ADVANCE(857);
      END_STATE();
    case 582:
      if (lookahead == 'r') ADVANCE(838);
      END_STATE();
    case 583:
      if (lookahead == 'r') ADVANCE(878);
      END_STATE();
    case 584:
      if (lookahead == 'r') ADVANCE(881);
      END_STATE();
    case 585:
      if (lookahead == 'r') ADVANCE(890);
      END_STATE();
    case 586:
      if (lookahead == 'r') ADVANCE(916);
      END_STATE();
    case 587:
      if (lookahead == 'r') ADVANCE(903);
      END_STATE();
    case 588:
      if (lookahead == 'r') ADVANCE(902);
      END_STATE();
    case 589:
      if (lookahead == 'r') ADVANCE(817);
      END_STATE();
    case 590:
      if (lookahead == 'r') ADVANCE(774);
      END_STATE();
    case 591:
      if (lookahead == 'r') ADVANCE(573);
      END_STATE();
    case 592:
      if (lookahead == 'r') ADVANCE(777);
      END_STATE();
    case 593:
      if (lookahead == 'r') ADVANCE(768);
      END_STATE();
    case 594:
      if (lookahead == 'r') ADVANCE(51);
      END_STATE();
    case 595:
      if (lookahead == 'r') ADVANCE(575);
      END_STATE();
    case 596:
      if (lookahead == 'r') ADVANCE(57);
      END_STATE();
    case 597:
      if (lookahead == 'r') ADVANCE(676);
      END_STATE();
    case 598:
      if (lookahead == 'r') ADVANCE(94);
      END_STATE();
    case 599:
      if (lookahead == 'r') ADVANCE(178);
      END_STATE();
    case 600:
      if (lookahead == 'r') ADVANCE(427);
      END_STATE();
    case 601:
      if (lookahead == 'r') ADVANCE(352);
      END_STATE();
    case 602:
      if (lookahead == 'r') ADVANCE(208);
      END_STATE();
    case 603:
      if (lookahead == 'r') ADVANCE(41);
      END_STATE();
    case 604:
      if (lookahead == 'r') ADVANCE(559);
      END_STATE();
    case 605:
      if (lookahead == 'r') ADVANCE(554);
      END_STATE();
    case 606:
      if (lookahead == 'r') ADVANCE(204);
      END_STATE();
    case 607:
      if (lookahead == 'r') ADVANCE(261);
      END_STATE();
    case 608:
      if (lookahead == 'r') ADVANCE(660);
      END_STATE();
    case 609:
      if (lookahead == 'r') ADVANCE(705);
      END_STATE();
    case 610:
      if (lookahead == 'r') ADVANCE(709);
      END_STATE();
    case 611:
      if (lookahead == 'r') ADVANCE(179);
      END_STATE();
    case 612:
      if (lookahead == 'r') ADVANCE(219);
      END_STATE();
    case 613:
      if (lookahead == 'r') ADVANCE(20);
      END_STATE();
    case 614:
      if (lookahead == 'r') ADVANCE(89);
      END_STATE();
    case 615:
      if (lookahead == 'r') ADVANCE(485);
      END_STATE();
    case 616:
      if (lookahead == 'r') ADVANCE(661);
      END_STATE();
    case 617:
      if (lookahead == 'r') ADVANCE(258);
      END_STATE();
    case 618:
      if (lookahead == 'r') ADVANCE(225);
      END_STATE();
    case 619:
      if (lookahead == 'r') ADVANCE(228);
      END_STATE();
    case 620:
      if (lookahead == 'r') ADVANCE(103);
      END_STATE();
    case 621:
      if (lookahead == 'r') ADVANCE(104);
      END_STATE();
    case 622:
      if (lookahead == 'r') ADVANCE(106);
      END_STATE();
    case 623:
      if (lookahead == 'r') ADVANCE(562);
      END_STATE();
    case 624:
      if (lookahead == 'r') ADVANCE(563);
      END_STATE();
    case 625:
      if (lookahead == 's') ADVANCE(405);
      END_STATE();
    case 626:
      if (lookahead == 's') ADVANCE(405);
      if (lookahead == 't') ADVANCE(124);
      if (lookahead == 'x') ADVANCE(840);
      END_STATE();
    case 627:
      if (lookahead == 's') ADVANCE(792);
      END_STATE();
    case 628:
      if (lookahead == 's') ADVANCE(848);
      END_STATE();
    case 629:
      if (lookahead == 's') ADVANCE(849);
      END_STATE();
    case 630:
      if (lookahead == 's') ADVANCE(801);
      END_STATE();
    case 631:
      if (lookahead == 's') ADVANCE(832);
      END_STATE();
    case 632:
      if (lookahead == 's') ADVANCE(804);
      END_STATE();
    case 633:
      if (lookahead == 's') ADVANCE(863);
      END_STATE();
    case 634:
      if (lookahead == 's') ADVANCE(879);
      END_STATE();
    case 635:
      if (lookahead == 's') ADVANCE(876);
      END_STATE();
    case 636:
      if (lookahead == 's') ADVANCE(901);
      END_STATE();
    case 637:
      if (lookahead == 's') ADVANCE(905);
      END_STATE();
    case 638:
      if (lookahead == 's') ADVANCE(907);
      END_STATE();
    case 639:
      if (lookahead == 's') ADVANCE(113);
      END_STATE();
    case 640:
      if (lookahead == 's') ADVANCE(744);
      END_STATE();
    case 641:
      if (lookahead == 's') ADVANCE(639);
      END_STATE();
    case 642:
      if (lookahead == 's') ADVANCE(567);
      END_STATE();
    case 643:
      if (lookahead == 's') ADVANCE(630);
      END_STATE();
    case 644:
      if (lookahead == 's') ADVANCE(343);
      END_STATE();
    case 645:
      if (lookahead == 's') ADVANCE(632);
      END_STATE();
    case 646:
      if (lookahead == 's') ADVANCE(21);
      END_STATE();
    case 647:
      if (lookahead == 's') ADVANCE(571);
      END_STATE();
    case 648:
      if (lookahead == 's') ADVANCE(527);
      END_STATE();
    case 649:
      if (lookahead == 's') ADVANCE(572);
      END_STATE();
    case 650:
      if (lookahead == 's') ADVANCE(633);
      END_STATE();
    case 651:
      if (lookahead == 's') ADVANCE(634);
      END_STATE();
    case 652:
      if (lookahead == 's') ADVANCE(635);
      END_STATE();
    case 653:
      if (lookahead == 's') ADVANCE(682);
      END_STATE();
    case 654:
      if (lookahead == 's') ADVANCE(683);
      END_STATE();
    case 655:
      if (lookahead == 's') ADVANCE(637);
      END_STATE();
    case 656:
      if (lookahead == 's') ADVANCE(498);
      END_STATE();
    case 657:
      if (lookahead == 's') ADVANCE(708);
      END_STATE();
    case 658:
      if (lookahead == 's') ADVANCE(710);
      END_STATE();
    case 659:
      if (lookahead == 's') ADVANCE(715);
      END_STATE();
    case 660:
      if (lookahead == 's') ADVANCE(197);
      END_STATE();
    case 661:
      if (lookahead == 's') ADVANCE(198);
      END_STATE();
    case 662:
      if (lookahead == 's') ADVANCE(217);
      END_STATE();
    case 663:
      if (lookahead == 's') ADVANCE(728);
      if (lookahead == 'v') ADVANCE(353);
      END_STATE();
    case 664:
      if (lookahead == 's') ADVANCE(706);
      END_STATE();
    case 665:
      if (lookahead == 's') ADVANCE(531);
      END_STATE();
    case 666:
      if (lookahead == 's') ADVANCE(718);
      END_STATE();
    case 667:
      if (lookahead == 's') ADVANCE(232);
      END_STATE();
    case 668:
      if (lookahead == 's') ADVANCE(379);
      END_STATE();
    case 669:
      if (lookahead == 's') ADVANCE(337);
      END_STATE();
    case 670:
      if (lookahead == 's') ADVANCE(738);
      END_STATE();
    case 671:
      if (lookahead == 's') ADVANCE(395);
      END_STATE();
    case 672:
      if (lookahead == 't') ADVANCE(930);
      END_STATE();
    case 673:
      if (lookahead == 't') ADVANCE(793);
      END_STATE();
    case 674:
      if (lookahead == 't') ADVANCE(809);
      END_STATE();
    case 675:
      if (lookahead == 't') ADVANCE(937);
      END_STATE();
    case 676:
      if (lookahead == 't') ADVANCE(806);
      END_STATE();
    case 677:
      if (lookahead == 't') ADVANCE(859);
      END_STATE();
    case 678:
      if (lookahead == 't') ADVANCE(803);
      END_STATE();
    case 679:
      if (lookahead == 't') ADVANCE(861);
      END_STATE();
    case 680:
      if (lookahead == 't') ADVANCE(939);
      END_STATE();
    case 681:
      if (lookahead == 't') ADVANCE(9);
      END_STATE();
    case 682:
      if (lookahead == 't') ADVANCE(869);
      END_STATE();
    case 683:
      if (lookahead == 't') ADVANCE(882);
      END_STATE();
    case 684:
      if (lookahead == 't') ADVANCE(852);
      END_STATE();
    case 685:
      if (lookahead == 't') ADVANCE(909);
      END_STATE();
    case 686:
      if (lookahead == 't') ADVANCE(10);
      END_STATE();
    case 687:
      if (lookahead == 't') ADVANCE(50);
      if (lookahead == 'y') ADVANCE(936);
      END_STATE();
    case 688:
      if (lookahead == 't') ADVANCE(775);
      END_STATE();
    case 689:
      if (lookahead == 't') ADVANCE(776);
      END_STATE();
    case 690:
      if (lookahead == 't') ADVANCE(15);
      END_STATE();
    case 691:
      if (lookahead == 't') ADVANCE(83);
      END_STATE();
    case 692:
      if (lookahead == 't') ADVANCE(366);
      END_STATE();
    case 693:
      if (lookahead == 't') ADVANCE(324);
      END_STATE();
    case 694:
      if (lookahead == 't') ADVANCE(629);
      END_STATE();
    case 695:
      if (lookahead == 't') ADVANCE(778);
      END_STATE();
    case 696:
      if (lookahead == 't') ADVANCE(17);
      END_STATE();
    case 697:
      if (lookahead == 't') ADVANCE(746);
      END_STATE();
    case 698:
      if (lookahead == 't') ADVANCE(175);
      END_STATE();
    case 699:
      if (lookahead == 't') ADVANCE(338);
      END_STATE();
    case 700:
      if (lookahead == 't') ADVANCE(780);
      END_STATE();
    case 701:
      if (lookahead == 't') ADVANCE(315);
      END_STATE();
    case 702:
      if (lookahead == 't') ADVANCE(11);
      END_STATE();
    case 703:
      if (lookahead == 't') ADVANCE(336);
      END_STATE();
    case 704:
      if (lookahead == 't') ADVANCE(316);
      END_STATE();
    case 705:
      if (lookahead == 't') ADVANCE(758);
      END_STATE();
    case 706:
      if (lookahead == 't') ADVANCE(77);
      END_STATE();
    case 707:
      if (lookahead == 't') ADVANCE(332);
      END_STATE();
    case 708:
      if (lookahead == 't') ADVANCE(22);
      END_STATE();
    case 709:
      if (lookahead == 't') ADVANCE(333);
      END_STATE();
    case 710:
      if (lookahead == 't') ADVANCE(19);
      END_STATE();
    case 711:
      if (lookahead == 't') ADVANCE(370);
      END_STATE();
    case 712:
      if (lookahead == 't') ADVANCE(205);
      END_STATE();
    case 713:
      if (lookahead == 't') ADVANCE(182);
      END_STATE();
    case 714:
      if (lookahead == 't') ADVANCE(184);
      END_STATE();
    case 715:
      if (lookahead == 't') ADVANCE(601);
      END_STATE();
    case 716:
      if (lookahead == 't') ADVANCE(529);
      END_STATE();
    case 717:
      if (lookahead == 't') ADVANCE(218);
      END_STATE();
    case 718:
      if (lookahead == 't') ADVANCE(186);
      END_STATE();
    case 719:
      if (lookahead == 't') ADVANCE(195);
      END_STATE();
    case 720:
      if (lookahead == 't') ADVANCE(102);
      END_STATE();
    case 721:
      if (lookahead == 't') ADVANCE(325);
      END_STATE();
    case 722:
      if (lookahead == 't') ADVANCE(348);
      END_STATE();
    case 723:
      if (lookahead == 't') ADVANCE(326);
      END_STATE();
    case 724:
      if (lookahead == 't') ADVANCE(28);
      END_STATE();
    case 725:
      if (lookahead == 't') ADVANCE(327);
      END_STATE();
    case 726:
      if (lookahead == 't') ADVANCE(329);
      END_STATE();
    case 727:
      if (lookahead == 't') ADVANCE(372);
      END_STATE();
    case 728:
      if (lookahead == 't') ADVANCE(93);
      END_STATE();
    case 729:
      if (lookahead == 't') ADVANCE(374);
      END_STATE();
    case 730:
      if (lookahead == 't') ADVANCE(377);
      END_STATE();
    case 731:
      if (lookahead == 't') ADVANCE(381);
      END_STATE();
    case 732:
      if (lookahead == 't') ADVANCE(382);
      END_STATE();
    case 733:
      if (lookahead == 't') ADVANCE(383);
      END_STATE();
    case 734:
      if (lookahead == 't') ADVANCE(385);
      END_STATE();
    case 735:
      if (lookahead == 't') ADVANCE(387);
      END_STATE();
    case 736:
      if (lookahead == 't') ADVANCE(389);
      END_STATE();
    case 737:
      if (lookahead == 't') ADVANCE(391);
      END_STATE();
    case 738:
      if (lookahead == 't') ADVANCE(101);
      END_STATE();
    case 739:
      if (lookahead == 't') ADVANCE(269);
      END_STATE();
    case 740:
      if (lookahead == 't') ADVANCE(270);
      END_STATE();
    case 741:
      if (lookahead == 't') ADVANCE(271);
      END_STATE();
    case 742:
      if (lookahead == 'u') ADVANCE(112);
      END_STATE();
    case 743:
      if (lookahead == 'u') ADVANCE(440);
      END_STATE();
    case 744:
      if (lookahead == 'u') ADVANCE(424);
      END_STATE();
    case 745:
      if (lookahead == 'u') ADVANCE(174);
      END_STATE();
    case 746:
      if (lookahead == 'u') ADVANCE(620);
      END_STATE();
    case 747:
      if (lookahead == 'u') ADVANCE(125);
      END_STATE();
    case 748:
      if (lookahead == 'u') ADVANCE(662);
      END_STATE();
    case 749:
      if (lookahead == 'u') ADVANCE(425);
      END_STATE();
    case 750:
      if (lookahead == 'u') ADVANCE(180);
      END_STATE();
    case 751:
      if (lookahead == 'u') ADVANCE(70);
      END_STATE();
    case 752:
      if (lookahead == 'u') ADVANCE(608);
      END_STATE();
    case 753:
      if (lookahead == 'u') ADVANCE(190);
      END_STATE();
    case 754:
      if (lookahead == 'u') ADVANCE(247);
      END_STATE();
    case 755:
      if (lookahead == 'u') ADVANCE(713);
      END_STATE();
    case 756:
      if (lookahead == 'u') ADVANCE(356);
      END_STATE();
    case 757:
      if (lookahead == 'u') ADVANCE(133);
      END_STATE();
    case 758:
      if (lookahead == 'u') ADVANCE(488);
      END_STATE();
    case 759:
      if (lookahead == 'u') ADVANCE(616);
      END_STATE();
    case 760:
      if (lookahead == 'v') ADVANCE(176);
      END_STATE();
    case 761:
      if (lookahead == 'v') ADVANCE(209);
      END_STATE();
    case 762:
      if (lookahead == 'v') ADVANCE(367);
      END_STATE();
    case 763:
      if (lookahead == 'v') ADVANCE(236);
      END_STATE();
    case 764:
      if (lookahead == 'v') ADVANCE(368);
      END_STATE();
    case 765:
      if (lookahead == 'v') ADVANCE(357);
      END_STATE();
    case 766:
      if (lookahead == 'w') ADVANCE(858);
      END_STATE();
    case 767:
      if (lookahead == 'w') ADVANCE(14);
      END_STATE();
    case 768:
      if (lookahead == 'w') ADVANCE(542);
      END_STATE();
    case 769:
      if (lookahead == 'w') ADVANCE(223);
      END_STATE();
    case 770:
      if (lookahead == 'w') ADVANCE(226);
      END_STATE();
    case 771:
      if (lookahead == 'w') ADVANCE(74);
      END_STATE();
    case 772:
      if (lookahead == 'w') ADVANCE(373);
      END_STATE();
    case 773:
      if (lookahead == 'y') ADVANCE(855);
      END_STATE();
    case 774:
      if (lookahead == 'y') ADVANCE(931);
      END_STATE();
    case 775:
      if (lookahead == 'y') ADVANCE(795);
      END_STATE();
    case 776:
      if (lookahead == 'y') ADVANCE(875);
      END_STATE();
    case 777:
      if (lookahead == 'y') ADVANCE(853);
      END_STATE();
    case 778:
      if (lookahead == 'y') ADVANCE(919);
      END_STATE();
    case 779:
      if (lookahead == 'y') ADVANCE(912);
      END_STATE();
    case 780:
      if (lookahead == 'y') ADVANCE(900);
      END_STATE();
    case 781:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 782:
      ACCEPT_TOKEN(anon_sym_fn);
      END_STATE();
    case 783:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 784:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 785:
      ACCEPT_TOKEN(aux_sym_block_token1);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(785);
      END_STATE();
    case 786:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 787:
      ACCEPT_TOKEN(anon_sym_execute);
      END_STATE();
    case 788:
      ACCEPT_TOKEN(anon_sym_as);
      END_STATE();
    case 789:
      ACCEPT_TOKEN(anon_sym_at);
      END_STATE();
    case 790:
      ACCEPT_TOKEN(anon_sym_align);
      END_STATE();
    case 791:
      ACCEPT_TOKEN(anon_sym_anchored);
      END_STATE();
    case 792:
      ACCEPT_TOKEN(anon_sym_eyes);
      END_STATE();
    case 793:
      ACCEPT_TOKEN(anon_sym_feet);
      END_STATE();
    case 794:
      ACCEPT_TOKEN(anon_sym_facing);
      END_STATE();
    case 795:
      ACCEPT_TOKEN(anon_sym_entity);
      END_STATE();
    case 796:
      ACCEPT_TOKEN(anon_sym_in);
      END_STATE();
    case 797:
      ACCEPT_TOKEN(anon_sym_in);
      if (lookahead == 't') ADVANCE(807);
      END_STATE();
    case 798:
      ACCEPT_TOKEN(anon_sym_positioned);
      END_STATE();
    case 799:
      ACCEPT_TOKEN(anon_sym_rotated);
      END_STATE();
    case 800:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 801:
      ACCEPT_TOKEN(anon_sym_unless);
      END_STATE();
    case 802:
      ACCEPT_TOKEN(anon_sym_store);
      END_STATE();
    case 803:
      ACCEPT_TOKEN(anon_sym_result);
      END_STATE();
    case 804:
      ACCEPT_TOKEN(anon_sym_success);
      END_STATE();
    case 805:
      ACCEPT_TOKEN(anon_sym_byte);
      END_STATE();
    case 806:
      ACCEPT_TOKEN(anon_sym_short);
      END_STATE();
    case 807:
      ACCEPT_TOKEN(anon_sym_int);
      END_STATE();
    case 808:
      ACCEPT_TOKEN(anon_sym_long);
      END_STATE();
    case 809:
      ACCEPT_TOKEN(anon_sym_float);
      END_STATE();
    case 810:
      ACCEPT_TOKEN(anon_sym_double);
      END_STATE();
    case 811:
      ACCEPT_TOKEN(anon_sym_run);
      END_STATE();
    case 812:
      ACCEPT_TOKEN(anon_sym_TILDE);
      END_STATE();
    case 813:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 814:
      ACCEPT_TOKEN(anon_sym_CARET);
      END_STATE();
    case 815:
      ACCEPT_TOKEN(sym_align_axes);
      if (('x' <= lookahead && lookahead <= 'z')) ADVANCE(815);
      END_STATE();
    case 816:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONoverworld);
      END_STATE();
    case 817:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONthe_nether);
      END_STATE();
    case 818:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONthe_end);
      END_STATE();
    case 819:
      ACCEPT_TOKEN(anon_sym_block);
      if (lookahead == 's') ADVANCE(826);
      END_STATE();
    case 820:
      ACCEPT_TOKEN(anon_sym_minecraft_COLON);
      END_STATE();
    case 821:
      ACCEPT_TOKEN(anon_sym_minecraft_COLON);
      ADVANCE_MAP(
        'a', 108,
        'b', 65,
        'c', 528,
        'd', 524,
        'f', 369,
        'g', 420,
        'h', 71,
        'i', 448,
        'j', 743,
        'l', 202,
        'm', 349,
        'n', 63,
        'p', 526,
        'r', 185,
        's', 75,
        'u', 470,
        'w', 100,
      );
      END_STATE();
    case 822:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 823:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(928);
      END_STATE();
    case 824:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 825:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 826:
      ACCEPT_TOKEN(anon_sym_blocks);
      END_STATE();
    case 827:
      ACCEPT_TOKEN(anon_sym_masked);
      END_STATE();
    case 828:
      ACCEPT_TOKEN(anon_sym_data);
      END_STATE();
    case 829:
      ACCEPT_TOKEN(anon_sym_storage);
      END_STATE();
    case 830:
      ACCEPT_TOKEN(anon_sym_predicate);
      END_STATE();
    case 831:
      ACCEPT_TOKEN(anon_sym_score);
      END_STATE();
    case 832:
      ACCEPT_TOKEN(anon_sym_matches);
      END_STATE();
    case 833:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '=') ADVANCE(834);
      END_STATE();
    case 834:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 835:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 836:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 837:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead == '=') ADVANCE(836);
      END_STATE();
    case 838:
      ACCEPT_TOKEN(anon_sym_bossbar);
      END_STATE();
    case 839:
      ACCEPT_TOKEN(anon_sym_value);
      END_STATE();
    case 840:
      ACCEPT_TOKEN(anon_sym_max);
      END_STATE();
    case 841:
      ACCEPT_TOKEN(sym_nbt_path);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(841);
      if (lookahead == '.' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '\\' ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= '{') ||
          lookahead == '}') ADVANCE(842);
      END_STATE();
    case 842:
      ACCEPT_TOKEN(sym_nbt_path);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '\\' ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= '{') ||
          lookahead == '}') ADVANCE(842);
      END_STATE();
    case 843:
      ACCEPT_TOKEN(anon_sym_ATs);
      END_STATE();
    case 844:
      ACCEPT_TOKEN(anon_sym_ATa);
      END_STATE();
    case 845:
      ACCEPT_TOKEN(anon_sym_ATp);
      END_STATE();
    case 846:
      ACCEPT_TOKEN(anon_sym_ATr);
      END_STATE();
    case 847:
      ACCEPT_TOKEN(anon_sym_ATe);
      END_STATE();
    case 848:
      ACCEPT_TOKEN(anon_sym_levels);
      END_STATE();
    case 849:
      ACCEPT_TOKEN(anon_sym_points);
      END_STATE();
    case 850:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 851:
      ACCEPT_TOKEN(anon_sym_xpadd);
      END_STATE();
    case 852:
      ACCEPT_TOKEN(anon_sym_xpset);
      END_STATE();
    case 853:
      ACCEPT_TOKEN(anon_sym_xpquery);
      END_STATE();
    case 854:
      ACCEPT_TOKEN(aux_sym_quoted_string_token1);
      END_STATE();
    case 855:
      ACCEPT_TOKEN(anon_sym_say);
      END_STATE();
    case 856:
      ACCEPT_TOKEN(anon_sym_clear);
      END_STATE();
    case 857:
      ACCEPT_TOKEN(anon_sym_eclear);
      END_STATE();
    case 858:
      ACCEPT_TOKEN(anon_sym_tellraw);
      END_STATE();
    case 859:
      ACCEPT_TOKEN(anon_sym_effect);
      END_STATE();
    case 860:
      ACCEPT_TOKEN(anon_sym_give);
      END_STATE();
    case 861:
      ACCEPT_TOKEN(anon_sym_enchant);
      END_STATE();
    case 862:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONspeed);
      END_STATE();
    case 863:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslowness);
      END_STATE();
    case 864:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhaste);
      END_STATE();
    case 865:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmining_fatigue);
      END_STATE();
    case 866:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONstrength);
      END_STATE();
    case 867:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_health);
      END_STATE();
    case 868:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_damage);
      END_STATE();
    case 869:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONjump_boost);
      END_STATE();
    case 870:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnausea);
      END_STATE();
    case 871:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONregeneration);
      END_STATE();
    case 872:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONresistance);
      END_STATE();
    case 873:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_resistance);
      END_STATE();
    case 874:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwater_breathing);
      END_STATE();
    case 875:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinvisibility);
      END_STATE();
    case 876:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblindness);
      END_STATE();
    case 877:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnight_vision);
      END_STATE();
    case 878:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhunger);
      END_STATE();
    case 879:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONweakness);
      END_STATE();
    case 880:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpoison);
      END_STATE();
    case 881:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwither);
      END_STATE();
    case 882:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhealth_boost);
      END_STATE();
    case 883:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONabsorption);
      END_STATE();
    case 884:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsaturation);
      END_STATE();
    case 885:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONglowing);
      END_STATE();
    case 886:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlevitation);
      END_STATE();
    case 887:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      END_STATE();
    case 888:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunluck);
      END_STATE();
    case 889:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslow_falling);
      END_STATE();
    case 890:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONconduit_power);
      END_STATE();
    case 891:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdolphins_grace);
      END_STATE();
    case 892:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbad_omen);
      END_STATE();
    case 893:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhero_of_the_village);
      END_STATE();
    case 894:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONprotection);
      END_STATE();
    case 895:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_protection);
      END_STATE();
    case 896:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfeather_falling);
      END_STATE();
    case 897:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblast_protection);
      END_STATE();
    case 898:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONprojectile_protection);
      END_STATE();
    case 899:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONrespiration);
      END_STATE();
    case 900:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONaqua_affinity);
      END_STATE();
    case 901:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONthorns);
      END_STATE();
    case 902:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdepth_strider);
      END_STATE();
    case 903:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfrost_walker);
      END_STATE();
    case 904:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbinding_curse);
      END_STATE();
    case 905:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsharpness);
      END_STATE();
    case 906:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsmite);
      END_STATE();
    case 907:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbane_of_arthropods);
      END_STATE();
    case 908:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONknockback);
      END_STATE();
    case 909:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_aspect);
      END_STATE();
    case 910:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlooting);
      END_STATE();
    case 911:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsweeping);
      END_STATE();
    case 912:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONefficiency);
      END_STATE();
    case 913:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsilk_touch);
      END_STATE();
    case 914:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunbreaking);
      END_STATE();
    case 915:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfortune);
      END_STATE();
    case 916:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpower);
      END_STATE();
    case 917:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpunch);
      END_STATE();
    case 918:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONflame);
      END_STATE();
    case 919:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinfinity);
      END_STATE();
    case 920:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck_of_the_sea);
      END_STATE();
    case 921:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlure);
      END_STATE();
    case 922:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmending);
      END_STATE();
    case 923:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONvanishing_curse);
      END_STATE();
    case 924:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsoul_speed);
      END_STATE();
    case 925:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONswift_sneak);
      END_STATE();
    case 926:
      ACCEPT_TOKEN(sym_text);
      if (lookahead == '[') ADVANCE(823);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(926);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(928);
      END_STATE();
    case 927:
      ACCEPT_TOKEN(sym_text);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(927);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(928);
      END_STATE();
    case 928:
      ACCEPT_TOKEN(sym_text);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(928);
      END_STATE();
    case 929:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 930:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 931:
      ACCEPT_TOKEN(anon_sym_query);
      END_STATE();
    case 932:
      ACCEPT_TOKEN(anon_sym_gms);
      if (lookahead == 'p') ADVANCE(934);
      END_STATE();
    case 933:
      ACCEPT_TOKEN(anon_sym_gma);
      END_STATE();
    case 934:
      ACCEPT_TOKEN(anon_sym_gmsp);
      END_STATE();
    case 935:
      ACCEPT_TOKEN(anon_sym_gmc);
      END_STATE();
    case 936:
      ACCEPT_TOKEN(anon_sym_day);
      END_STATE();
    case 937:
      ACCEPT_TOKEN(anon_sym_night);
      END_STATE();
    case 938:
      ACCEPT_TOKEN(anon_sym_noon);
      END_STATE();
    case 939:
      ACCEPT_TOKEN(anon_sym_midnight);
      END_STATE();
    case 940:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == ':') ADVANCE(53);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 941:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == ':') ADVANCE(556);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 942:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == ':') ADVANCE(820);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 943:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(952);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 944:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(953);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 945:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(954);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 946:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(961);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 947:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(962);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 948:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(963);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 949:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(946);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 950:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(947);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 951:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(948);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 952:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(964);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 953:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(965);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 954:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(966);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 955:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(958);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 956:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(959);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 957:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(960);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 958:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(949);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 959:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(950);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 960:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(951);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 961:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(943);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 962:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(944);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 963:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(945);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 964:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(940);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 965:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(941);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 966:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(942);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 967:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(967);
      END_STATE();
    case 968:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(968);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 2},
  [3] = {.lex_state = 2},
  [4] = {.lex_state = 2},
  [5] = {.lex_state = 2},
  [6] = {.lex_state = 2},
  [7] = {.lex_state = 0},
  [8] = {.lex_state = 0},
  [9] = {.lex_state = 3},
  [10] = {.lex_state = 4},
  [11] = {.lex_state = 3},
  [12] = {.lex_state = 116},
  [13] = {.lex_state = 116},
  [14] = {.lex_state = 3},
  [15] = {.lex_state = 3},
  [16] = {.lex_state = 3},
  [17] = {.lex_state = 116},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 116},
  [21] = {.lex_state = 2},
  [22] = {.lex_state = 2},
  [23] = {.lex_state = 2},
  [24] = {.lex_state = 2},
  [25] = {.lex_state = 2},
  [26] = {.lex_state = 2},
  [27] = {.lex_state = 2},
  [28] = {.lex_state = 0},
  [29] = {.lex_state = 0},
  [30] = {.lex_state = 0},
  [31] = {.lex_state = 0},
  [32] = {.lex_state = 0},
  [33] = {.lex_state = 0},
  [34] = {.lex_state = 2},
  [35] = {.lex_state = 2},
  [36] = {.lex_state = 2},
  [37] = {.lex_state = 116},
  [38] = {.lex_state = 2},
  [39] = {.lex_state = 2},
  [40] = {.lex_state = 2},
  [41] = {.lex_state = 2},
  [42] = {.lex_state = 116},
  [43] = {.lex_state = 2},
  [44] = {.lex_state = 2},
  [45] = {.lex_state = 2},
  [46] = {.lex_state = 2},
  [47] = {.lex_state = 2},
  [48] = {.lex_state = 2},
  [49] = {.lex_state = 2},
  [50] = {.lex_state = 2},
  [51] = {.lex_state = 2},
  [52] = {.lex_state = 2},
  [53] = {.lex_state = 2},
  [54] = {.lex_state = 2},
  [55] = {.lex_state = 2},
  [56] = {.lex_state = 2},
  [57] = {.lex_state = 2},
  [58] = {.lex_state = 2},
  [59] = {.lex_state = 2},
  [60] = {.lex_state = 2},
  [61] = {.lex_state = 2},
  [62] = {.lex_state = 2},
  [63] = {.lex_state = 2},
  [64] = {.lex_state = 2},
  [65] = {.lex_state = 2},
  [66] = {.lex_state = 2},
  [67] = {.lex_state = 2},
  [68] = {.lex_state = 2},
  [69] = {.lex_state = 2},
  [70] = {.lex_state = 2},
  [71] = {.lex_state = 2},
  [72] = {.lex_state = 2},
  [73] = {.lex_state = 2},
  [74] = {.lex_state = 2},
  [75] = {.lex_state = 2},
  [76] = {.lex_state = 2},
  [77] = {.lex_state = 2},
  [78] = {.lex_state = 2},
  [79] = {.lex_state = 2},
  [80] = {.lex_state = 2},
  [81] = {.lex_state = 2},
  [82] = {.lex_state = 2},
  [83] = {.lex_state = 2},
  [84] = {.lex_state = 2},
  [85] = {.lex_state = 2},
  [86] = {.lex_state = 2},
  [87] = {.lex_state = 2},
  [88] = {.lex_state = 2},
  [89] = {.lex_state = 2},
  [90] = {.lex_state = 2},
  [91] = {.lex_state = 2},
  [92] = {.lex_state = 2},
  [93] = {.lex_state = 2},
  [94] = {.lex_state = 2},
  [95] = {.lex_state = 2},
  [96] = {.lex_state = 2},
  [97] = {.lex_state = 2},
  [98] = {.lex_state = 2},
  [99] = {.lex_state = 2},
  [100] = {.lex_state = 2},
  [101] = {.lex_state = 2},
  [102] = {.lex_state = 2},
  [103] = {.lex_state = 2},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 0},
  [108] = {.lex_state = 0},
  [109] = {.lex_state = 0},
  [110] = {.lex_state = 0},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 0},
  [114] = {.lex_state = 0},
  [115] = {.lex_state = 0},
  [116] = {.lex_state = 0},
  [117] = {.lex_state = 0},
  [118] = {.lex_state = 0},
  [119] = {.lex_state = 0},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 0},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
  [125] = {.lex_state = 0},
  [126] = {.lex_state = 0},
  [127] = {.lex_state = 0},
  [128] = {.lex_state = 0},
  [129] = {.lex_state = 0},
  [130] = {.lex_state = 0},
  [131] = {.lex_state = 0},
  [132] = {.lex_state = 0},
  [133] = {.lex_state = 0},
  [134] = {.lex_state = 0},
  [135] = {.lex_state = 0},
  [136] = {.lex_state = 0},
  [137] = {.lex_state = 0},
  [138] = {.lex_state = 0},
  [139] = {.lex_state = 0},
  [140] = {.lex_state = 0},
  [141] = {.lex_state = 0},
  [142] = {.lex_state = 0},
  [143] = {.lex_state = 0},
  [144] = {.lex_state = 0},
  [145] = {.lex_state = 0},
  [146] = {.lex_state = 1},
  [147] = {.lex_state = 0},
  [148] = {.lex_state = 0},
  [149] = {.lex_state = 0},
  [150] = {.lex_state = 0},
  [151] = {.lex_state = 1},
  [152] = {.lex_state = 0},
  [153] = {.lex_state = 1},
  [154] = {.lex_state = 1},
  [155] = {.lex_state = 1},
  [156] = {.lex_state = 1},
  [157] = {.lex_state = 1},
  [158] = {.lex_state = 1},
  [159] = {.lex_state = 1},
  [160] = {.lex_state = 1},
  [161] = {.lex_state = 1},
  [162] = {.lex_state = 1},
  [163] = {.lex_state = 1},
  [164] = {.lex_state = 7},
  [165] = {.lex_state = 7},
  [166] = {.lex_state = 1},
  [167] = {.lex_state = 1},
  [168] = {.lex_state = 1},
  [169] = {.lex_state = 1},
  [170] = {.lex_state = 1},
  [171] = {.lex_state = 1},
  [172] = {.lex_state = 1},
  [173] = {.lex_state = 0},
  [174] = {.lex_state = 1},
  [175] = {.lex_state = 1},
  [176] = {.lex_state = 926},
  [177] = {.lex_state = 6},
  [178] = {.lex_state = 6},
  [179] = {.lex_state = 0},
  [180] = {.lex_state = 0},
  [181] = {.lex_state = 6},
  [182] = {.lex_state = 1},
  [183] = {.lex_state = 0},
  [184] = {.lex_state = 1},
  [185] = {.lex_state = 1},
  [186] = {.lex_state = 1},
  [187] = {.lex_state = 1},
  [188] = {.lex_state = 0},
  [189] = {.lex_state = 0},
  [190] = {.lex_state = 1},
  [191] = {.lex_state = 7},
  [192] = {.lex_state = 7},
  [193] = {.lex_state = 7},
  [194] = {.lex_state = 7},
  [195] = {.lex_state = 7},
  [196] = {.lex_state = 7},
  [197] = {.lex_state = 0},
  [198] = {.lex_state = 6},
  [199] = {.lex_state = 926},
  [200] = {.lex_state = 7},
  [201] = {.lex_state = 0},
  [202] = {.lex_state = 0},
  [203] = {.lex_state = 0},
  [204] = {.lex_state = 0},
  [205] = {.lex_state = 7},
  [206] = {.lex_state = 0},
  [207] = {.lex_state = 0},
  [208] = {.lex_state = 0},
  [209] = {.lex_state = 927},
  [210] = {.lex_state = 0},
  [211] = {.lex_state = 0},
  [212] = {.lex_state = 0},
  [213] = {.lex_state = 1},
  [214] = {.lex_state = 6},
  [215] = {.lex_state = 0},
  [216] = {.lex_state = 0},
  [217] = {.lex_state = 6},
  [218] = {.lex_state = 6},
  [219] = {.lex_state = 927},
  [220] = {.lex_state = 6},
  [221] = {.lex_state = 6},
  [222] = {.lex_state = 6},
  [223] = {.lex_state = 927},
  [224] = {.lex_state = 6},
  [225] = {.lex_state = 6},
  [226] = {.lex_state = 0},
  [227] = {.lex_state = 927},
  [228] = {.lex_state = 0},
  [229] = {.lex_state = 0},
  [230] = {.lex_state = 0},
  [231] = {.lex_state = 0},
  [232] = {.lex_state = 0},
  [233] = {.lex_state = 0},
  [234] = {.lex_state = 0},
  [235] = {.lex_state = 6},
  [236] = {.lex_state = 0},
  [237] = {.lex_state = 0},
  [238] = {.lex_state = 0},
  [239] = {.lex_state = 6},
  [240] = {.lex_state = 1},
  [241] = {.lex_state = 1},
  [242] = {.lex_state = 0},
  [243] = {.lex_state = 0},
  [244] = {.lex_state = 6},
  [245] = {.lex_state = 1},
  [246] = {.lex_state = 1},
  [247] = {.lex_state = 0},
  [248] = {.lex_state = 6},
  [249] = {.lex_state = 6},
  [250] = {.lex_state = 6},
  [251] = {.lex_state = 1},
  [252] = {.lex_state = 6},
  [253] = {.lex_state = 1},
  [254] = {.lex_state = 0},
  [255] = {.lex_state = 6},
  [256] = {.lex_state = 927},
  [257] = {.lex_state = 0},
  [258] = {.lex_state = 0},
  [259] = {.lex_state = 0},
  [260] = {.lex_state = 0},
  [261] = {.lex_state = 0},
  [262] = {.lex_state = 1},
  [263] = {.lex_state = 0},
  [264] = {.lex_state = 1},
  [265] = {.lex_state = 0},
  [266] = {.lex_state = 1},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [anon_sym_fn] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_execute] = ACTIONS(1),
    [anon_sym_as] = ACTIONS(1),
    [anon_sym_at] = ACTIONS(1),
    [anon_sym_align] = ACTIONS(1),
    [anon_sym_anchored] = ACTIONS(1),
    [anon_sym_eyes] = ACTIONS(1),
    [anon_sym_feet] = ACTIONS(1),
    [anon_sym_facing] = ACTIONS(1),
    [anon_sym_entity] = ACTIONS(1),
    [anon_sym_in] = ACTIONS(1),
    [anon_sym_positioned] = ACTIONS(1),
    [anon_sym_rotated] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_unless] = ACTIONS(1),
    [anon_sym_store] = ACTIONS(1),
    [anon_sym_result] = ACTIONS(1),
    [anon_sym_success] = ACTIONS(1),
    [anon_sym_byte] = ACTIONS(1),
    [anon_sym_short] = ACTIONS(1),
    [anon_sym_int] = ACTIONS(1),
    [anon_sym_long] = ACTIONS(1),
    [anon_sym_float] = ACTIONS(1),
    [anon_sym_double] = ACTIONS(1),
    [anon_sym_run] = ACTIONS(1),
    [anon_sym_TILDE] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_CARET] = ACTIONS(1),
    [sym_align_axes] = ACTIONS(1),
    [anon_sym_block] = ACTIONS(1),
    [anon_sym_minecraft_COLON] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_blocks] = ACTIONS(1),
    [anon_sym_masked] = ACTIONS(1),
    [anon_sym_data] = ACTIONS(1),
    [anon_sym_storage] = ACTIONS(1),
    [anon_sym_predicate] = ACTIONS(1),
    [anon_sym_score] = ACTIONS(1),
    [anon_sym_matches] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_LT_EQ] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_GT_EQ] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_bossbar] = ACTIONS(1),
    [anon_sym_value] = ACTIONS(1),
    [anon_sym_max] = ACTIONS(1),
    [anon_sym_ATs] = ACTIONS(1),
    [anon_sym_ATa] = ACTIONS(1),
    [anon_sym_ATp] = ACTIONS(1),
    [anon_sym_ATr] = ACTIONS(1),
    [anon_sym_ATe] = ACTIONS(1),
    [anon_sym_levels] = ACTIONS(1),
    [anon_sym_points] = ACTIONS(1),
    [anon_sym_DOT_DOT] = ACTIONS(1),
    [aux_sym_quoted_string_token1] = ACTIONS(1),
    [anon_sym_say] = ACTIONS(1),
    [anon_sym_clear] = ACTIONS(1),
    [anon_sym_eclear] = ACTIONS(1),
    [anon_sym_tellraw] = ACTIONS(1),
    [anon_sym_effect] = ACTIONS(1),
    [anon_sym_give] = ACTIONS(1),
    [anon_sym_enchant] = ACTIONS(1),
    [anon_sym_minecraft_COLONspeed] = ACTIONS(1),
    [anon_sym_minecraft_COLONslowness] = ACTIONS(1),
    [anon_sym_minecraft_COLONhaste] = ACTIONS(1),
    [anon_sym_minecraft_COLONmining_fatigue] = ACTIONS(1),
    [anon_sym_minecraft_COLONstrength] = ACTIONS(1),
    [anon_sym_minecraft_COLONinstant_health] = ACTIONS(1),
    [anon_sym_minecraft_COLONinstant_damage] = ACTIONS(1),
    [anon_sym_minecraft_COLONjump_boost] = ACTIONS(1),
    [anon_sym_minecraft_COLONnausea] = ACTIONS(1),
    [anon_sym_minecraft_COLONregeneration] = ACTIONS(1),
    [anon_sym_minecraft_COLONresistance] = ACTIONS(1),
    [anon_sym_minecraft_COLONfire_resistance] = ACTIONS(1),
    [anon_sym_minecraft_COLONwater_breathing] = ACTIONS(1),
    [anon_sym_minecraft_COLONinvisibility] = ACTIONS(1),
    [anon_sym_minecraft_COLONblindness] = ACTIONS(1),
    [anon_sym_minecraft_COLONnight_vision] = ACTIONS(1),
    [anon_sym_minecraft_COLONhunger] = ACTIONS(1),
    [anon_sym_minecraft_COLONweakness] = ACTIONS(1),
    [anon_sym_minecraft_COLONpoison] = ACTIONS(1),
    [anon_sym_minecraft_COLONwither] = ACTIONS(1),
    [anon_sym_minecraft_COLONhealth_boost] = ACTIONS(1),
    [anon_sym_minecraft_COLONabsorption] = ACTIONS(1),
    [anon_sym_minecraft_COLONsaturation] = ACTIONS(1),
    [anon_sym_minecraft_COLONglowing] = ACTIONS(1),
    [anon_sym_minecraft_COLONlevitation] = ACTIONS(1),
    [anon_sym_minecraft_COLONluck] = ACTIONS(1),
    [anon_sym_minecraft_COLONunluck] = ACTIONS(1),
    [anon_sym_minecraft_COLONslow_falling] = ACTIONS(1),
    [anon_sym_minecraft_COLONconduit_power] = ACTIONS(1),
    [anon_sym_minecraft_COLONdolphins_grace] = ACTIONS(1),
    [anon_sym_minecraft_COLONbad_omen] = ACTIONS(1),
    [anon_sym_minecraft_COLONhero_of_the_village] = ACTIONS(1),
    [anon_sym_time] = ACTIONS(1),
    [anon_sym_set] = ACTIONS(1),
    [anon_sym_query] = ACTIONS(1),
    [anon_sym_gms] = ACTIONS(1),
    [anon_sym_gma] = ACTIONS(1),
    [anon_sym_gmsp] = ACTIONS(1),
    [anon_sym_gmc] = ACTIONS(1),
    [anon_sym_day] = ACTIONS(1),
    [anon_sym_night] = ACTIONS(1),
    [anon_sym_noon] = ACTIONS(1),
    [anon_sym_midnight] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(208),
    [sym__definition] = STATE(150),
    [sym_function_definition] = STATE(150),
    [aux_sym_source_file_repeat1] = STATE(150),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_fn] = ACTIONS(5),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 4,
    ACTIONS(9), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(11), 1,
      anon_sym_LBRACK,
    STATE(4), 1,
      sym_selector_arguments,
    ACTIONS(7), 50,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_eyes,
      anon_sym_feet,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_levels,
      anon_sym_points,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [62] = 2,
    ACTIONS(15), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(13), 51,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_eyes,
      anon_sym_feet,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_LBRACK,
      anon_sym_levels,
      anon_sym_points,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [119] = 2,
    ACTIONS(19), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(17), 50,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_eyes,
      anon_sym_feet,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_levels,
      anon_sym_points,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [175] = 2,
    ACTIONS(23), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(21), 50,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_eyes,
      anon_sym_feet,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_levels,
      anon_sym_points,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [231] = 2,
    ACTIONS(27), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(25), 50,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_eyes,
      anon_sym_feet,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_levels,
      anon_sym_points,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [287] = 4,
    ACTIONS(29), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(31), 1,
      aux_sym_quoted_string_token1,
    STATE(243), 2,
      sym_vanilla_effect,
      sym_custom_effect,
    ACTIONS(33), 32,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [332] = 4,
    ACTIONS(31), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(35), 1,
      anon_sym_minecraft_COLON,
    STATE(228), 2,
      sym_vanilla_effect,
      sym_custom_effect,
    ACTIONS(33), 32,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [377] = 4,
    ACTIONS(9), 1,
      sym_identifier,
    ACTIONS(37), 1,
      anon_sym_LBRACK,
    STATE(16), 1,
      sym_selector_arguments,
    ACTIONS(7), 33,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONprotection,
      anon_sym_minecraft_COLONfire_protection,
      anon_sym_minecraft_COLONfeather_falling,
      anon_sym_minecraft_COLONblast_protection,
      anon_sym_minecraft_COLONprojectile_protection,
      anon_sym_minecraft_COLONrespiration,
      anon_sym_minecraft_COLONaqua_affinity,
      anon_sym_minecraft_COLONthorns,
      anon_sym_minecraft_COLONdepth_strider,
      anon_sym_minecraft_COLONfrost_walker,
      anon_sym_minecraft_COLONbinding_curse,
      anon_sym_minecraft_COLONsharpness,
      anon_sym_minecraft_COLONsmite,
      anon_sym_minecraft_COLONbane_of_arthropods,
      anon_sym_minecraft_COLONknockback,
      anon_sym_minecraft_COLONfire_aspect,
      anon_sym_minecraft_COLONlooting,
      anon_sym_minecraft_COLONsweeping,
      anon_sym_minecraft_COLONefficiency,
      anon_sym_minecraft_COLONsilk_touch,
      anon_sym_minecraft_COLONunbreaking,
      anon_sym_minecraft_COLONfortune,
      anon_sym_minecraft_COLONpower,
      anon_sym_minecraft_COLONpunch,
      anon_sym_minecraft_COLONflame,
      anon_sym_minecraft_COLONinfinity,
      anon_sym_minecraft_COLONluck_of_the_sea,
      anon_sym_minecraft_COLONlure,
      anon_sym_minecraft_COLONmending,
      anon_sym_minecraft_COLONvanishing_curse,
      anon_sym_minecraft_COLONsoul_speed,
      anon_sym_minecraft_COLONswift_sneak,
  [422] = 3,
    ACTIONS(39), 1,
      aux_sym_quoted_string_token1,
    STATE(231), 2,
      sym_vanilla_enchant,
      sym_custom_enchant,
    ACTIONS(41), 32,
      anon_sym_minecraft_COLONprotection,
      anon_sym_minecraft_COLONfire_protection,
      anon_sym_minecraft_COLONfeather_falling,
      anon_sym_minecraft_COLONblast_protection,
      anon_sym_minecraft_COLONprojectile_protection,
      anon_sym_minecraft_COLONrespiration,
      anon_sym_minecraft_COLONaqua_affinity,
      anon_sym_minecraft_COLONthorns,
      anon_sym_minecraft_COLONdepth_strider,
      anon_sym_minecraft_COLONfrost_walker,
      anon_sym_minecraft_COLONbinding_curse,
      anon_sym_minecraft_COLONsharpness,
      anon_sym_minecraft_COLONsmite,
      anon_sym_minecraft_COLONbane_of_arthropods,
      anon_sym_minecraft_COLONknockback,
      anon_sym_minecraft_COLONfire_aspect,
      anon_sym_minecraft_COLONlooting,
      anon_sym_minecraft_COLONsweeping,
      anon_sym_minecraft_COLONefficiency,
      anon_sym_minecraft_COLONsilk_touch,
      anon_sym_minecraft_COLONunbreaking,
      anon_sym_minecraft_COLONfortune,
      anon_sym_minecraft_COLONpower,
      anon_sym_minecraft_COLONpunch,
      anon_sym_minecraft_COLONflame,
      anon_sym_minecraft_COLONinfinity,
      anon_sym_minecraft_COLONluck_of_the_sea,
      anon_sym_minecraft_COLONlure,
      anon_sym_minecraft_COLONmending,
      anon_sym_minecraft_COLONvanishing_curse,
      anon_sym_minecraft_COLONsoul_speed,
      anon_sym_minecraft_COLONswift_sneak,
  [464] = 2,
    ACTIONS(15), 1,
      sym_identifier,
    ACTIONS(13), 34,
      anon_sym_LBRACK,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONprotection,
      anon_sym_minecraft_COLONfire_protection,
      anon_sym_minecraft_COLONfeather_falling,
      anon_sym_minecraft_COLONblast_protection,
      anon_sym_minecraft_COLONprojectile_protection,
      anon_sym_minecraft_COLONrespiration,
      anon_sym_minecraft_COLONaqua_affinity,
      anon_sym_minecraft_COLONthorns,
      anon_sym_minecraft_COLONdepth_strider,
      anon_sym_minecraft_COLONfrost_walker,
      anon_sym_minecraft_COLONbinding_curse,
      anon_sym_minecraft_COLONsharpness,
      anon_sym_minecraft_COLONsmite,
      anon_sym_minecraft_COLONbane_of_arthropods,
      anon_sym_minecraft_COLONknockback,
      anon_sym_minecraft_COLONfire_aspect,
      anon_sym_minecraft_COLONlooting,
      anon_sym_minecraft_COLONsweeping,
      anon_sym_minecraft_COLONefficiency,
      anon_sym_minecraft_COLONsilk_touch,
      anon_sym_minecraft_COLONunbreaking,
      anon_sym_minecraft_COLONfortune,
      anon_sym_minecraft_COLONpower,
      anon_sym_minecraft_COLONpunch,
      anon_sym_minecraft_COLONflame,
      anon_sym_minecraft_COLONinfinity,
      anon_sym_minecraft_COLONluck_of_the_sea,
      anon_sym_minecraft_COLONlure,
      anon_sym_minecraft_COLONmending,
      anon_sym_minecraft_COLONvanishing_curse,
      anon_sym_minecraft_COLONsoul_speed,
      anon_sym_minecraft_COLONswift_sneak,
  [504] = 19,
    ACTIONS(43), 1,
      aux_sym_block_token1,
    ACTIONS(45), 1,
      anon_sym_RBRACE,
    ACTIONS(47), 1,
      anon_sym_execute,
    ACTIONS(49), 1,
      anon_sym_xpadd,
    ACTIONS(51), 1,
      anon_sym_xpset,
    ACTIONS(53), 1,
      anon_sym_xpquery,
    ACTIONS(55), 1,
      anon_sym_say,
    ACTIONS(57), 1,
      anon_sym_clear,
    ACTIONS(59), 1,
      anon_sym_eclear,
    ACTIONS(61), 1,
      anon_sym_tellraw,
    ACTIONS(63), 1,
      anon_sym_effect,
    ACTIONS(65), 1,
      anon_sym_enchant,
    ACTIONS(67), 1,
      anon_sym_time,
    ACTIONS(69), 1,
      anon_sym_gms,
    ACTIONS(71), 1,
      anon_sym_gma,
    ACTIONS(73), 1,
      anon_sym_gmsp,
    ACTIONS(75), 1,
      anon_sym_gmc,
    STATE(13), 1,
      aux_sym_block_repeat1,
    STATE(263), 16,
      sym__command,
      sym_execute_command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_enchant_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [577] = 19,
    ACTIONS(77), 1,
      aux_sym_block_token1,
    ACTIONS(80), 1,
      anon_sym_RBRACE,
    ACTIONS(82), 1,
      anon_sym_execute,
    ACTIONS(85), 1,
      anon_sym_xpadd,
    ACTIONS(88), 1,
      anon_sym_xpset,
    ACTIONS(91), 1,
      anon_sym_xpquery,
    ACTIONS(94), 1,
      anon_sym_say,
    ACTIONS(97), 1,
      anon_sym_clear,
    ACTIONS(100), 1,
      anon_sym_eclear,
    ACTIONS(103), 1,
      anon_sym_tellraw,
    ACTIONS(106), 1,
      anon_sym_effect,
    ACTIONS(109), 1,
      anon_sym_enchant,
    ACTIONS(112), 1,
      anon_sym_time,
    ACTIONS(115), 1,
      anon_sym_gms,
    ACTIONS(118), 1,
      anon_sym_gma,
    ACTIONS(121), 1,
      anon_sym_gmsp,
    ACTIONS(124), 1,
      anon_sym_gmc,
    STATE(13), 1,
      aux_sym_block_repeat1,
    STATE(263), 16,
      sym__command,
      sym_execute_command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_enchant_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [650] = 2,
    ACTIONS(23), 1,
      sym_identifier,
    ACTIONS(21), 33,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONprotection,
      anon_sym_minecraft_COLONfire_protection,
      anon_sym_minecraft_COLONfeather_falling,
      anon_sym_minecraft_COLONblast_protection,
      anon_sym_minecraft_COLONprojectile_protection,
      anon_sym_minecraft_COLONrespiration,
      anon_sym_minecraft_COLONaqua_affinity,
      anon_sym_minecraft_COLONthorns,
      anon_sym_minecraft_COLONdepth_strider,
      anon_sym_minecraft_COLONfrost_walker,
      anon_sym_minecraft_COLONbinding_curse,
      anon_sym_minecraft_COLONsharpness,
      anon_sym_minecraft_COLONsmite,
      anon_sym_minecraft_COLONbane_of_arthropods,
      anon_sym_minecraft_COLONknockback,
      anon_sym_minecraft_COLONfire_aspect,
      anon_sym_minecraft_COLONlooting,
      anon_sym_minecraft_COLONsweeping,
      anon_sym_minecraft_COLONefficiency,
      anon_sym_minecraft_COLONsilk_touch,
      anon_sym_minecraft_COLONunbreaking,
      anon_sym_minecraft_COLONfortune,
      anon_sym_minecraft_COLONpower,
      anon_sym_minecraft_COLONpunch,
      anon_sym_minecraft_COLONflame,
      anon_sym_minecraft_COLONinfinity,
      anon_sym_minecraft_COLONluck_of_the_sea,
      anon_sym_minecraft_COLONlure,
      anon_sym_minecraft_COLONmending,
      anon_sym_minecraft_COLONvanishing_curse,
      anon_sym_minecraft_COLONsoul_speed,
      anon_sym_minecraft_COLONswift_sneak,
  [689] = 2,
    ACTIONS(27), 1,
      sym_identifier,
    ACTIONS(25), 33,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONprotection,
      anon_sym_minecraft_COLONfire_protection,
      anon_sym_minecraft_COLONfeather_falling,
      anon_sym_minecraft_COLONblast_protection,
      anon_sym_minecraft_COLONprojectile_protection,
      anon_sym_minecraft_COLONrespiration,
      anon_sym_minecraft_COLONaqua_affinity,
      anon_sym_minecraft_COLONthorns,
      anon_sym_minecraft_COLONdepth_strider,
      anon_sym_minecraft_COLONfrost_walker,
      anon_sym_minecraft_COLONbinding_curse,
      anon_sym_minecraft_COLONsharpness,
      anon_sym_minecraft_COLONsmite,
      anon_sym_minecraft_COLONbane_of_arthropods,
      anon_sym_minecraft_COLONknockback,
      anon_sym_minecraft_COLONfire_aspect,
      anon_sym_minecraft_COLONlooting,
      anon_sym_minecraft_COLONsweeping,
      anon_sym_minecraft_COLONefficiency,
      anon_sym_minecraft_COLONsilk_touch,
      anon_sym_minecraft_COLONunbreaking,
      anon_sym_minecraft_COLONfortune,
      anon_sym_minecraft_COLONpower,
      anon_sym_minecraft_COLONpunch,
      anon_sym_minecraft_COLONflame,
      anon_sym_minecraft_COLONinfinity,
      anon_sym_minecraft_COLONluck_of_the_sea,
      anon_sym_minecraft_COLONlure,
      anon_sym_minecraft_COLONmending,
      anon_sym_minecraft_COLONvanishing_curse,
      anon_sym_minecraft_COLONsoul_speed,
      anon_sym_minecraft_COLONswift_sneak,
  [728] = 2,
    ACTIONS(19), 1,
      sym_identifier,
    ACTIONS(17), 33,
      aux_sym_quoted_string_token1,
      anon_sym_minecraft_COLONprotection,
      anon_sym_minecraft_COLONfire_protection,
      anon_sym_minecraft_COLONfeather_falling,
      anon_sym_minecraft_COLONblast_protection,
      anon_sym_minecraft_COLONprojectile_protection,
      anon_sym_minecraft_COLONrespiration,
      anon_sym_minecraft_COLONaqua_affinity,
      anon_sym_minecraft_COLONthorns,
      anon_sym_minecraft_COLONdepth_strider,
      anon_sym_minecraft_COLONfrost_walker,
      anon_sym_minecraft_COLONbinding_curse,
      anon_sym_minecraft_COLONsharpness,
      anon_sym_minecraft_COLONsmite,
      anon_sym_minecraft_COLONbane_of_arthropods,
      anon_sym_minecraft_COLONknockback,
      anon_sym_minecraft_COLONfire_aspect,
      anon_sym_minecraft_COLONlooting,
      anon_sym_minecraft_COLONsweeping,
      anon_sym_minecraft_COLONefficiency,
      anon_sym_minecraft_COLONsilk_touch,
      anon_sym_minecraft_COLONunbreaking,
      anon_sym_minecraft_COLONfortune,
      anon_sym_minecraft_COLONpower,
      anon_sym_minecraft_COLONpunch,
      anon_sym_minecraft_COLONflame,
      anon_sym_minecraft_COLONinfinity,
      anon_sym_minecraft_COLONluck_of_the_sea,
      anon_sym_minecraft_COLONlure,
      anon_sym_minecraft_COLONmending,
      anon_sym_minecraft_COLONvanishing_curse,
      anon_sym_minecraft_COLONsoul_speed,
      anon_sym_minecraft_COLONswift_sneak,
  [767] = 19,
    ACTIONS(43), 1,
      aux_sym_block_token1,
    ACTIONS(47), 1,
      anon_sym_execute,
    ACTIONS(49), 1,
      anon_sym_xpadd,
    ACTIONS(51), 1,
      anon_sym_xpset,
    ACTIONS(53), 1,
      anon_sym_xpquery,
    ACTIONS(55), 1,
      anon_sym_say,
    ACTIONS(57), 1,
      anon_sym_clear,
    ACTIONS(59), 1,
      anon_sym_eclear,
    ACTIONS(61), 1,
      anon_sym_tellraw,
    ACTIONS(63), 1,
      anon_sym_effect,
    ACTIONS(65), 1,
      anon_sym_enchant,
    ACTIONS(67), 1,
      anon_sym_time,
    ACTIONS(69), 1,
      anon_sym_gms,
    ACTIONS(71), 1,
      anon_sym_gma,
    ACTIONS(73), 1,
      anon_sym_gmsp,
    ACTIONS(75), 1,
      anon_sym_gmc,
    ACTIONS(127), 1,
      anon_sym_RBRACE,
    STATE(12), 1,
      aux_sym_block_repeat1,
    STATE(263), 16,
      sym__command,
      sym_execute_command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_enchant_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [840] = 2,
    STATE(247), 1,
      sym_vanilla_effect,
    ACTIONS(33), 32,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [878] = 2,
    STATE(226), 1,
      sym_vanilla_effect,
    ACTIONS(33), 32,
      anon_sym_minecraft_COLONspeed,
      anon_sym_minecraft_COLONslowness,
      anon_sym_minecraft_COLONhaste,
      anon_sym_minecraft_COLONmining_fatigue,
      anon_sym_minecraft_COLONstrength,
      anon_sym_minecraft_COLONinstant_health,
      anon_sym_minecraft_COLONinstant_damage,
      anon_sym_minecraft_COLONjump_boost,
      anon_sym_minecraft_COLONnausea,
      anon_sym_minecraft_COLONregeneration,
      anon_sym_minecraft_COLONresistance,
      anon_sym_minecraft_COLONfire_resistance,
      anon_sym_minecraft_COLONwater_breathing,
      anon_sym_minecraft_COLONinvisibility,
      anon_sym_minecraft_COLONblindness,
      anon_sym_minecraft_COLONnight_vision,
      anon_sym_minecraft_COLONhunger,
      anon_sym_minecraft_COLONweakness,
      anon_sym_minecraft_COLONpoison,
      anon_sym_minecraft_COLONwither,
      anon_sym_minecraft_COLONhealth_boost,
      anon_sym_minecraft_COLONabsorption,
      anon_sym_minecraft_COLONsaturation,
      anon_sym_minecraft_COLONglowing,
      anon_sym_minecraft_COLONlevitation,
      anon_sym_minecraft_COLONluck,
      anon_sym_minecraft_COLONunluck,
      anon_sym_minecraft_COLONslow_falling,
      anon_sym_minecraft_COLONconduit_power,
      anon_sym_minecraft_COLONdolphins_grace,
      anon_sym_minecraft_COLONbad_omen,
      anon_sym_minecraft_COLONhero_of_the_village,
  [916] = 17,
    ACTIONS(49), 1,
      anon_sym_xpadd,
    ACTIONS(51), 1,
      anon_sym_xpset,
    ACTIONS(53), 1,
      anon_sym_xpquery,
    ACTIONS(55), 1,
      anon_sym_say,
    ACTIONS(57), 1,
      anon_sym_clear,
    ACTIONS(59), 1,
      anon_sym_eclear,
    ACTIONS(61), 1,
      anon_sym_tellraw,
    ACTIONS(63), 1,
      anon_sym_effect,
    ACTIONS(65), 1,
      anon_sym_enchant,
    ACTIONS(67), 1,
      anon_sym_time,
    ACTIONS(69), 1,
      anon_sym_gms,
    ACTIONS(71), 1,
      anon_sym_gma,
    ACTIONS(73), 1,
      anon_sym_gmsp,
    ACTIONS(75), 1,
      anon_sym_gmc,
    ACTIONS(129), 1,
      aux_sym_block_token1,
    ACTIONS(131), 1,
      anon_sym_execute,
    STATE(98), 16,
      sym__command,
      sym_execute_command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_enchant_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [983] = 15,
    ACTIONS(133), 1,
      anon_sym_SEMI,
    ACTIONS(135), 1,
      anon_sym_as,
    ACTIONS(138), 1,
      anon_sym_at,
    ACTIONS(141), 1,
      anon_sym_align,
    ACTIONS(144), 1,
      anon_sym_anchored,
    ACTIONS(147), 1,
      anon_sym_facing,
    ACTIONS(150), 1,
      anon_sym_in,
    ACTIONS(153), 1,
      anon_sym_positioned,
    ACTIONS(156), 1,
      anon_sym_rotated,
    ACTIONS(159), 1,
      anon_sym_if,
    ACTIONS(162), 1,
      anon_sym_unless,
    ACTIONS(165), 1,
      anon_sym_store,
    ACTIONS(168), 1,
      anon_sym_run,
    STATE(21), 2,
      sym_execute_subcommand,
      aux_sym_execute_command_repeat1,
    STATE(67), 15,
      sym_execute_as,
      sym_execute_at,
      sym_execute_align,
      sym_execute_anchored,
      sym_execute_facing,
      sym_execute_facing_entity,
      sym_execute_in,
      sym_execute_positioned,
      sym_execute_positioned_as,
      sym_execute_rotated,
      sym_execute_rotated_as,
      sym_execute_if,
      sym_execute_unless,
      sym_execute_store,
      sym_execute_run,
  [1044] = 16,
    ACTIONS(69), 1,
      anon_sym_gms,
    ACTIONS(171), 1,
      anon_sym_execute,
    ACTIONS(173), 1,
      anon_sym_xpadd,
    ACTIONS(175), 1,
      anon_sym_xpset,
    ACTIONS(177), 1,
      anon_sym_xpquery,
    ACTIONS(179), 1,
      anon_sym_say,
    ACTIONS(181), 1,
      anon_sym_clear,
    ACTIONS(183), 1,
      anon_sym_eclear,
    ACTIONS(185), 1,
      anon_sym_tellraw,
    ACTIONS(187), 1,
      anon_sym_effect,
    ACTIONS(189), 1,
      anon_sym_enchant,
    ACTIONS(191), 1,
      anon_sym_time,
    ACTIONS(193), 1,
      anon_sym_gma,
    ACTIONS(195), 1,
      anon_sym_gmsp,
    ACTIONS(197), 1,
      anon_sym_gmc,
    STATE(64), 15,
      sym_execute_command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_enchant_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [1107] = 15,
    ACTIONS(199), 1,
      anon_sym_SEMI,
    ACTIONS(201), 1,
      anon_sym_as,
    ACTIONS(203), 1,
      anon_sym_at,
    ACTIONS(205), 1,
      anon_sym_align,
    ACTIONS(207), 1,
      anon_sym_anchored,
    ACTIONS(209), 1,
      anon_sym_facing,
    ACTIONS(211), 1,
      anon_sym_in,
    ACTIONS(213), 1,
      anon_sym_positioned,
    ACTIONS(215), 1,
      anon_sym_rotated,
    ACTIONS(217), 1,
      anon_sym_if,
    ACTIONS(219), 1,
      anon_sym_unless,
    ACTIONS(221), 1,
      anon_sym_store,
    ACTIONS(223), 1,
      anon_sym_run,
    STATE(21), 2,
      sym_execute_subcommand,
      aux_sym_execute_command_repeat1,
    STATE(67), 15,
      sym_execute_as,
      sym_execute_at,
      sym_execute_align,
      sym_execute_anchored,
      sym_execute_facing,
      sym_execute_facing_entity,
      sym_execute_in,
      sym_execute_positioned,
      sym_execute_positioned_as,
      sym_execute_rotated,
      sym_execute_rotated_as,
      sym_execute_if,
      sym_execute_unless,
      sym_execute_store,
      sym_execute_run,
  [1168] = 3,
    STATE(21), 2,
      sym_execute_subcommand,
      aux_sym_execute_command_repeat1,
    ACTIONS(199), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
    STATE(67), 15,
      sym_execute_as,
      sym_execute_at,
      sym_execute_align,
      sym_execute_anchored,
      sym_execute_facing,
      sym_execute_facing_entity,
      sym_execute_in,
      sym_execute_positioned,
      sym_execute_positioned_as,
      sym_execute_rotated,
      sym_execute_rotated_as,
      sym_execute_if,
      sym_execute_unless,
      sym_execute_store,
      sym_execute_run,
  [1205] = 16,
    ACTIONS(69), 1,
      anon_sym_gms,
    ACTIONS(173), 1,
      anon_sym_xpadd,
    ACTIONS(175), 1,
      anon_sym_xpset,
    ACTIONS(177), 1,
      anon_sym_xpquery,
    ACTIONS(179), 1,
      anon_sym_say,
    ACTIONS(181), 1,
      anon_sym_clear,
    ACTIONS(183), 1,
      anon_sym_eclear,
    ACTIONS(185), 1,
      anon_sym_tellraw,
    ACTIONS(187), 1,
      anon_sym_effect,
    ACTIONS(189), 1,
      anon_sym_enchant,
    ACTIONS(191), 1,
      anon_sym_time,
    ACTIONS(193), 1,
      anon_sym_gma,
    ACTIONS(195), 1,
      anon_sym_gmsp,
    ACTIONS(197), 1,
      anon_sym_gmc,
    ACTIONS(225), 1,
      anon_sym_execute,
    STATE(64), 15,
      sym_execute_command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_enchant_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [1268] = 14,
    ACTIONS(201), 1,
      anon_sym_as,
    ACTIONS(203), 1,
      anon_sym_at,
    ACTIONS(205), 1,
      anon_sym_align,
    ACTIONS(207), 1,
      anon_sym_anchored,
    ACTIONS(209), 1,
      anon_sym_facing,
    ACTIONS(211), 1,
      anon_sym_in,
    ACTIONS(213), 1,
      anon_sym_positioned,
    ACTIONS(215), 1,
      anon_sym_rotated,
    ACTIONS(217), 1,
      anon_sym_if,
    ACTIONS(219), 1,
      anon_sym_unless,
    ACTIONS(221), 1,
      anon_sym_store,
    ACTIONS(223), 1,
      anon_sym_run,
    STATE(23), 2,
      sym_execute_subcommand,
      aux_sym_execute_command_repeat1,
    STATE(67), 15,
      sym_execute_as,
      sym_execute_at,
      sym_execute_align,
      sym_execute_anchored,
      sym_execute_facing,
      sym_execute_facing_entity,
      sym_execute_in,
      sym_execute_positioned,
      sym_execute_positioned_as,
      sym_execute_rotated,
      sym_execute_rotated_as,
      sym_execute_if,
      sym_execute_unless,
      sym_execute_store,
      sym_execute_run,
  [1326] = 14,
    ACTIONS(201), 1,
      anon_sym_as,
    ACTIONS(203), 1,
      anon_sym_at,
    ACTIONS(205), 1,
      anon_sym_align,
    ACTIONS(207), 1,
      anon_sym_anchored,
    ACTIONS(209), 1,
      anon_sym_facing,
    ACTIONS(211), 1,
      anon_sym_in,
    ACTIONS(213), 1,
      anon_sym_positioned,
    ACTIONS(215), 1,
      anon_sym_rotated,
    ACTIONS(217), 1,
      anon_sym_if,
    ACTIONS(219), 1,
      anon_sym_unless,
    ACTIONS(221), 1,
      anon_sym_store,
    ACTIONS(223), 1,
      anon_sym_run,
    STATE(24), 2,
      sym_execute_subcommand,
      aux_sym_execute_command_repeat1,
    STATE(67), 15,
      sym_execute_as,
      sym_execute_at,
      sym_execute_align,
      sym_execute_anchored,
      sym_execute_facing,
      sym_execute_facing_entity,
      sym_execute_in,
      sym_execute_positioned,
      sym_execute_positioned_as,
      sym_execute_rotated,
      sym_execute_rotated_as,
      sym_execute_if,
      sym_execute_unless,
      sym_execute_store,
      sym_execute_run,
  [1384] = 3,
    ACTIONS(229), 1,
      anon_sym_in,
    ACTIONS(231), 6,
      anon_sym_byte,
      anon_sym_short,
      anon_sym_int,
      anon_sym_long,
      anon_sym_float,
      anon_sym_double,
    ACTIONS(227), 12,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1410] = 2,
    ACTIONS(235), 1,
      anon_sym_in,
    ACTIONS(233), 18,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_byte,
      anon_sym_short,
      anon_sym_int,
      anon_sym_long,
      anon_sym_float,
      anon_sym_double,
      anon_sym_run,
  [1434] = 2,
    ACTIONS(239), 1,
      anon_sym_in,
    ACTIONS(237), 18,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_byte,
      anon_sym_short,
      anon_sym_int,
      anon_sym_long,
      anon_sym_float,
      anon_sym_double,
      anon_sym_run,
  [1458] = 2,
    ACTIONS(243), 1,
      anon_sym_in,
    ACTIONS(241), 18,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_byte,
      anon_sym_short,
      anon_sym_int,
      anon_sym_long,
      anon_sym_float,
      anon_sym_double,
      anon_sym_run,
  [1482] = 2,
    ACTIONS(247), 1,
      anon_sym_in,
    ACTIONS(245), 18,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_byte,
      anon_sym_short,
      anon_sym_int,
      anon_sym_long,
      anon_sym_float,
      anon_sym_double,
      anon_sym_run,
  [1506] = 2,
    ACTIONS(251), 1,
      anon_sym_in,
    ACTIONS(249), 18,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_byte,
      anon_sym_short,
      anon_sym_int,
      anon_sym_long,
      anon_sym_float,
      anon_sym_double,
      anon_sym_run,
  [1530] = 3,
    ACTIONS(255), 1,
      anon_sym_DASH,
    ACTIONS(257), 1,
      sym_number,
    ACTIONS(253), 16,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
  [1555] = 3,
    ACTIONS(261), 1,
      anon_sym_DASH,
    ACTIONS(263), 1,
      sym_number,
    ACTIONS(259), 16,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
  [1580] = 1,
    ACTIONS(265), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1600] = 2,
    ACTIONS(267), 1,
      aux_sym_block_token1,
    ACTIONS(269), 16,
      anon_sym_RBRACE,
      anon_sym_execute,
      anon_sym_xpadd,
      anon_sym_xpset,
      anon_sym_xpquery,
      anon_sym_say,
      anon_sym_clear,
      anon_sym_eclear,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_enchant,
      anon_sym_time,
      anon_sym_gms,
      anon_sym_gma,
      anon_sym_gmsp,
      anon_sym_gmc,
  [1622] = 1,
    ACTIONS(271), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1642] = 1,
    ACTIONS(273), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1662] = 1,
    ACTIONS(275), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1682] = 1,
    ACTIONS(277), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1702] = 2,
    ACTIONS(279), 1,
      aux_sym_block_token1,
    ACTIONS(80), 16,
      anon_sym_RBRACE,
      anon_sym_execute,
      anon_sym_xpadd,
      anon_sym_xpset,
      anon_sym_xpquery,
      anon_sym_say,
      anon_sym_clear,
      anon_sym_eclear,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_enchant,
      anon_sym_time,
      anon_sym_gms,
      anon_sym_gma,
      anon_sym_gmsp,
      anon_sym_gmc,
  [1724] = 1,
    ACTIONS(282), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1744] = 1,
    ACTIONS(284), 17,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
      anon_sym_TILDE,
      anon_sym_CARET,
      anon_sym_masked,
      sym_number,
  [1764] = 2,
    ACTIONS(288), 2,
      anon_sym_eyes,
      anon_sym_feet,
    ACTIONS(286), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1784] = 2,
    ACTIONS(292), 1,
      anon_sym_LBRACK,
    ACTIONS(290), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1803] = 2,
    ACTIONS(296), 1,
      anon_sym_LBRACK,
    ACTIONS(294), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1822] = 2,
    ACTIONS(300), 1,
      anon_sym_masked,
    ACTIONS(298), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1841] = 2,
    ACTIONS(304), 1,
      anon_sym_DOT_DOT,
    ACTIONS(302), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1860] = 2,
    ACTIONS(308), 1,
      sym_number,
    ACTIONS(306), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1879] = 1,
    ACTIONS(310), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1895] = 1,
    ACTIONS(312), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1911] = 1,
    ACTIONS(314), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1927] = 1,
    ACTIONS(316), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1943] = 1,
    ACTIONS(318), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1959] = 1,
    ACTIONS(320), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1975] = 1,
    ACTIONS(322), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [1991] = 1,
    ACTIONS(324), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2007] = 1,
    ACTIONS(326), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2023] = 1,
    ACTIONS(328), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2039] = 1,
    ACTIONS(330), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2055] = 1,
    ACTIONS(332), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2071] = 1,
    ACTIONS(334), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2087] = 1,
    ACTIONS(336), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2103] = 1,
    ACTIONS(338), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2119] = 1,
    ACTIONS(340), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2135] = 1,
    ACTIONS(342), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2151] = 1,
    ACTIONS(344), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2167] = 1,
    ACTIONS(346), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2183] = 1,
    ACTIONS(348), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2199] = 1,
    ACTIONS(350), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2215] = 1,
    ACTIONS(352), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2231] = 1,
    ACTIONS(354), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2247] = 1,
    ACTIONS(356), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2263] = 1,
    ACTIONS(358), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2279] = 1,
    ACTIONS(360), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2295] = 1,
    ACTIONS(362), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2311] = 1,
    ACTIONS(364), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2327] = 1,
    ACTIONS(366), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2343] = 1,
    ACTIONS(368), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2359] = 1,
    ACTIONS(370), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2375] = 1,
    ACTIONS(372), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2391] = 1,
    ACTIONS(374), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2407] = 1,
    ACTIONS(376), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2423] = 1,
    ACTIONS(378), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2439] = 1,
    ACTIONS(380), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2455] = 1,
    ACTIONS(382), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2471] = 1,
    ACTIONS(384), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2487] = 1,
    ACTIONS(386), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2503] = 1,
    ACTIONS(388), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2519] = 1,
    ACTIONS(390), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2535] = 1,
    ACTIONS(392), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2551] = 1,
    ACTIONS(394), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2567] = 1,
    ACTIONS(306), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2583] = 1,
    ACTIONS(396), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2599] = 1,
    ACTIONS(398), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2615] = 1,
    ACTIONS(400), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2631] = 1,
    ACTIONS(402), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2647] = 1,
    ACTIONS(404), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2663] = 1,
    ACTIONS(406), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2679] = 1,
    ACTIONS(408), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2695] = 1,
    ACTIONS(410), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2711] = 1,
    ACTIONS(412), 13,
      anon_sym_SEMI,
      anon_sym_as,
      anon_sym_at,
      anon_sym_align,
      anon_sym_anchored,
      anon_sym_facing,
      anon_sym_in,
      anon_sym_positioned,
      anon_sym_rotated,
      anon_sym_if,
      anon_sym_unless,
      anon_sym_store,
      anon_sym_run,
  [2727] = 8,
    ACTIONS(414), 1,
      anon_sym_entity,
    ACTIONS(416), 1,
      anon_sym_block,
    ACTIONS(418), 1,
      anon_sym_blocks,
    ACTIONS(420), 1,
      anon_sym_data,
    ACTIONS(422), 1,
      anon_sym_predicate,
    ACTIONS(424), 1,
      anon_sym_score,
    STATE(93), 1,
      sym_execute_condition,
    STATE(95), 6,
      sym_condition_block,
      sym_condition_blocks,
      sym_condition_data,
      sym_condition_entity,
      sym_condition_predicate,
      sym_condition_score,
  [2757] = 8,
    ACTIONS(414), 1,
      anon_sym_entity,
    ACTIONS(416), 1,
      anon_sym_block,
    ACTIONS(418), 1,
      anon_sym_blocks,
    ACTIONS(420), 1,
      anon_sym_data,
    ACTIONS(422), 1,
      anon_sym_predicate,
    ACTIONS(424), 1,
      anon_sym_score,
    STATE(97), 1,
      sym_execute_condition,
    STATE(95), 6,
      sym_condition_block,
      sym_condition_blocks,
      sym_condition_data,
      sym_condition_entity,
      sym_condition_predicate,
      sym_condition_score,
  [2787] = 8,
    ACTIONS(426), 1,
      anon_sym_entity,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(432), 1,
      sym_number,
    STATE(79), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(117), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2816] = 8,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(432), 1,
      sym_number,
    ACTIONS(434), 1,
      anon_sym_as,
    STATE(82), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(117), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2845] = 8,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(436), 1,
      anon_sym_as,
    ACTIONS(438), 1,
      sym_number,
    STATE(83), 1,
      sym_rotation,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(118), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2874] = 7,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(440), 1,
      sym_number,
    STATE(200), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(122), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2900] = 7,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(442), 1,
      sym_number,
    STATE(222), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(115), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2926] = 7,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(442), 1,
      sym_number,
    STATE(235), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(115), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2952] = 7,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(432), 1,
      sym_number,
    STATE(113), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(117), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [2978] = 7,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(432), 1,
      sym_number,
    STATE(114), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(117), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3004] = 7,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(432), 1,
      sym_number,
    STATE(48), 1,
      sym_position,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(117), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3030] = 6,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(444), 1,
      sym_number,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(121), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3053] = 6,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(446), 1,
      sym_number,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(36), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3076] = 6,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(448), 1,
      sym_number,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(116), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3099] = 6,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(450), 1,
      sym_number,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(61), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3122] = 6,
    ACTIONS(454), 1,
      anon_sym_DOT_DOT,
    ACTIONS(456), 1,
      sym_number,
    STATE(9), 1,
      sym_selector_type,
    STATE(89), 1,
      sym_range_value,
    STATE(246), 1,
      sym_target_selector,
    ACTIONS(452), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3145] = 6,
    ACTIONS(458), 1,
      anon_sym_TILDE,
    ACTIONS(460), 1,
      anon_sym_CARET,
    ACTIONS(462), 1,
      sym_number,
    STATE(191), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(192), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(205), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3168] = 6,
    ACTIONS(464), 1,
      anon_sym_TILDE,
    ACTIONS(466), 1,
      anon_sym_CARET,
    ACTIONS(468), 1,
      sym_number,
    STATE(217), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(218), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(244), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3191] = 6,
    ACTIONS(428), 1,
      anon_sym_TILDE,
    ACTIONS(430), 1,
      anon_sym_CARET,
    ACTIONS(470), 1,
      sym_number,
    STATE(38), 2,
      sym_relative_coordinate_plain,
      sym_relative_coordinate_offset,
    STATE(41), 2,
      sym_local_coordinate_plain,
      sym_local_coordinate_offset,
    STATE(120), 3,
      sym__coordinate,
      sym_relative_coordinate,
      sym_local_coordinate,
  [3214] = 4,
    ACTIONS(474), 1,
      anon_sym_give,
    STATE(2), 1,
      sym_selector_type,
    STATE(8), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3231] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(58), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3245] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(173), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3259] = 3,
    STATE(9), 1,
      sym_selector_type,
    STATE(240), 1,
      sym_target_selector,
    ACTIONS(452), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3273] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(180), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3287] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(59), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3301] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(60), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3315] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(68), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3329] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(62), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3343] = 3,
    STATE(181), 1,
      sym_selector_type,
    STATE(255), 1,
      sym_target_selector,
    ACTIONS(476), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3357] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(179), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3371] = 3,
    STATE(9), 1,
      sym_selector_type,
    STATE(245), 1,
      sym_target_selector,
    ACTIONS(452), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3385] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(99), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3399] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(102), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3413] = 3,
    STATE(176), 1,
      sym_selector_type,
    STATE(256), 1,
      sym_target_selector,
    ACTIONS(478), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3427] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(71), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3441] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(7), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3455] = 3,
    STATE(9), 1,
      sym_selector_type,
    STATE(10), 1,
      sym_target_selector,
    ACTIONS(452), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3469] = 3,
    STATE(181), 1,
      sym_selector_type,
    STATE(214), 1,
      sym_target_selector,
    ACTIONS(476), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3483] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(55), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3497] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(45), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3511] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(56), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3525] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(57), 1,
      sym_target_selector,
    ACTIONS(472), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [3539] = 5,
    ACTIONS(480), 1,
      anon_sym_DOT_DOT,
    ACTIONS(482), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(484), 1,
      sym_identifier,
    ACTIONS(486), 1,
      sym_number,
    STATE(186), 2,
      sym_range_value,
      sym_quoted_string,
  [3556] = 2,
    ACTIONS(490), 2,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(488), 4,
      anon_sym_matches,
      anon_sym_LT_EQ,
      anon_sym_EQ,
      anon_sym_GT_EQ,
  [3567] = 3,
    ACTIONS(494), 1,
      sym_number,
    STATE(53), 1,
      sym_time_unit,
    ACTIONS(492), 4,
      anon_sym_day,
      anon_sym_night,
      anon_sym_noon,
      anon_sym_midnight,
  [3580] = 6,
    ACTIONS(496), 1,
      anon_sym_entity,
    ACTIONS(498), 1,
      anon_sym_block,
    ACTIONS(500), 1,
      anon_sym_storage,
    ACTIONS(502), 1,
      anon_sym_score,
    ACTIONS(504), 1,
      anon_sym_bossbar,
    STATE(28), 1,
      sym_store_target,
  [3599] = 3,
    ACTIONS(5), 1,
      anon_sym_fn,
    ACTIONS(506), 1,
      ts_builtin_sym_end,
    STATE(152), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [3611] = 3,
    ACTIONS(510), 1,
      sym_identifier,
    STATE(81), 1,
      sym_dimension,
    ACTIONS(508), 3,
      anon_sym_minecraft_COLONoverworld,
      anon_sym_minecraft_COLONthe_nether,
      anon_sym_minecraft_COLONthe_end,
  [3623] = 3,
    ACTIONS(512), 1,
      ts_builtin_sym_end,
    ACTIONS(514), 1,
      anon_sym_fn,
    STATE(152), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [3635] = 4,
    ACTIONS(517), 1,
      anon_sym_RBRACK,
    ACTIONS(519), 1,
      sym_identifier,
    STATE(163), 1,
      aux_sym_condition_block_repeat1,
    STATE(182), 1,
      sym_block_state,
  [3648] = 4,
    ACTIONS(521), 1,
      anon_sym_RBRACK,
    ACTIONS(523), 1,
      sym_identifier,
    STATE(154), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3661] = 4,
    ACTIONS(519), 1,
      sym_identifier,
    ACTIONS(526), 1,
      anon_sym_RBRACK,
    STATE(153), 1,
      aux_sym_condition_block_repeat1,
    STATE(182), 1,
      sym_block_state,
  [3674] = 4,
    ACTIONS(528), 1,
      anon_sym_RBRACK,
    ACTIONS(530), 1,
      sym_identifier,
    STATE(154), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3687] = 2,
    ACTIONS(534), 1,
      anon_sym_DOT_DOT,
    ACTIONS(532), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3696] = 4,
    ACTIONS(517), 1,
      anon_sym_RBRACK,
    ACTIONS(519), 1,
      sym_identifier,
    STATE(161), 1,
      aux_sym_condition_block_repeat1,
    STATE(182), 1,
      sym_block_state,
  [3709] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(536), 1,
      anon_sym_RBRACK,
    STATE(156), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3722] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(538), 1,
      anon_sym_RBRACK,
    STATE(154), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3735] = 4,
    ACTIONS(519), 1,
      sym_identifier,
    ACTIONS(540), 1,
      anon_sym_RBRACK,
    STATE(163), 1,
      aux_sym_condition_block_repeat1,
    STATE(182), 1,
      sym_block_state,
  [3748] = 3,
    ACTIONS(482), 1,
      aux_sym_quoted_string_token1,
    STATE(175), 1,
      sym_quoted_string,
    ACTIONS(542), 2,
      sym_identifier,
      sym_number,
  [3759] = 4,
    ACTIONS(544), 1,
      anon_sym_RBRACK,
    ACTIONS(546), 1,
      sym_identifier,
    STATE(163), 1,
      aux_sym_condition_block_repeat1,
    STATE(182), 1,
      sym_block_state,
  [3772] = 4,
    ACTIONS(259), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(549), 1,
      anon_sym_DASH,
    ACTIONS(551), 1,
      sym_identifier,
    ACTIONS(553), 1,
      sym_number,
  [3785] = 4,
    ACTIONS(253), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(555), 1,
      anon_sym_DASH,
    ACTIONS(557), 1,
      sym_identifier,
    ACTIONS(559), 1,
      sym_number,
  [3798] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(561), 1,
      anon_sym_RBRACK,
    STATE(167), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3811] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(563), 1,
      anon_sym_RBRACK,
    STATE(154), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3824] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(565), 1,
      anon_sym_RBRACK,
    STATE(169), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3837] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(567), 1,
      anon_sym_RBRACK,
    STATE(154), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3850] = 4,
    ACTIONS(530), 1,
      sym_identifier,
    ACTIONS(569), 1,
      anon_sym_RBRACK,
    STATE(160), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(172), 1,
      sym_selector_argument,
  [3863] = 2,
    ACTIONS(571), 1,
      sym_number,
    ACTIONS(306), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3872] = 2,
    ACTIONS(573), 1,
      anon_sym_COMMA,
    ACTIONS(575), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [3880] = 2,
    STATE(65), 1,
      sym_xp_type,
    ACTIONS(577), 2,
      anon_sym_levels,
      anon_sym_points,
  [3888] = 1,
    ACTIONS(398), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3894] = 1,
    ACTIONS(579), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3900] = 3,
    ACTIONS(9), 1,
      sym_text,
    ACTIONS(581), 1,
      anon_sym_LBRACK,
    STATE(219), 1,
      sym_selector_arguments,
  [3910] = 3,
    ACTIONS(259), 1,
      sym_nbt_path,
    ACTIONS(583), 1,
      anon_sym_DASH,
    ACTIONS(585), 1,
      sym_number,
  [3920] = 3,
    ACTIONS(253), 1,
      sym_nbt_path,
    ACTIONS(587), 1,
      anon_sym_DASH,
    ACTIONS(589), 1,
      sym_number,
  [3930] = 2,
    STATE(66), 1,
      sym_xp_type,
    ACTIONS(577), 2,
      anon_sym_levels,
      anon_sym_points,
  [3938] = 2,
    STATE(101), 1,
      sym_xp_type,
    ACTIONS(577), 2,
      anon_sym_levels,
      anon_sym_points,
  [3946] = 3,
    ACTIONS(7), 1,
      sym_nbt_path,
    ACTIONS(591), 1,
      anon_sym_LBRACK,
    STATE(248), 1,
      sym_selector_arguments,
  [3956] = 2,
    ACTIONS(593), 1,
      anon_sym_COMMA,
    ACTIONS(595), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [3964] = 3,
    ACTIONS(597), 1,
      anon_sym_entity,
    ACTIONS(599), 1,
      anon_sym_block,
    ACTIONS(601), 1,
      anon_sym_storage,
  [3974] = 1,
    ACTIONS(306), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3980] = 1,
    ACTIONS(603), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3986] = 1,
    ACTIONS(532), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [3992] = 1,
    ACTIONS(544), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [3997] = 1,
    ACTIONS(605), 2,
      anon_sym_value,
      anon_sym_max,
  [4002] = 1,
    ACTIONS(607), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [4007] = 1,
    ACTIONS(521), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [4012] = 2,
    ACTIONS(271), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(609), 1,
      sym_identifier,
  [4019] = 2,
    ACTIONS(277), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(611), 1,
      sym_identifier,
  [4026] = 2,
    ACTIONS(273), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(613), 1,
      sym_identifier,
  [4033] = 2,
    ACTIONS(275), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(615), 1,
      sym_identifier,
  [4040] = 2,
    ACTIONS(282), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(617), 1,
      sym_identifier,
  [4047] = 2,
    ACTIONS(284), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(619), 1,
      sym_identifier,
  [4054] = 1,
    ACTIONS(621), 2,
      anon_sym_result,
      anon_sym_success,
  [4059] = 1,
    ACTIONS(13), 2,
      anon_sym_LBRACK,
      sym_nbt_path,
  [4064] = 1,
    ACTIONS(15), 2,
      anon_sym_LBRACK,
      sym_text,
  [4069] = 2,
    ACTIONS(623), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(625), 1,
      sym_identifier,
  [4076] = 1,
    ACTIONS(627), 2,
      anon_sym_eyes,
      anon_sym_feet,
  [4081] = 1,
    ACTIONS(629), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [4086] = 2,
    ACTIONS(631), 1,
      anon_sym_LBRACE,
    STATE(189), 1,
      sym_block,
  [4093] = 1,
    ACTIONS(633), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [4098] = 2,
    ACTIONS(265), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(635), 1,
      sym_identifier,
  [4105] = 2,
    ACTIONS(637), 1,
      anon_sym_set,
    ACTIONS(639), 1,
      anon_sym_query,
  [4112] = 1,
    ACTIONS(641), 1,
      sym_number,
  [4116] = 1,
    ACTIONS(643), 1,
      ts_builtin_sym_end,
  [4120] = 1,
    ACTIONS(645), 1,
      sym_text,
  [4124] = 1,
    ACTIONS(647), 1,
      sym_number,
  [4128] = 1,
    ACTIONS(649), 1,
      sym_number,
  [4132] = 1,
    ACTIONS(651), 1,
      sym_number,
  [4136] = 1,
    ACTIONS(653), 1,
      sym_identifier,
  [4140] = 1,
    ACTIONS(655), 1,
      sym_nbt_path,
  [4144] = 1,
    ACTIONS(657), 1,
      sym_number,
  [4148] = 1,
    ACTIONS(659), 1,
      sym_number,
  [4152] = 1,
    ACTIONS(271), 1,
      sym_nbt_path,
  [4156] = 1,
    ACTIONS(277), 1,
      sym_nbt_path,
  [4160] = 1,
    ACTIONS(17), 1,
      sym_text,
  [4164] = 1,
    ACTIONS(273), 1,
      sym_nbt_path,
  [4168] = 1,
    ACTIONS(275), 1,
      sym_nbt_path,
  [4172] = 1,
    ACTIONS(661), 1,
      sym_nbt_path,
  [4176] = 1,
    ACTIONS(21), 1,
      sym_text,
  [4180] = 1,
    ACTIONS(282), 1,
      sym_nbt_path,
  [4184] = 1,
    ACTIONS(284), 1,
      sym_nbt_path,
  [4188] = 1,
    ACTIONS(663), 1,
      sym_number,
  [4192] = 1,
    ACTIONS(25), 1,
      sym_text,
  [4196] = 1,
    ACTIONS(665), 1,
      sym_number,
  [4200] = 1,
    ACTIONS(667), 1,
      sym_number,
  [4204] = 1,
    ACTIONS(669), 1,
      sym_number,
  [4208] = 1,
    ACTIONS(671), 1,
      sym_number,
  [4212] = 1,
    ACTIONS(673), 1,
      sym_number,
  [4216] = 1,
    ACTIONS(675), 1,
      sym_number,
  [4220] = 1,
    ACTIONS(677), 1,
      sym_number,
  [4224] = 1,
    ACTIONS(679), 1,
      sym_nbt_path,
  [4228] = 1,
    ACTIONS(681), 1,
      sym_number,
  [4232] = 1,
    ACTIONS(683), 1,
      sym_align_axes,
  [4236] = 1,
    ACTIONS(685), 1,
      anon_sym_EQ,
  [4240] = 1,
    ACTIONS(687), 1,
      sym_nbt_path,
  [4244] = 1,
    ACTIONS(689), 1,
      sym_identifier,
  [4248] = 1,
    ACTIONS(691), 1,
      sym_identifier,
  [4252] = 1,
    ACTIONS(693), 1,
      sym_number,
  [4256] = 1,
    ACTIONS(695), 1,
      sym_number,
  [4260] = 1,
    ACTIONS(265), 1,
      sym_nbt_path,
  [4264] = 1,
    ACTIONS(697), 1,
      sym_identifier,
  [4268] = 1,
    ACTIONS(699), 1,
      sym_identifier,
  [4272] = 1,
    ACTIONS(701), 1,
      sym_number,
  [4276] = 1,
    ACTIONS(17), 1,
      sym_nbt_path,
  [4280] = 1,
    ACTIONS(21), 1,
      sym_nbt_path,
  [4284] = 1,
    ACTIONS(25), 1,
      sym_nbt_path,
  [4288] = 1,
    ACTIONS(703), 1,
      sym_identifier,
  [4292] = 1,
    ACTIONS(705), 1,
      sym_nbt_path,
  [4296] = 1,
    ACTIONS(707), 1,
      sym_identifier,
  [4300] = 1,
    ACTIONS(709), 1,
      sym_number,
  [4304] = 1,
    ACTIONS(711), 1,
      sym_nbt_path,
  [4308] = 1,
    ACTIONS(713), 1,
      sym_text,
  [4312] = 1,
    ACTIONS(715), 1,
      sym_number,
  [4316] = 1,
    ACTIONS(717), 1,
      sym_number,
  [4320] = 1,
    ACTIONS(719), 1,
      sym_number,
  [4324] = 1,
    ACTIONS(721), 1,
      sym_number,
  [4328] = 1,
    ACTIONS(723), 1,
      sym_number,
  [4332] = 1,
    ACTIONS(725), 1,
      sym_identifier,
  [4336] = 1,
    ACTIONS(727), 1,
      anon_sym_SEMI,
  [4340] = 1,
    ACTIONS(729), 1,
      sym_identifier,
  [4344] = 1,
    ACTIONS(731), 1,
      anon_sym_EQ,
  [4348] = 1,
    ACTIONS(733), 1,
      sym_identifier,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 62,
  [SMALL_STATE(4)] = 119,
  [SMALL_STATE(5)] = 175,
  [SMALL_STATE(6)] = 231,
  [SMALL_STATE(7)] = 287,
  [SMALL_STATE(8)] = 332,
  [SMALL_STATE(9)] = 377,
  [SMALL_STATE(10)] = 422,
  [SMALL_STATE(11)] = 464,
  [SMALL_STATE(12)] = 504,
  [SMALL_STATE(13)] = 577,
  [SMALL_STATE(14)] = 650,
  [SMALL_STATE(15)] = 689,
  [SMALL_STATE(16)] = 728,
  [SMALL_STATE(17)] = 767,
  [SMALL_STATE(18)] = 840,
  [SMALL_STATE(19)] = 878,
  [SMALL_STATE(20)] = 916,
  [SMALL_STATE(21)] = 983,
  [SMALL_STATE(22)] = 1044,
  [SMALL_STATE(23)] = 1107,
  [SMALL_STATE(24)] = 1168,
  [SMALL_STATE(25)] = 1205,
  [SMALL_STATE(26)] = 1268,
  [SMALL_STATE(27)] = 1326,
  [SMALL_STATE(28)] = 1384,
  [SMALL_STATE(29)] = 1410,
  [SMALL_STATE(30)] = 1434,
  [SMALL_STATE(31)] = 1458,
  [SMALL_STATE(32)] = 1482,
  [SMALL_STATE(33)] = 1506,
  [SMALL_STATE(34)] = 1530,
  [SMALL_STATE(35)] = 1555,
  [SMALL_STATE(36)] = 1580,
  [SMALL_STATE(37)] = 1600,
  [SMALL_STATE(38)] = 1622,
  [SMALL_STATE(39)] = 1642,
  [SMALL_STATE(40)] = 1662,
  [SMALL_STATE(41)] = 1682,
  [SMALL_STATE(42)] = 1702,
  [SMALL_STATE(43)] = 1724,
  [SMALL_STATE(44)] = 1744,
  [SMALL_STATE(45)] = 1764,
  [SMALL_STATE(46)] = 1784,
  [SMALL_STATE(47)] = 1803,
  [SMALL_STATE(48)] = 1822,
  [SMALL_STATE(49)] = 1841,
  [SMALL_STATE(50)] = 1860,
  [SMALL_STATE(51)] = 1879,
  [SMALL_STATE(52)] = 1895,
  [SMALL_STATE(53)] = 1911,
  [SMALL_STATE(54)] = 1927,
  [SMALL_STATE(55)] = 1943,
  [SMALL_STATE(56)] = 1959,
  [SMALL_STATE(57)] = 1975,
  [SMALL_STATE(58)] = 1991,
  [SMALL_STATE(59)] = 2007,
  [SMALL_STATE(60)] = 2023,
  [SMALL_STATE(61)] = 2039,
  [SMALL_STATE(62)] = 2055,
  [SMALL_STATE(63)] = 2071,
  [SMALL_STATE(64)] = 2087,
  [SMALL_STATE(65)] = 2103,
  [SMALL_STATE(66)] = 2119,
  [SMALL_STATE(67)] = 2135,
  [SMALL_STATE(68)] = 2151,
  [SMALL_STATE(69)] = 2167,
  [SMALL_STATE(70)] = 2183,
  [SMALL_STATE(71)] = 2199,
  [SMALL_STATE(72)] = 2215,
  [SMALL_STATE(73)] = 2231,
  [SMALL_STATE(74)] = 2247,
  [SMALL_STATE(75)] = 2263,
  [SMALL_STATE(76)] = 2279,
  [SMALL_STATE(77)] = 2295,
  [SMALL_STATE(78)] = 2311,
  [SMALL_STATE(79)] = 2327,
  [SMALL_STATE(80)] = 2343,
  [SMALL_STATE(81)] = 2359,
  [SMALL_STATE(82)] = 2375,
  [SMALL_STATE(83)] = 2391,
  [SMALL_STATE(84)] = 2407,
  [SMALL_STATE(85)] = 2423,
  [SMALL_STATE(86)] = 2439,
  [SMALL_STATE(87)] = 2455,
  [SMALL_STATE(88)] = 2471,
  [SMALL_STATE(89)] = 2487,
  [SMALL_STATE(90)] = 2503,
  [SMALL_STATE(91)] = 2519,
  [SMALL_STATE(92)] = 2535,
  [SMALL_STATE(93)] = 2551,
  [SMALL_STATE(94)] = 2567,
  [SMALL_STATE(95)] = 2583,
  [SMALL_STATE(96)] = 2599,
  [SMALL_STATE(97)] = 2615,
  [SMALL_STATE(98)] = 2631,
  [SMALL_STATE(99)] = 2647,
  [SMALL_STATE(100)] = 2663,
  [SMALL_STATE(101)] = 2679,
  [SMALL_STATE(102)] = 2695,
  [SMALL_STATE(103)] = 2711,
  [SMALL_STATE(104)] = 2727,
  [SMALL_STATE(105)] = 2757,
  [SMALL_STATE(106)] = 2787,
  [SMALL_STATE(107)] = 2816,
  [SMALL_STATE(108)] = 2845,
  [SMALL_STATE(109)] = 2874,
  [SMALL_STATE(110)] = 2900,
  [SMALL_STATE(111)] = 2926,
  [SMALL_STATE(112)] = 2952,
  [SMALL_STATE(113)] = 2978,
  [SMALL_STATE(114)] = 3004,
  [SMALL_STATE(115)] = 3030,
  [SMALL_STATE(116)] = 3053,
  [SMALL_STATE(117)] = 3076,
  [SMALL_STATE(118)] = 3099,
  [SMALL_STATE(119)] = 3122,
  [SMALL_STATE(120)] = 3145,
  [SMALL_STATE(121)] = 3168,
  [SMALL_STATE(122)] = 3191,
  [SMALL_STATE(123)] = 3214,
  [SMALL_STATE(124)] = 3231,
  [SMALL_STATE(125)] = 3245,
  [SMALL_STATE(126)] = 3259,
  [SMALL_STATE(127)] = 3273,
  [SMALL_STATE(128)] = 3287,
  [SMALL_STATE(129)] = 3301,
  [SMALL_STATE(130)] = 3315,
  [SMALL_STATE(131)] = 3329,
  [SMALL_STATE(132)] = 3343,
  [SMALL_STATE(133)] = 3357,
  [SMALL_STATE(134)] = 3371,
  [SMALL_STATE(135)] = 3385,
  [SMALL_STATE(136)] = 3399,
  [SMALL_STATE(137)] = 3413,
  [SMALL_STATE(138)] = 3427,
  [SMALL_STATE(139)] = 3441,
  [SMALL_STATE(140)] = 3455,
  [SMALL_STATE(141)] = 3469,
  [SMALL_STATE(142)] = 3483,
  [SMALL_STATE(143)] = 3497,
  [SMALL_STATE(144)] = 3511,
  [SMALL_STATE(145)] = 3525,
  [SMALL_STATE(146)] = 3539,
  [SMALL_STATE(147)] = 3556,
  [SMALL_STATE(148)] = 3567,
  [SMALL_STATE(149)] = 3580,
  [SMALL_STATE(150)] = 3599,
  [SMALL_STATE(151)] = 3611,
  [SMALL_STATE(152)] = 3623,
  [SMALL_STATE(153)] = 3635,
  [SMALL_STATE(154)] = 3648,
  [SMALL_STATE(155)] = 3661,
  [SMALL_STATE(156)] = 3674,
  [SMALL_STATE(157)] = 3687,
  [SMALL_STATE(158)] = 3696,
  [SMALL_STATE(159)] = 3709,
  [SMALL_STATE(160)] = 3722,
  [SMALL_STATE(161)] = 3735,
  [SMALL_STATE(162)] = 3748,
  [SMALL_STATE(163)] = 3759,
  [SMALL_STATE(164)] = 3772,
  [SMALL_STATE(165)] = 3785,
  [SMALL_STATE(166)] = 3798,
  [SMALL_STATE(167)] = 3811,
  [SMALL_STATE(168)] = 3824,
  [SMALL_STATE(169)] = 3837,
  [SMALL_STATE(170)] = 3850,
  [SMALL_STATE(171)] = 3863,
  [SMALL_STATE(172)] = 3872,
  [SMALL_STATE(173)] = 3880,
  [SMALL_STATE(174)] = 3888,
  [SMALL_STATE(175)] = 3894,
  [SMALL_STATE(176)] = 3900,
  [SMALL_STATE(177)] = 3910,
  [SMALL_STATE(178)] = 3920,
  [SMALL_STATE(179)] = 3930,
  [SMALL_STATE(180)] = 3938,
  [SMALL_STATE(181)] = 3946,
  [SMALL_STATE(182)] = 3956,
  [SMALL_STATE(183)] = 3964,
  [SMALL_STATE(184)] = 3974,
  [SMALL_STATE(185)] = 3980,
  [SMALL_STATE(186)] = 3986,
  [SMALL_STATE(187)] = 3992,
  [SMALL_STATE(188)] = 3997,
  [SMALL_STATE(189)] = 4002,
  [SMALL_STATE(190)] = 4007,
  [SMALL_STATE(191)] = 4012,
  [SMALL_STATE(192)] = 4019,
  [SMALL_STATE(193)] = 4026,
  [SMALL_STATE(194)] = 4033,
  [SMALL_STATE(195)] = 4040,
  [SMALL_STATE(196)] = 4047,
  [SMALL_STATE(197)] = 4054,
  [SMALL_STATE(198)] = 4059,
  [SMALL_STATE(199)] = 4064,
  [SMALL_STATE(200)] = 4069,
  [SMALL_STATE(201)] = 4076,
  [SMALL_STATE(202)] = 4081,
  [SMALL_STATE(203)] = 4086,
  [SMALL_STATE(204)] = 4093,
  [SMALL_STATE(205)] = 4098,
  [SMALL_STATE(206)] = 4105,
  [SMALL_STATE(207)] = 4112,
  [SMALL_STATE(208)] = 4116,
  [SMALL_STATE(209)] = 4120,
  [SMALL_STATE(210)] = 4124,
  [SMALL_STATE(211)] = 4128,
  [SMALL_STATE(212)] = 4132,
  [SMALL_STATE(213)] = 4136,
  [SMALL_STATE(214)] = 4140,
  [SMALL_STATE(215)] = 4144,
  [SMALL_STATE(216)] = 4148,
  [SMALL_STATE(217)] = 4152,
  [SMALL_STATE(218)] = 4156,
  [SMALL_STATE(219)] = 4160,
  [SMALL_STATE(220)] = 4164,
  [SMALL_STATE(221)] = 4168,
  [SMALL_STATE(222)] = 4172,
  [SMALL_STATE(223)] = 4176,
  [SMALL_STATE(224)] = 4180,
  [SMALL_STATE(225)] = 4184,
  [SMALL_STATE(226)] = 4188,
  [SMALL_STATE(227)] = 4192,
  [SMALL_STATE(228)] = 4196,
  [SMALL_STATE(229)] = 4200,
  [SMALL_STATE(230)] = 4204,
  [SMALL_STATE(231)] = 4208,
  [SMALL_STATE(232)] = 4212,
  [SMALL_STATE(233)] = 4216,
  [SMALL_STATE(234)] = 4220,
  [SMALL_STATE(235)] = 4224,
  [SMALL_STATE(236)] = 4228,
  [SMALL_STATE(237)] = 4232,
  [SMALL_STATE(238)] = 4236,
  [SMALL_STATE(239)] = 4240,
  [SMALL_STATE(240)] = 4244,
  [SMALL_STATE(241)] = 4248,
  [SMALL_STATE(242)] = 4252,
  [SMALL_STATE(243)] = 4256,
  [SMALL_STATE(244)] = 4260,
  [SMALL_STATE(245)] = 4264,
  [SMALL_STATE(246)] = 4268,
  [SMALL_STATE(247)] = 4272,
  [SMALL_STATE(248)] = 4276,
  [SMALL_STATE(249)] = 4280,
  [SMALL_STATE(250)] = 4284,
  [SMALL_STATE(251)] = 4288,
  [SMALL_STATE(252)] = 4292,
  [SMALL_STATE(253)] = 4296,
  [SMALL_STATE(254)] = 4300,
  [SMALL_STATE(255)] = 4304,
  [SMALL_STATE(256)] = 4308,
  [SMALL_STATE(257)] = 4312,
  [SMALL_STATE(258)] = 4316,
  [SMALL_STATE(259)] = 4320,
  [SMALL_STATE(260)] = 4324,
  [SMALL_STATE(261)] = 4328,
  [SMALL_STATE(262)] = 4332,
  [SMALL_STATE(263)] = 4336,
  [SMALL_STATE(264)] = 4340,
  [SMALL_STATE(265)] = 4344,
  [SMALL_STATE(266)] = 4348,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(262),
  [7] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 1, 0, 2),
  [9] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 1, 0, 2),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(159),
  [13] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_type, 1, 0, 0),
  [15] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_type, 1, 0, 0),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 2, 0, 2),
  [19] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 2, 0, 2),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [23] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [25] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [27] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(212),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(168),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(229),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(230),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(202),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [49] = {.entry = {.count = 1, .reusable = false}}, SHIFT(216),
  [51] = {.entry = {.count = 1, .reusable = false}}, SHIFT(236),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(127),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(209),
  [57] = {.entry = {.count = 1, .reusable = false}}, SHIFT(135),
  [59] = {.entry = {.count = 1, .reusable = false}}, SHIFT(136),
  [61] = {.entry = {.count = 1, .reusable = false}}, SHIFT(137),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(123),
  [65] = {.entry = {.count = 1, .reusable = false}}, SHIFT(140),
  [67] = {.entry = {.count = 1, .reusable = false}}, SHIFT(206),
  [69] = {.entry = {.count = 1, .reusable = false}}, SHIFT(142),
  [71] = {.entry = {.count = 1, .reusable = false}}, SHIFT(144),
  [73] = {.entry = {.count = 1, .reusable = false}}, SHIFT(145),
  [75] = {.entry = {.count = 1, .reusable = false}}, SHIFT(124),
  [77] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(22),
  [80] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [82] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(26),
  [85] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(216),
  [88] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(236),
  [91] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(127),
  [94] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(209),
  [97] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(135),
  [100] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(136),
  [103] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(137),
  [106] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(123),
  [109] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(140),
  [112] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(206),
  [115] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(142),
  [118] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(144),
  [121] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(145),
  [124] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(124),
  [127] = {.entry = {.count = 1, .reusable = false}}, SHIFT(204),
  [129] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [131] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [133] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0),
  [135] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(130),
  [138] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(138),
  [141] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(237),
  [144] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(201),
  [147] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(106),
  [150] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(151),
  [153] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(107),
  [156] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(108),
  [159] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(104),
  [162] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(105),
  [165] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(197),
  [168] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_execute_command_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [171] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [173] = {.entry = {.count = 1, .reusable = true}}, SHIFT(216),
  [175] = {.entry = {.count = 1, .reusable = true}}, SHIFT(236),
  [177] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [179] = {.entry = {.count = 1, .reusable = true}}, SHIFT(209),
  [181] = {.entry = {.count = 1, .reusable = true}}, SHIFT(135),
  [183] = {.entry = {.count = 1, .reusable = true}}, SHIFT(136),
  [185] = {.entry = {.count = 1, .reusable = true}}, SHIFT(137),
  [187] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(140),
  [191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(206),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(144),
  [195] = {.entry = {.count = 1, .reusable = true}}, SHIFT(145),
  [197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [199] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_command, 2, 0, 0),
  [201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [203] = {.entry = {.count = 1, .reusable = true}}, SHIFT(138),
  [205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(237),
  [207] = {.entry = {.count = 1, .reusable = true}}, SHIFT(201),
  [209] = {.entry = {.count = 1, .reusable = true}}, SHIFT(106),
  [211] = {.entry = {.count = 1, .reusable = true}}, SHIFT(151),
  [213] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [215] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [217] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [219] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [221] = {.entry = {.count = 1, .reusable = true}}, SHIFT(197),
  [223] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [227] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_store, 3, 0, 19),
  [229] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_execute_store, 3, 0, 19),
  [231] = {.entry = {.count = 1, .reusable = true}}, SHIFT(211),
  [233] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_store_target, 3, 0, 31),
  [235] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_store_target, 3, 0, 31),
  [237] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_store_target, 3, 0, 33),
  [239] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_store_target, 3, 0, 33),
  [241] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_store_target, 3, 0, 34),
  [243] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_store_target, 3, 0, 34),
  [245] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_store_target, 3, 0, 32),
  [247] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_store_target, 3, 0, 32),
  [249] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_store_target, 3, 0, 18),
  [251] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_store_target, 3, 0, 18),
  [253] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_local_coordinate_plain, 1, 0, 0),
  [255] = {.entry = {.count = 1, .reusable = true}}, SHIFT(242),
  [257] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_relative_coordinate_plain, 1, 0, 0),
  [261] = {.entry = {.count = 1, .reusable = true}}, SHIFT(258),
  [263] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [265] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_position, 3, 0, 23),
  [267] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [269] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_relative_coordinate, 1, 0, 0),
  [273] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_relative_coordinate_offset, 2, 0, 0),
  [275] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_local_coordinate_offset, 2, 0, 0),
  [277] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_local_coordinate, 1, 0, 0),
  [279] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(37),
  [282] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_relative_coordinate_offset, 3, 0, 0),
  [284] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_local_coordinate_offset, 3, 0, 0),
  [286] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_facing_entity, 3, 0, 16),
  [288] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [290] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_block, 4, 0, 26),
  [292] = {.entry = {.count = 1, .reusable = true}}, SHIFT(158),
  [294] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_block, 3, 0, 24),
  [296] = {.entry = {.count = 1, .reusable = true}}, SHIFT(155),
  [298] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_blocks, 4, 0, 27),
  [300] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [302] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 1, 0, 0),
  [304] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [306] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 2, 0, 0),
  [308] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [310] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_block, 7, 0, 44),
  [312] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_unit, 1, 0, 0),
  [314] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 14),
  [316] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 15),
  [318] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_survival_command, 2, 0, 4),
  [320] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_adventure_command, 2, 0, 4),
  [322] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_spectator_command, 2, 0, 4),
  [324] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_creative_command, 2, 0, 4),
  [326] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_positioned_as, 3, 0, 16),
  [328] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_rotated_as, 3, 0, 16),
  [330] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rotation, 2, 0, 17),
  [332] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_entity, 2, 0, 4),
  [334] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_predicate, 2, 0, 18),
  [336] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command, 2, 0, 0),
  [338] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_add_command, 4, 0, 20),
  [340] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_set_command, 4, 0, 20),
  [342] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_subcommand, 1, 0, 0),
  [344] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_as, 2, 0, 4),
  [346] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enchant_command, 4, 0, 21),
  [348] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_facing_entity, 4, 0, 22),
  [350] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_at, 2, 0, 4),
  [352] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_align, 2, 0, 5),
  [354] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_anchored, 2, 0, 6),
  [356] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 5, 0, 25),
  [358] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_say_command, 2, 0, 3),
  [360] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_data, 4, 0, 28),
  [362] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_data, 4, 0, 29),
  [364] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_data, 4, 0, 30),
  [366] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_facing, 2, 0, 7),
  [368] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_dimension, 1, 0, 0),
  [370] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_in, 2, 0, 8),
  [372] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_positioned, 2, 0, 7),
  [374] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_rotated, 2, 0, 9),
  [376] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_store, 5, 0, 35),
  [378] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 6, 0, 37),
  [380] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 6, 0, 38),
  [382] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_block, 5, 0, 39),
  [384] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_blocks, 5, 0, 27),
  [386] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_score, 5, 0, 40),
  [388] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 7, 0, 41),
  [390] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_block, 6, 0, 42),
  [392] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_condition_score, 6, 0, 43),
  [394] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_if, 2, 0, 10),
  [396] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_condition, 1, 0, 0),
  [398] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 3, 0, 0),
  [400] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_unless, 2, 0, 10),
  [402] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_execute_run, 2, 0, 11),
  [404] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inv_clear_command, 2, 0, 4),
  [406] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_type, 1, 0, 0),
  [408] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_query_command, 3, 0, 12),
  [410] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_clear_command, 2, 0, 4),
  [412] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tellraw_command, 3, 0, 13),
  [414] = {.entry = {.count = 1, .reusable = true}}, SHIFT(131),
  [416] = {.entry = {.count = 1, .reusable = false}}, SHIFT(109),
  [418] = {.entry = {.count = 1, .reusable = true}}, SHIFT(112),
  [420] = {.entry = {.count = 1, .reusable = true}}, SHIFT(183),
  [422] = {.entry = {.count = 1, .reusable = true}}, SHIFT(253),
  [424] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [426] = {.entry = {.count = 1, .reusable = true}}, SHIFT(143),
  [428] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [430] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [432] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [434] = {.entry = {.count = 1, .reusable = true}}, SHIFT(128),
  [436] = {.entry = {.count = 1, .reusable = true}}, SHIFT(129),
  [438] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [440] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [442] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [444] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
  [446] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [448] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [450] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [452] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [454] = {.entry = {.count = 1, .reusable = true}}, SHIFT(257),
  [456] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [458] = {.entry = {.count = 1, .reusable = true}}, SHIFT(164),
  [460] = {.entry = {.count = 1, .reusable = true}}, SHIFT(165),
  [462] = {.entry = {.count = 1, .reusable = true}}, SHIFT(205),
  [464] = {.entry = {.count = 1, .reusable = true}}, SHIFT(177),
  [466] = {.entry = {.count = 1, .reusable = true}}, SHIFT(178),
  [468] = {.entry = {.count = 1, .reusable = true}}, SHIFT(244),
  [470] = {.entry = {.count = 1, .reusable = true}}, SHIFT(120),
  [472] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [474] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [476] = {.entry = {.count = 1, .reusable = true}}, SHIFT(198),
  [478] = {.entry = {.count = 1, .reusable = true}}, SHIFT(199),
  [480] = {.entry = {.count = 1, .reusable = true}}, SHIFT(215),
  [482] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [484] = {.entry = {.count = 1, .reusable = true}}, SHIFT(186),
  [486] = {.entry = {.count = 1, .reusable = true}}, SHIFT(157),
  [488] = {.entry = {.count = 1, .reusable = true}}, SHIFT(119),
  [490] = {.entry = {.count = 1, .reusable = false}}, SHIFT(119),
  [492] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [494] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [496] = {.entry = {.count = 1, .reusable = true}}, SHIFT(141),
  [498] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [500] = {.entry = {.count = 1, .reusable = true}}, SHIFT(213),
  [502] = {.entry = {.count = 1, .reusable = true}}, SHIFT(134),
  [504] = {.entry = {.count = 1, .reusable = true}}, SHIFT(266),
  [506] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [508] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [510] = {.entry = {.count = 1, .reusable = false}}, SHIFT(80),
  [512] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [514] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(262),
  [517] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [519] = {.entry = {.count = 1, .reusable = true}}, SHIFT(238),
  [521] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0),
  [523] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0), SHIFT_REPEAT(265),
  [526] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [528] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [530] = {.entry = {.count = 1, .reusable = true}}, SHIFT(265),
  [532] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_argument, 3, 0, 36),
  [534] = {.entry = {.count = 1, .reusable = true}}, SHIFT(171),
  [536] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [538] = {.entry = {.count = 1, .reusable = true}}, SHIFT(250),
  [540] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [542] = {.entry = {.count = 1, .reusable = true}}, SHIFT(175),
  [544] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_condition_block_repeat1, 2, 0, 0),
  [546] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_condition_block_repeat1, 2, 0, 0), SHIFT_REPEAT(238),
  [549] = {.entry = {.count = 1, .reusable = true}}, SHIFT(254),
  [551] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_relative_coordinate_plain, 1, 0, 0),
  [553] = {.entry = {.count = 1, .reusable = true}}, SHIFT(193),
  [555] = {.entry = {.count = 1, .reusable = true}}, SHIFT(207),
  [557] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_local_coordinate_plain, 1, 0, 0),
  [559] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [561] = {.entry = {.count = 1, .reusable = true}}, SHIFT(223),
  [563] = {.entry = {.count = 1, .reusable = true}}, SHIFT(227),
  [565] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [567] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [569] = {.entry = {.count = 1, .reusable = true}}, SHIFT(249),
  [571] = {.entry = {.count = 1, .reusable = true}}, SHIFT(174),
  [573] = {.entry = {.count = 1, .reusable = true}}, SHIFT(190),
  [575] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 1, 0, 0),
  [577] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [579] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block_state, 3, 0, 36),
  [581] = {.entry = {.count = 1, .reusable = false}}, SHIFT(166),
  [583] = {.entry = {.count = 1, .reusable = true}}, SHIFT(260),
  [585] = {.entry = {.count = 1, .reusable = false}}, SHIFT(220),
  [587] = {.entry = {.count = 1, .reusable = true}}, SHIFT(261),
  [589] = {.entry = {.count = 1, .reusable = false}}, SHIFT(221),
  [591] = {.entry = {.count = 1, .reusable = true}}, SHIFT(170),
  [593] = {.entry = {.count = 1, .reusable = true}}, SHIFT(187),
  [595] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_condition_block_repeat1, 1, 0, 0),
  [597] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [599] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [601] = {.entry = {.count = 1, .reusable = true}}, SHIFT(264),
  [603] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted_string, 1, 0, 0),
  [605] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [607] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 3, 0, 1),
  [609] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_relative_coordinate, 1, 0, 0),
  [611] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_local_coordinate, 1, 0, 0),
  [613] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_relative_coordinate_offset, 2, 0, 0),
  [615] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_local_coordinate_offset, 2, 0, 0),
  [617] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_relative_coordinate_offset, 3, 0, 0),
  [619] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_local_coordinate_offset, 3, 0, 0),
  [621] = {.entry = {.count = 1, .reusable = true}}, SHIFT(149),
  [623] = {.entry = {.count = 1, .reusable = true}}, SHIFT(251),
  [625] = {.entry = {.count = 1, .reusable = false}}, SHIFT(47),
  [627] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [629] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [631] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [633] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [635] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_position, 3, 0, 23),
  [637] = {.entry = {.count = 1, .reusable = true}}, SHIFT(148),
  [639] = {.entry = {.count = 1, .reusable = true}}, SHIFT(241),
  [641] = {.entry = {.count = 1, .reusable = true}}, SHIFT(196),
  [643] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [645] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [647] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_effect, 1, 0, 0),
  [649] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [651] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_effect, 1, 0, 0),
  [653] = {.entry = {.count = 1, .reusable = true}}, SHIFT(239),
  [655] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [657] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [659] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [661] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [663] = {.entry = {.count = 1, .reusable = true}}, SHIFT(232),
  [665] = {.entry = {.count = 1, .reusable = true}}, SHIFT(259),
  [667] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_enchant, 1, 0, 0),
  [669] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_enchant, 1, 0, 0),
  [671] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [673] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [675] = {.entry = {.count = 1, .reusable = true}}, SHIFT(85),
  [677] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [679] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [681] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [683] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [685] = {.entry = {.count = 1, .reusable = true}}, SHIFT(162),
  [687] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [689] = {.entry = {.count = 1, .reusable = true}}, SHIFT(147),
  [691] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [693] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [695] = {.entry = {.count = 1, .reusable = true}}, SHIFT(233),
  [697] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [699] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [701] = {.entry = {.count = 1, .reusable = true}}, SHIFT(234),
  [703] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [705] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [707] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [709] = {.entry = {.count = 1, .reusable = true}}, SHIFT(195),
  [711] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [713] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [715] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [717] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [719] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [721] = {.entry = {.count = 1, .reusable = true}}, SHIFT(224),
  [723] = {.entry = {.count = 1, .reusable = true}}, SHIFT(225),
  [725] = {.entry = {.count = 1, .reusable = true}}, SHIFT(203),
  [727] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [729] = {.entry = {.count = 1, .reusable = true}}, SHIFT(252),
  [731] = {.entry = {.count = 1, .reusable = true}}, SHIFT(146),
  [733] = {.entry = {.count = 1, .reusable = true}}, SHIFT(188),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_sand(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
