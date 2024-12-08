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
#define STATE_COUNT 92
#define LARGE_STATE_COUNT 7
#define SYMBOL_COUNT 140
#define ALIAS_COUNT 0
#define TOKEN_COUNT 106
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 15
#define MAX_ALIAS_SEQUENCE_LENGTH 5
#define PRODUCTION_ID_COUNT 13

enum ts_symbol_identifiers {
  anon_sym_fn = 1,
  anon_sym_LBRACE = 2,
  anon_sym_SEMI = 3,
  aux_sym_block_token1 = 4,
  anon_sym_RBRACE = 5,
  anon_sym_ATs = 6,
  anon_sym_ATa = 7,
  anon_sym_ATp = 8,
  anon_sym_ATr = 9,
  anon_sym_ATe = 10,
  anon_sym_LBRACK = 11,
  anon_sym_COMMA = 12,
  anon_sym_RBRACK = 13,
  anon_sym_EQ = 14,
  anon_sym_levels = 15,
  anon_sym_points = 16,
  anon_sym_DOT_DOT = 17,
  anon_sym_xpadd = 18,
  anon_sym_xpset = 19,
  anon_sym_xpquery = 20,
  aux_sym_quoted_string_token1 = 21,
  anon_sym_say = 22,
  anon_sym_clear = 23,
  anon_sym_eclear = 24,
  anon_sym_tellraw = 25,
  anon_sym_effect = 26,
  anon_sym_enchant = 27,
  anon_sym_minecraft_COLONspeed = 28,
  anon_sym_minecraft_COLONslowness = 29,
  anon_sym_minecraft_COLONhaste = 30,
  anon_sym_minecraft_COLONmining_fatigue = 31,
  anon_sym_minecraft_COLONstrength = 32,
  anon_sym_minecraft_COLONinstant_health = 33,
  anon_sym_minecraft_COLONinstant_damage = 34,
  anon_sym_minecraft_COLONjump_boost = 35,
  anon_sym_minecraft_COLONnausea = 36,
  anon_sym_minecraft_COLONregeneration = 37,
  anon_sym_minecraft_COLONresistance = 38,
  anon_sym_minecraft_COLONfire_resistance = 39,
  anon_sym_minecraft_COLONwater_breathing = 40,
  anon_sym_minecraft_COLONinvisibility = 41,
  anon_sym_minecraft_COLONblindness = 42,
  anon_sym_minecraft_COLONnight_vision = 43,
  anon_sym_minecraft_COLONhunger = 44,
  anon_sym_minecraft_COLONweakness = 45,
  anon_sym_minecraft_COLONpoison = 46,
  anon_sym_minecraft_COLONwither = 47,
  anon_sym_minecraft_COLONhealth_boost = 48,
  anon_sym_minecraft_COLONabsorption = 49,
  anon_sym_minecraft_COLONsaturation = 50,
  anon_sym_minecraft_COLONglowing = 51,
  anon_sym_minecraft_COLONlevitation = 52,
  anon_sym_minecraft_COLONluck = 53,
  anon_sym_minecraft_COLONunluck = 54,
  anon_sym_minecraft_COLONslow_falling = 55,
  anon_sym_minecraft_COLONconduit_power = 56,
  anon_sym_minecraft_COLONdolphins_grace = 57,
  anon_sym_minecraft_COLONbad_omen = 58,
  anon_sym_minecraft_COLONhero_of_the_village = 59,
  anon_sym_minecraft_COLONprotection = 60,
  anon_sym_minecraft_COLONfire_protection = 61,
  anon_sym_minecraft_COLONfeather_falling = 62,
  anon_sym_minecraft_COLONblast_protection = 63,
  anon_sym_minecraft_COLONprojectile_protection = 64,
  anon_sym_minecraft_COLONrespiration = 65,
  anon_sym_minecraft_COLONaqua_affinity = 66,
  anon_sym_minecraft_COLONthorns = 67,
  anon_sym_minecraft_COLONdepth_strider = 68,
  anon_sym_minecraft_COLONfrost_walker = 69,
  anon_sym_minecraft_COLONbinding_curse = 70,
  anon_sym_minecraft_COLONsharpness = 71,
  anon_sym_minecraft_COLONsmite = 72,
  anon_sym_minecraft_COLONbane_of_arthropods = 73,
  anon_sym_minecraft_COLONknockback = 74,
  anon_sym_minecraft_COLONfire_aspect = 75,
  anon_sym_minecraft_COLONlooting = 76,
  anon_sym_minecraft_COLONsweeping = 77,
  anon_sym_minecraft_COLONefficiency = 78,
  anon_sym_minecraft_COLONsilk_touch = 79,
  anon_sym_minecraft_COLONunbreaking = 80,
  anon_sym_minecraft_COLONfortune = 81,
  anon_sym_minecraft_COLONpower = 82,
  anon_sym_minecraft_COLONpunch = 83,
  anon_sym_minecraft_COLONflame = 84,
  anon_sym_minecraft_COLONinfinity = 85,
  anon_sym_minecraft_COLONluck_of_the_sea = 86,
  anon_sym_minecraft_COLONlure = 87,
  anon_sym_minecraft_COLONmending = 88,
  anon_sym_minecraft_COLONvanishing_curse = 89,
  anon_sym_minecraft_COLONsoul_speed = 90,
  anon_sym_minecraft_COLONswift_sneak = 91,
  sym_text = 92,
  anon_sym_time = 93,
  anon_sym_set = 94,
  anon_sym_query = 95,
  anon_sym_gms = 96,
  anon_sym_gma = 97,
  anon_sym_gmsp = 98,
  anon_sym_gmc = 99,
  anon_sym_day = 100,
  anon_sym_night = 101,
  anon_sym_noon = 102,
  anon_sym_midnight = 103,
  sym_identifier = 104,
  sym_number = 105,
  sym_source_file = 106,
  sym__definition = 107,
  sym_function_definition = 108,
  sym_block = 109,
  sym__command = 110,
  sym_target_selector = 111,
  sym_selector_type = 112,
  sym_selector_arguments = 113,
  sym_selector_argument = 114,
  sym_xp_type = 115,
  sym_range_value = 116,
  sym_xp_add_command = 117,
  sym_xp_set_command = 118,
  sym_xp_query_command = 119,
  sym_quoted_string = 120,
  sym_say_command = 121,
  sym_inv_clear_command = 122,
  sym_effect_clear_command = 123,
  sym_tellraw_command = 124,
  sym_effect_command = 125,
  sym_enchant_command = 126,
  sym_vanilla_effect = 127,
  sym_vanilla_enchant = 128,
  sym_custom_effect = 129,
  sym_custom_enchant = 130,
  sym_time_command = 131,
  sym_gm_survival_command = 132,
  sym_gm_adventure_command = 133,
  sym_gm_spectator_command = 134,
  sym_gm_creative_command = 135,
  sym_time_unit = 136,
  aux_sym_source_file_repeat1 = 137,
  aux_sym_block_repeat1 = 138,
  aux_sym_selector_arguments_repeat1 = 139,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [anon_sym_fn] = "fn",
  [anon_sym_LBRACE] = "{",
  [anon_sym_SEMI] = ";",
  [aux_sym_block_token1] = "block_token1",
  [anon_sym_RBRACE] = "}",
  [anon_sym_ATs] = "@s",
  [anon_sym_ATa] = "@a",
  [anon_sym_ATp] = "@p",
  [anon_sym_ATr] = "@r",
  [anon_sym_ATe] = "@e",
  [anon_sym_LBRACK] = "[",
  [anon_sym_COMMA] = ",",
  [anon_sym_RBRACK] = "]",
  [anon_sym_EQ] = "=",
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
  [aux_sym_selector_arguments_repeat1] = "selector_arguments_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [anon_sym_fn] = anon_sym_fn,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [aux_sym_block_token1] = aux_sym_block_token1,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_ATs] = anon_sym_ATs,
  [anon_sym_ATa] = anon_sym_ATa,
  [anon_sym_ATp] = anon_sym_ATp,
  [anon_sym_ATr] = anon_sym_ATr,
  [anon_sym_ATe] = anon_sym_ATe,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_EQ] = anon_sym_EQ,
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
  [anon_sym_EQ] = {
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
  [aux_sym_selector_arguments_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_amount = 1,
  field_amplifier = 2,
  field_body = 3,
  field_duration = 4,
  field_effect_type = 5,
  field_enchantment = 6,
  field_etype = 7,
  field_key = 8,
  field_level = 9,
  field_message = 10,
  field_name = 11,
  field_query_type = 12,
  field_selector = 13,
  field_target = 14,
  field_value = 15,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_amount] = "amount",
  [field_amplifier] = "amplifier",
  [field_body] = "body",
  [field_duration] = "duration",
  [field_effect_type] = "effect_type",
  [field_enchantment] = "enchantment",
  [field_etype] = "etype",
  [field_key] = "key",
  [field_level] = "level",
  [field_message] = "message",
  [field_name] = "name",
  [field_query_type] = "query_type",
  [field_selector] = "selector",
  [field_target] = "target",
  [field_value] = "value",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 2},
  [2] = {.index = 2, .length = 1},
  [3] = {.index = 3, .length = 1},
  [4] = {.index = 4, .length = 1},
  [5] = {.index = 5, .length = 2},
  [6] = {.index = 7, .length = 2},
  [7] = {.index = 9, .length = 1},
  [8] = {.index = 10, .length = 1},
  [9] = {.index = 11, .length = 3},
  [10] = {.index = 14, .length = 3},
  [11] = {.index = 17, .length = 4},
  [12] = {.index = 21, .length = 2},
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
    {field_etype, 2},
    {field_target, 1},
  [7] =
    {field_message, 2},
    {field_target, 1},
  [9] =
    {field_value, 2},
  [10] =
    {field_query_type, 2},
  [11] =
    {field_amount, 1},
    {field_etype, 3},
    {field_target, 2},
  [14] =
    {field_enchantment, 2},
    {field_level, 3},
    {field_target, 1},
  [17] =
    {field_amplifier, 4},
    {field_duration, 3},
    {field_effect_type, 2},
    {field_target, 1},
  [21] =
    {field_key, 0},
    {field_value, 2},
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
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 31,
  [37] = 35,
  [38] = 38,
  [39] = 2,
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
  [53] = 3,
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
  [86] = 5,
  [87] = 4,
  [88] = 6,
  [89] = 89,
  [90] = 90,
  [91] = 91,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(669);
      ADVANCE_MAP(
        '"', 4,
        ',', 682,
        '.', 5,
        ';', 672,
        '=', 684,
        '@', 41,
        '[', 680,
        ']', 683,
        'c', 359,
        'd', 42,
        'e', 130,
        'f', 385,
        'g', 379,
        'l', 151,
        'm', 290,
        'n', 291,
        'p', 448,
        'q', 638,
        's', 51,
        't', 168,
        'x', 489,
        '{', 671,
        '}', 674,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(778);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == ',') ADVANCE(682);
      if (lookahead == '.') ADVANCE(5);
      if (lookahead == ']') ADVANCE(683);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(778);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(777);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == 'm') ADVANCE(319);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == 'm') ADVANCE(343);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(3);
      END_STATE();
    case 4:
      if (lookahead == '"') ADVANCE(691);
      if (lookahead != 0) ADVANCE(4);
      END_STATE();
    case 5:
      if (lookahead == '.') ADVANCE(687);
      END_STATE();
    case 6:
      if (lookahead == ':') ADVANCE(46);
      END_STATE();
    case 7:
      if (lookahead == ':') ADVANCE(49);
      END_STATE();
    case 8:
      if (lookahead == ':') ADVANCE(62);
      END_STATE();
    case 9:
      if (lookahead == '_') ADVANCE(73);
      END_STATE();
    case 10:
      if (lookahead == '_') ADVANCE(104);
      END_STATE();
    case 11:
      if (lookahead == '_') ADVANCE(145);
      END_STATE();
    case 12:
      if (lookahead == '_') ADVANCE(72);
      END_STATE();
    case 13:
      if (lookahead == '_') ADVANCE(658);
      END_STATE();
    case 14:
      if (lookahead == '_') ADVANCE(107);
      END_STATE();
    case 15:
      if (lookahead == '_') ADVANCE(238);
      if (lookahead == 'n') ADVANCE(189);
      END_STATE();
    case 16:
      if (lookahead == '_') ADVANCE(120);
      END_STATE();
    case 17:
      if (lookahead == '_') ADVANCE(501);
      END_STATE();
    case 18:
      if (lookahead == '_') ADVANCE(268);
      END_STATE();
    case 19:
      if (lookahead == '_') ADVANCE(551);
      END_STATE();
    case 20:
      if (lookahead == '_') ADVANCE(494);
      END_STATE();
    case 21:
      if (lookahead == '_') ADVANCE(94);
      END_STATE();
    case 22:
      if (lookahead == '_') ADVANCE(480);
      END_STATE();
    case 23:
      if (lookahead == '_') ADVANCE(456);
      END_STATE();
    case 24:
      if (lookahead == '_') ADVANCE(576);
      END_STATE();
    case 25:
      if (lookahead == '_') ADVANCE(457);
      END_STATE();
    case 26:
      if (lookahead == '_') ADVANCE(611);
      END_STATE();
    case 27:
      if (lookahead == '_') ADVANCE(654);
      END_STATE();
    case 28:
      if (lookahead == '_') ADVANCE(244);
      END_STATE();
    case 29:
      if (lookahead == '_') ADVANCE(652);
      END_STATE();
    case 30:
      if (lookahead == '_') ADVANCE(570);
      END_STATE();
    case 31:
      if (lookahead == '_') ADVANCE(621);
      END_STATE();
    case 32:
      if (lookahead == '_') ADVANCE(84);
      END_STATE();
    case 33:
      if (lookahead == '_') ADVANCE(622);
      END_STATE();
    case 34:
      if (lookahead == '_') ADVANCE(566);
      END_STATE();
    case 35:
      if (lookahead == '_') ADVANCE(526);
      END_STATE();
    case 36:
      if (lookahead == '_') ADVANCE(486);
      END_STATE();
    case 37:
      if (lookahead == '_') ADVANCE(108);
      END_STATE();
    case 38:
      if (lookahead == '_') ADVANCE(247);
      END_STATE();
    case 39:
      if (lookahead == '_') ADVANCE(133);
      END_STATE();
    case 40:
      if (lookahead == '_') ADVANCE(502);
      END_STATE();
    case 41:
      if (lookahead == 'a') ADVANCE(676);
      if (lookahead == 'e') ADVANCE(679);
      if (lookahead == 'p') ADVANCE(677);
      if (lookahead == 'r') ADVANCE(678);
      if (lookahead == 's') ADVANCE(675);
      END_STATE();
    case 42:
      if (lookahead == 'a') ADVANCE(661);
      END_STATE();
    case 43:
      if (lookahead == 'a') ADVANCE(770);
      if (lookahead == 'c') ADVANCE(772);
      if (lookahead == 's') ADVANCE(769);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(143);
      if (lookahead == 'q') ADVANCE(644);
      if (lookahead == 's') ADVANCE(179);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(655);
      END_STATE();
    case 46:
      ADVANCE_MAP(
        'a', 101,
        'b', 53,
        'c', 461,
        'd', 171,
        'e', 232,
        'f', 182,
        'g', 364,
        'h', 60,
        'i', 387,
        'j', 637,
        'k', 409,
        'l', 173,
        'm', 223,
        'n', 58,
        'p', 446,
        'r', 154,
        's', 66,
        't', 278,
        'u', 388,
        'v', 89,
        'w', 70,
      );
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(706);
      END_STATE();
    case 48:
      if (lookahead == 'a') ADVANCE(757);
      END_STATE();
    case 49:
      ADVANCE_MAP(
        'a', 503,
        'b', 77,
        'd', 170,
        'e', 232,
        'f', 183,
        'i', 444,
        'k', 409,
        'l', 455,
        'm', 222,
        'p', 477,
        'r', 203,
        's', 285,
        't', 278,
        'u', 441,
        'v', 89,
      );
      END_STATE();
    case 50:
      if (lookahead == 'a') ADVANCE(662);
      END_STATE();
    case 51:
      if (lookahead == 'a') ADVANCE(662);
      if (lookahead == 'e') ADVANCE(579);
      END_STATE();
    case 52:
      if (lookahead == 'a') ADVANCE(236);
      END_STATE();
    case 53:
      if (lookahead == 'a') ADVANCE(139);
      if (lookahead == 'i') ADVANCE(406);
      if (lookahead == 'l') ADVANCE(82);
      END_STATE();
    case 54:
      if (lookahead == 'a') ADVANCE(504);
      END_STATE();
    case 55:
      if (lookahead == 'a') ADVANCE(21);
      END_STATE();
    case 56:
      if (lookahead == 'a') ADVANCE(355);
      END_STATE();
    case 57:
      if (lookahead == 'a') ADVANCE(383);
      END_STATE();
    case 58:
      if (lookahead == 'a') ADVANCE(639);
      if (lookahead == 'i') ADVANCE(267);
      END_STATE();
    case 59:
      if (lookahead == 'a') ADVANCE(505);
      END_STATE();
    case 60:
      if (lookahead == 'a') ADVANCE(572);
      if (lookahead == 'e') ADVANCE(65);
      if (lookahead == 'u') ADVANCE(402);
      END_STATE();
    case 61:
      if (lookahead == 'a') ADVANCE(413);
      END_STATE();
    case 62:
      ADVANCE_MAP(
        'a', 100,
        'b', 71,
        'c', 461,
        'd', 454,
        'f', 333,
        'g', 364,
        'h', 60,
        'i', 401,
        'j', 637,
        'l', 174,
        'm', 322,
        'n', 58,
        'p', 464,
        'r', 175,
        's', 67,
        'u', 423,
        'w', 70,
      );
      END_STATE();
    case 63:
      if (lookahead == 'a') ADVANCE(356);
      END_STATE();
    case 64:
      if (lookahead == 'a') ADVANCE(516);
      END_STATE();
    case 65:
      if (lookahead == 'a') ADVANCE(370);
      if (lookahead == 'r') ADVANCE(479);
      END_STATE();
    case 66:
      ADVANCE_MAP(
        'a', 594,
        'h', 64,
        'i', 360,
        'l', 451,
        'm', 326,
        'o', 640,
        'p', 194,
        't', 522,
        'w', 197,
      );
      END_STATE();
    case 67:
      if (lookahead == 'a') ADVANCE(594);
      if (lookahead == 'l') ADVANCE(451);
      if (lookahead == 'p') ADVANCE(194);
      if (lookahead == 't') ADVANCE(522);
      END_STATE();
    case 68:
      if (lookahead == 'a') ADVANCE(350);
      END_STATE();
    case 69:
      if (lookahead == 'a') ADVANCE(367);
      END_STATE();
    case 70:
      if (lookahead == 'a') ADVANCE(613);
      if (lookahead == 'e') ADVANCE(56);
      if (lookahead == 'i') ADVANCE(593);
      END_STATE();
    case 71:
      if (lookahead == 'a') ADVANCE(138);
      if (lookahead == 'l') ADVANCE(302);
      END_STATE();
    case 72:
      if (lookahead == 'a') ADVANCE(552);
      if (lookahead == 'p') ADVANCE(534);
      END_STATE();
    case 73:
      if (lookahead == 'a') ADVANCE(552);
      if (lookahead == 'p') ADVANCE(534);
      if (lookahead == 'r') ADVANCE(213);
      END_STATE();
    case 74:
      if (lookahead == 'a') ADVANCE(416);
      END_STATE();
    case 75:
      if (lookahead == 'a') ADVANCE(610);
      END_STATE();
    case 76:
      if (lookahead == 'a') ADVANCE(626);
      END_STATE();
    case 77:
      if (lookahead == 'a') ADVANCE(420);
      if (lookahead == 'i') ADVANCE(406);
      if (lookahead == 'l') ADVANCE(81);
      END_STATE();
    case 78:
      if (lookahead == 'a') ADVANCE(368);
      END_STATE();
    case 79:
      if (lookahead == 'a') ADVANCE(382);
      END_STATE();
    case 80:
      if (lookahead == 'a') ADVANCE(264);
      END_STATE();
    case 81:
      if (lookahead == 'a') ADVANCE(559);
      END_STATE();
    case 82:
      if (lookahead == 'a') ADVANCE(559);
      if (lookahead == 'i') ADVANCE(411);
      END_STATE();
    case 83:
      if (lookahead == 'a') ADVANCE(426);
      END_STATE();
    case 84:
      if (lookahead == 'a') ADVANCE(527);
      END_STATE();
    case 85:
      if (lookahead == 'a') ADVANCE(372);
      END_STATE();
    case 86:
      if (lookahead == 'a') ADVANCE(126);
      END_STATE();
    case 87:
      if (lookahead == 'a') ADVANCE(617);
      END_STATE();
    case 88:
      if (lookahead == 'a') ADVANCE(435);
      END_STATE();
    case 89:
      if (lookahead == 'a') ADVANCE(433);
      END_STATE();
    case 90:
      if (lookahead == 'a') ADVANCE(240);
      END_STATE();
    case 91:
      if (lookahead == 'a') ADVANCE(265);
      END_STATE();
    case 92:
      if (lookahead == 'a') ADVANCE(116);
      END_STATE();
    case 93:
      if (lookahead == 'a') ADVANCE(241);
      END_STATE();
    case 94:
      if (lookahead == 'a') ADVANCE(242);
      END_STATE();
    case 95:
      if (lookahead == 'a') ADVANCE(625);
      END_STATE();
    case 96:
      if (lookahead == 'a') ADVANCE(377);
      END_STATE();
    case 97:
      if (lookahead == 'a') ADVANCE(628);
      END_STATE();
    case 98:
      if (lookahead == 'a') ADVANCE(629);
      END_STATE();
    case 99:
      if (lookahead == 'a') ADVANCE(630);
      END_STATE();
    case 100:
      if (lookahead == 'b') ADVANCE(554);
      END_STATE();
    case 101:
      if (lookahead == 'b') ADVANCE(554);
      if (lookahead == 'q') ADVANCE(641);
      END_STATE();
    case 102:
      if (lookahead == 'b') ADVANCE(300);
      END_STATE();
    case 103:
      if (lookahead == 'b') ADVANCE(92);
      END_STATE();
    case 104:
      if (lookahead == 'b') ADVANCE(484);
      END_STATE();
    case 105:
      if (lookahead == 'b') ADVANCE(525);
      END_STATE();
    case 106:
      if (lookahead == 'b') ADVANCE(525);
      if (lookahead == 'l') ADVANCE(645);
      END_STATE();
    case 107:
      if (lookahead == 'b') ADVANCE(528);
      END_STATE();
    case 108:
      if (lookahead == 'b') ADVANCE(485);
      END_STATE();
    case 109:
      if (lookahead == 'c') ADVANCE(359);
      if (lookahead == 'e') ADVANCE(130);
      if (lookahead == 'g') ADVANCE(379);
      if (lookahead == 's') ADVANCE(50);
      if (lookahead == 't') ADVANCE(168);
      if (lookahead == 'x') ADVANCE(489);
      if (lookahead == '}') ADVANCE(674);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(673);
      END_STATE();
    case 110:
      if (lookahead == 'c') ADVANCE(275);
      END_STATE();
    case 111:
      if (lookahead == 'c') ADVANCE(347);
      if (lookahead == 'r') ADVANCE(155);
      END_STATE();
    case 112:
      if (lookahead == 'c') ADVANCE(352);
      END_STATE();
    case 113:
      if (lookahead == 'c') ADVANCE(582);
      END_STATE();
    case 114:
      if (lookahead == 'c') ADVANCE(348);
      END_STATE();
    case 115:
      if (lookahead == 'c') ADVANCE(666);
      END_STATE();
    case 116:
      if (lookahead == 'c') ADVANCE(349);
      END_STATE();
    case 117:
      if (lookahead == 'c') ADVANCE(270);
      END_STATE();
    case 118:
      if (lookahead == 'c') ADVANCE(313);
      END_STATE();
    case 119:
      if (lookahead == 'c') ADVANCE(357);
      if (lookahead == 'r') ADVANCE(155);
      END_STATE();
    case 120:
      if (lookahead == 'c') ADVANCE(642);
      END_STATE();
    case 121:
      if (lookahead == 'c') ADVANCE(272);
      END_STATE();
    case 122:
      if (lookahead == 'c') ADVANCE(351);
      END_STATE();
    case 123:
      if (lookahead == 'c') ADVANCE(606);
      END_STATE();
    case 124:
      if (lookahead == 'c') ADVANCE(587);
      END_STATE();
    case 125:
      if (lookahead == 'c') ADVANCE(160);
      END_STATE();
    case 126:
      if (lookahead == 'c') ADVANCE(162);
      END_STATE();
    case 127:
      if (lookahead == 'c') ADVANCE(165);
      END_STATE();
    case 128:
      if (lookahead == 'c') ADVANCE(515);
      END_STATE();
    case 129:
      if (lookahead == 'c') ADVANCE(532);
      END_STATE();
    case 130:
      if (lookahead == 'c') ADVANCE(373);
      if (lookahead == 'f') ADVANCE(231);
      if (lookahead == 'n') ADVANCE(110);
      END_STATE();
    case 131:
      if (lookahead == 'c') ADVANCE(537);
      END_STATE();
    case 132:
      if (lookahead == 'c') ADVANCE(627);
      END_STATE();
    case 133:
      if (lookahead == 'c') ADVANCE(650);
      END_STATE();
    case 134:
      if (lookahead == 'c') ADVANCE(631);
      END_STATE();
    case 135:
      if (lookahead == 'c') ADVANCE(632);
      END_STATE();
    case 136:
      if (lookahead == 'c') ADVANCE(633);
      END_STATE();
    case 137:
      if (lookahead == 'd') ADVANCE(688);
      END_STATE();
    case 138:
      if (lookahead == 'd') ADVANCE(22);
      END_STATE();
    case 139:
      if (lookahead == 'd') ADVANCE(22);
      if (lookahead == 'n') ADVANCE(210);
      END_STATE();
    case 140:
      if (lookahead == 'd') ADVANCE(698);
      END_STATE();
    case 141:
      if (lookahead == 'd') ADVANCE(761);
      END_STATE();
    case 142:
      if (lookahead == 'd') ADVANCE(403);
      if (lookahead == 'n') ADVANCE(176);
      END_STATE();
    case 143:
      if (lookahead == 'd') ADVANCE(137);
      END_STATE();
    case 144:
      if (lookahead == 'd') ADVANCE(648);
      END_STATE();
    case 145:
      if (lookahead == 'd') ADVANCE(57);
      if (lookahead == 'h') ADVANCE(199);
      END_STATE();
    case 146:
      if (lookahead == 'd') ADVANCE(550);
      END_STATE();
    case 147:
      if (lookahead == 'd') ADVANCE(202);
      END_STATE();
    case 148:
      if (lookahead == 'd') ADVANCE(323);
      END_STATE();
    case 149:
      if (lookahead == 'd') ADVANCE(307);
      END_STATE();
    case 150:
      if (lookahead == 'd') ADVANCE(436);
      END_STATE();
    case 151:
      if (lookahead == 'e') ADVANCE(651);
      END_STATE();
    case 152:
      if (lookahead == 'e') ADVANCE(512);
      END_STATE();
    case 153:
      if (lookahead == 'e') ADVANCE(766);
      END_STATE();
    case 154:
      if (lookahead == 'e') ADVANCE(260);
      END_STATE();
    case 155:
      if (lookahead == 'e') ADVANCE(758);
      END_STATE();
    case 156:
      if (lookahead == 'e') ADVANCE(755);
      END_STATE();
    case 157:
      if (lookahead == 'e') ADVANCE(700);
      END_STATE();
    case 158:
      if (lookahead == 'e') ADVANCE(743);
      END_STATE();
    case 159:
      if (lookahead == 'e') ADVANCE(752);
      END_STATE();
    case 160:
      if (lookahead == 'e') ADVANCE(708);
      END_STATE();
    case 161:
      if (lookahead == 'e') ADVANCE(741);
      END_STATE();
    case 162:
      if (lookahead == 'e') ADVANCE(728);
      END_STATE();
    case 163:
      if (lookahead == 'e') ADVANCE(704);
      END_STATE();
    case 164:
      if (lookahead == 'e') ADVANCE(701);
      END_STATE();
    case 165:
      if (lookahead == 'e') ADVANCE(709);
      END_STATE();
    case 166:
      if (lookahead == 'e') ADVANCE(760);
      END_STATE();
    case 167:
      if (lookahead == 'e') ADVANCE(730);
      END_STATE();
    case 168:
      if (lookahead == 'e') ADVANCE(361);
      if (lookahead == 'i') ADVANCE(381);
      END_STATE();
    case 169:
      if (lookahead == 'e') ADVANCE(54);
      END_STATE();
    case 170:
      if (lookahead == 'e') ADVANCE(493);
      END_STATE();
    case 171:
      if (lookahead == 'e') ADVANCE(493);
      if (lookahead == 'o') ADVANCE(362);
      END_STATE();
    case 172:
      if (lookahead == 'e') ADVANCE(113);
      END_STATE();
    case 173:
      if (lookahead == 'e') ADVANCE(653);
      if (lookahead == 'o') ADVANCE(463);
      if (lookahead == 'u') ADVANCE(111);
      END_STATE();
    case 174:
      if (lookahead == 'e') ADVANCE(653);
      if (lookahead == 'u') ADVANCE(122);
      END_STATE();
    case 175:
      if (lookahead == 'e') ADVANCE(261);
      END_STATE();
    case 176:
      if (lookahead == 'e') ADVANCE(128);
      END_STATE();
    case 177:
      if (lookahead == 'e') ADVANCE(9);
      END_STATE();
    case 178:
      if (lookahead == 'e') ADVANCE(358);
      END_STATE();
    case 179:
      if (lookahead == 'e') ADVANCE(581);
      END_STATE();
    case 180:
      if (lookahead == 'e') ADVANCE(499);
      END_STATE();
    case 181:
      if (lookahead == 'e') ADVANCE(140);
      END_STATE();
    case 182:
      if (lookahead == 'e') ADVANCE(87);
      if (lookahead == 'i') ADVANCE(518);
      if (lookahead == 'l') ADVANCE(79);
      if (lookahead == 'o') ADVANCE(519);
      if (lookahead == 'r') ADVANCE(481);
      END_STATE();
    case 183:
      if (lookahead == 'e') ADVANCE(87);
      if (lookahead == 'i') ADVANCE(529);
      if (lookahead == 'l') ADVANCE(79);
      if (lookahead == 'o') ADVANCE(519);
      if (lookahead == 'r') ADVANCE(481);
      END_STATE();
    case 184:
      if (lookahead == 'e') ADVANCE(141);
      END_STATE();
    case 185:
      if (lookahead == 'e') ADVANCE(506);
      END_STATE();
    case 186:
      if (lookahead == 'e') ADVANCE(517);
      END_STATE();
    case 187:
      if (lookahead == 'e') ADVANCE(47);
      END_STATE();
    case 188:
      if (lookahead == 'e') ADVANCE(507);
      END_STATE();
    case 189:
      if (lookahead == 'e') ADVANCE(558);
      END_STATE();
    case 190:
      if (lookahead == 'e') ADVANCE(63);
      END_STATE();
    case 191:
      if (lookahead == 'e') ADVANCE(508);
      END_STATE();
    case 192:
      if (lookahead == 'e') ADVANCE(427);
      END_STATE();
    case 193:
      if (lookahead == 'e') ADVANCE(407);
      END_STATE();
    case 194:
      if (lookahead == 'e') ADVANCE(181);
      END_STATE();
    case 195:
      if (lookahead == 'e') ADVANCE(509);
      END_STATE();
    case 196:
      if (lookahead == 'e') ADVANCE(68);
      END_STATE();
    case 197:
      if (lookahead == 'e') ADVANCE(180);
      if (lookahead == 'i') ADVANCE(239);
      END_STATE();
    case 198:
      if (lookahead == 'e') ADVANCE(30);
      END_STATE();
    case 199:
      if (lookahead == 'e') ADVANCE(85);
      END_STATE();
    case 200:
      if (lookahead == 'e') ADVANCE(510);
      END_STATE();
    case 201:
      if (lookahead == 'e') ADVANCE(12);
      END_STATE();
    case 202:
      if (lookahead == 'e') ADVANCE(511);
      END_STATE();
    case 203:
      if (lookahead == 'e') ADVANCE(555);
      END_STATE();
    case 204:
      if (lookahead == 'e') ADVANCE(35);
      END_STATE();
    case 205:
      if (lookahead == 'e') ADVANCE(48);
      END_STATE();
    case 206:
      if (lookahead == 'e') ADVANCE(390);
      END_STATE();
    case 207:
      if (lookahead == 'e') ADVANCE(414);
      END_STATE();
    case 208:
      if (lookahead == 'e') ADVANCE(184);
      END_STATE();
    case 209:
      if (lookahead == 'e') ADVANCE(514);
      END_STATE();
    case 210:
      if (lookahead == 'e') ADVANCE(23);
      END_STATE();
    case 211:
      if (lookahead == 'e') ADVANCE(59);
      END_STATE();
    case 212:
      if (lookahead == 'e') ADVANCE(123);
      END_STATE();
    case 213:
      if (lookahead == 'e') ADVANCE(556);
      END_STATE();
    case 214:
      if (lookahead == 'e') ADVANCE(535);
      END_STATE();
    case 215:
      if (lookahead == 'e') ADVANCE(560);
      END_STATE();
    case 216:
      if (lookahead == 'e') ADVANCE(132);
      END_STATE();
    case 217:
      if (lookahead == 'e') ADVANCE(562);
      END_STATE();
    case 218:
      if (lookahead == 'e') ADVANCE(124);
      END_STATE();
    case 219:
      if (lookahead == 'e') ADVANCE(76);
      END_STATE();
    case 220:
      if (lookahead == 'e') ADVANCE(564);
      END_STATE();
    case 221:
      if (lookahead == 'e') ADVANCE(129);
      END_STATE();
    case 222:
      if (lookahead == 'e') ADVANCE(440);
      END_STATE();
    case 223:
      if (lookahead == 'e') ADVANCE(440);
      if (lookahead == 'i') ADVANCE(431);
      END_STATE();
    case 224:
      if (lookahead == 'e') ADVANCE(29);
      END_STATE();
    case 225:
      if (lookahead == 'e') ADVANCE(131);
      END_STATE();
    case 226:
      if (lookahead == 'e') ADVANCE(134);
      END_STATE();
    case 227:
      if (lookahead == 'e') ADVANCE(135);
      END_STATE();
    case 228:
      if (lookahead == 'e') ADVANCE(540);
      END_STATE();
    case 229:
      if (lookahead == 'e') ADVANCE(136);
      END_STATE();
    case 230:
      if (lookahead == 'e') ADVANCE(40);
      END_STATE();
    case 231:
      if (lookahead == 'f') ADVANCE(172);
      END_STATE();
    case 232:
      if (lookahead == 'f') ADVANCE(233);
      END_STATE();
    case 233:
      if (lookahead == 'f') ADVANCE(294);
      END_STATE();
    case 234:
      if (lookahead == 'f') ADVANCE(327);
      END_STATE();
    case 235:
      if (lookahead == 'f') ADVANCE(327);
      if (lookahead == 's') ADVANCE(624);
      if (lookahead == 'v') ADVANCE(297);
      END_STATE();
    case 236:
      if (lookahead == 'f') ADVANCE(585);
      END_STATE();
    case 237:
      if (lookahead == 'f') ADVANCE(32);
      END_STATE();
    case 238:
      if (lookahead == 'f') ADVANCE(78);
      END_STATE();
    case 239:
      if (lookahead == 'f') ADVANCE(618);
      END_STATE();
    case 240:
      if (lookahead == 'f') ADVANCE(590);
      END_STATE();
    case 241:
      if (lookahead == 'f') ADVANCE(591);
      END_STATE();
    case 242:
      if (lookahead == 'f') ADVANCE(246);
      END_STATE();
    case 243:
      if (lookahead == 'f') ADVANCE(31);
      END_STATE();
    case 244:
      if (lookahead == 'f') ADVANCE(75);
      END_STATE();
    case 245:
      if (lookahead == 'f') ADVANCE(33);
      END_STATE();
    case 246:
      if (lookahead == 'f') ADVANCE(342);
      END_STATE();
    case 247:
      if (lookahead == 'f') ADVANCE(96);
      END_STATE();
    case 248:
      if (lookahead == 'g') ADVANCE(721);
      END_STATE();
    case 249:
      if (lookahead == 'g') ADVANCE(747);
      END_STATE();
    case 250:
      if (lookahead == 'g') ADVANCE(759);
      END_STATE();
    case 251:
      if (lookahead == 'g') ADVANCE(748);
      END_STATE();
    case 252:
      if (lookahead == 'g') ADVANCE(751);
      END_STATE();
    case 253:
      if (lookahead == 'g') ADVANCE(726);
      END_STATE();
    case 254:
      if (lookahead == 'g') ADVANCE(733);
      END_STATE();
    case 255:
      if (lookahead == 'g') ADVANCE(710);
      END_STATE();
    case 256:
      if (lookahead == 'g') ADVANCE(274);
      END_STATE();
    case 257:
      if (lookahead == 'g') ADVANCE(276);
      END_STATE();
    case 258:
      if (lookahead == 'g') ADVANCE(28);
      END_STATE();
    case 259:
      if (lookahead == 'g') ADVANCE(16);
      END_STATE();
    case 260:
      if (lookahead == 'g') ADVANCE(192);
      if (lookahead == 's') ADVANCE(330);
      END_STATE();
    case 261:
      if (lookahead == 'g') ADVANCE(192);
      if (lookahead == 's') ADVANCE(329);
      END_STATE();
    case 262:
      if (lookahead == 'g') ADVANCE(188);
      END_STATE();
    case 263:
      if (lookahead == 'g') ADVANCE(602);
      END_STATE();
    case 264:
      if (lookahead == 'g') ADVANCE(163);
      END_STATE();
    case 265:
      if (lookahead == 'g') ADVANCE(167);
      END_STATE();
    case 266:
      if (lookahead == 'g') ADVANCE(643);
      END_STATE();
    case 267:
      if (lookahead == 'g') ADVANCE(280);
      END_STATE();
    case 268:
      if (lookahead == 'g') ADVANCE(524);
      END_STATE();
    case 269:
      if (lookahead == 'g') ADVANCE(39);
      END_STATE();
    case 270:
      if (lookahead == 'h') ADVANCE(754);
      END_STATE();
    case 271:
      if (lookahead == 'h') ADVANCE(702);
      END_STATE();
    case 272:
      if (lookahead == 'h') ADVANCE(750);
      END_STATE();
    case 273:
      if (lookahead == 'h') ADVANCE(703);
      END_STATE();
    case 274:
      if (lookahead == 'h') ADVANCE(580);
      END_STATE();
    case 275:
      if (lookahead == 'h') ADVANCE(61);
      END_STATE();
    case 276:
      if (lookahead == 'h') ADVANCE(584);
      END_STATE();
    case 277:
      if (lookahead == 'h') ADVANCE(34);
      END_STATE();
    case 278:
      if (lookahead == 'h') ADVANCE(458);
      END_STATE();
    case 279:
      if (lookahead == 'h') ADVANCE(523);
      END_STATE();
    case 280:
      if (lookahead == 'h') ADVANCE(603);
      END_STATE();
    case 281:
      if (lookahead == 'h') ADVANCE(191);
      END_STATE();
    case 282:
      if (lookahead == 'h') ADVANCE(224);
      END_STATE();
    case 283:
      if (lookahead == 'h') ADVANCE(198);
      END_STATE();
    case 284:
      if (lookahead == 'h') ADVANCE(214);
      END_STATE();
    case 285:
      if (lookahead == 'h') ADVANCE(64);
      if (lookahead == 'i') ADVANCE(360);
      if (lookahead == 'm') ADVANCE(326);
      if (lookahead == 'o') ADVANCE(640);
      if (lookahead == 'w') ADVANCE(197);
      END_STATE();
    case 286:
      if (lookahead == 'h') ADVANCE(308);
      END_STATE();
    case 287:
      if (lookahead == 'h') ADVANCE(317);
      END_STATE();
    case 288:
      if (lookahead == 'h') ADVANCE(37);
      END_STATE();
    case 289:
      if (lookahead == 'h') ADVANCE(344);
      END_STATE();
    case 290:
      if (lookahead == 'i') ADVANCE(142);
      END_STATE();
    case 291:
      if (lookahead == 'i') ADVANCE(256);
      if (lookahead == 'o') ADVANCE(450);
      END_STATE();
    case 292:
      if (lookahead == 'i') ADVANCE(102);
      END_STATE();
    case 293:
      if (lookahead == 'i') ADVANCE(405);
      END_STATE();
    case 294:
      if (lookahead == 'i') ADVANCE(118);
      END_STATE();
    case 295:
      if (lookahead == 'i') ADVANCE(574);
      END_STATE();
    case 296:
      if (lookahead == 'i') ADVANCE(574);
      if (lookahead == 'w') ADVANCE(185);
      END_STATE();
    case 297:
      if (lookahead == 'i') ADVANCE(553);
      END_STATE();
    case 298:
      if (lookahead == 'i') ADVANCE(147);
      END_STATE();
    case 299:
      if (lookahead == 'i') ADVANCE(577);
      END_STATE();
    case 300:
      if (lookahead == 'i') ADVANCE(374);
      END_STATE();
    case 301:
      if (lookahead == 'i') ADVANCE(266);
      END_STATE();
    case 302:
      if (lookahead == 'i') ADVANCE(411);
      END_STATE();
    case 303:
      if (lookahead == 'i') ADVANCE(404);
      END_STATE();
    case 304:
      if (lookahead == 'i') ADVANCE(410);
      END_STATE();
    case 305:
      if (lookahead == 'i') ADVANCE(412);
      END_STATE();
    case 306:
      if (lookahead == 'i') ADVANCE(619);
      END_STATE();
    case 307:
      if (lookahead == 'i') ADVANCE(415);
      END_STATE();
    case 308:
      if (lookahead == 'i') ADVANCE(425);
      END_STATE();
    case 309:
      if (lookahead == 'i') ADVANCE(417);
      END_STATE();
    case 310:
      if (lookahead == 'i') ADVANCE(609);
      END_STATE();
    case 311:
      if (lookahead == 'i') ADVANCE(592);
      END_STATE();
    case 312:
      if (lookahead == 'i') ADVANCE(418);
      END_STATE();
    case 313:
      if (lookahead == 'i') ADVANCE(207);
      END_STATE();
    case 314:
      if (lookahead == 'i') ADVANCE(421);
      END_STATE();
    case 315:
      if (lookahead == 'i') ADVANCE(596);
      END_STATE();
    case 316:
      if (lookahead == 'i') ADVANCE(422);
      END_STATE();
    case 317:
      if (lookahead == 'i') ADVANCE(424);
      END_STATE();
    case 318:
      if (lookahead == 'i') ADVANCE(598);
      END_STATE();
    case 319:
      if (lookahead == 'i') ADVANCE(442);
      END_STATE();
    case 320:
      if (lookahead == 'i') ADVANCE(371);
      END_STATE();
    case 321:
      if (lookahead == 'i') ADVANCE(257);
      END_STATE();
    case 322:
      if (lookahead == 'i') ADVANCE(431);
      END_STATE();
    case 323:
      if (lookahead == 'i') ADVANCE(432);
      END_STATE();
    case 324:
      if (lookahead == 'i') ADVANCE(369);
      END_STATE();
    case 325:
      if (lookahead == 'i') ADVANCE(578);
      END_STATE();
    case 326:
      if (lookahead == 'i') ADVANCE(615);
      END_STATE();
    case 327:
      if (lookahead == 'i') ADVANCE(437);
      END_STATE();
    case 328:
      if (lookahead == 'i') ADVANCE(466);
      END_STATE();
    case 329:
      if (lookahead == 'i') ADVANCE(573);
      END_STATE();
    case 330:
      if (lookahead == 'i') ADVANCE(573);
      if (lookahead == 'p') ADVANCE(345);
      END_STATE();
    case 331:
      if (lookahead == 'i') ADVANCE(467);
      END_STATE();
    case 332:
      if (lookahead == 'i') ADVANCE(468);
      END_STATE();
    case 333:
      if (lookahead == 'i') ADVANCE(530);
      END_STATE();
    case 334:
      if (lookahead == 'i') ADVANCE(470);
      END_STATE();
    case 335:
      if (lookahead == 'i') ADVANCE(575);
      END_STATE();
    case 336:
      if (lookahead == 'i') ADVANCE(471);
      END_STATE();
    case 337:
      if (lookahead == 'i') ADVANCE(472);
      END_STATE();
    case 338:
      if (lookahead == 'i') ADVANCE(473);
      END_STATE();
    case 339:
      if (lookahead == 'i') ADVANCE(474);
      END_STATE();
    case 340:
      if (lookahead == 'i') ADVANCE(475);
      END_STATE();
    case 341:
      if (lookahead == 'i') ADVANCE(476);
      END_STATE();
    case 342:
      if (lookahead == 'i') ADVANCE(439);
      END_STATE();
    case 343:
      if (lookahead == 'i') ADVANCE(443);
      END_STATE();
    case 344:
      if (lookahead == 'i') ADVANCE(445);
      END_STATE();
    case 345:
      if (lookahead == 'i') ADVANCE(539);
      END_STATE();
    case 346:
      if (lookahead == 'j') ADVANCE(212);
      if (lookahead == 't') ADVANCE(216);
      END_STATE();
    case 347:
      if (lookahead == 'k') ADVANCE(724);
      END_STATE();
    case 348:
      if (lookahead == 'k') ADVANCE(725);
      END_STATE();
    case 349:
      if (lookahead == 'k') ADVANCE(745);
      END_STATE();
    case 350:
      if (lookahead == 'k') ADVANCE(762);
      END_STATE();
    case 351:
      if (lookahead == 'k') ADVANCE(723);
      END_STATE();
    case 352:
      if (lookahead == 'k') ADVANCE(103);
      END_STATE();
    case 353:
      if (lookahead == 'k') ADVANCE(26);
      END_STATE();
    case 354:
      if (lookahead == 'k') ADVANCE(195);
      END_STATE();
    case 355:
      if (lookahead == 'k') ADVANCE(434);
      END_STATE();
    case 356:
      if (lookahead == 'k') ADVANCE(312);
      END_STATE();
    case 357:
      if (lookahead == 'k') ADVANCE(36);
      END_STATE();
    case 358:
      if (lookahead == 'l') ADVANCE(543);
      END_STATE();
    case 359:
      if (lookahead == 'l') ADVANCE(169);
      END_STATE();
    case 360:
      if (lookahead == 'l') ADVANCE(353);
      END_STATE();
    case 361:
      if (lookahead == 'l') ADVANCE(363);
      END_STATE();
    case 362:
      if (lookahead == 'l') ADVANCE(491);
      END_STATE();
    case 363:
      if (lookahead == 'l') ADVANCE(513);
      END_STATE();
    case 364:
      if (lookahead == 'l') ADVANCE(449);
      END_STATE();
    case 365:
      if (lookahead == 'l') ADVANCE(19);
      END_STATE();
    case 366:
      if (lookahead == 'l') ADVANCE(645);
      END_STATE();
    case 367:
      if (lookahead == 'l') ADVANCE(354);
      END_STATE();
    case 368:
      if (lookahead == 'l') ADVANCE(375);
      END_STATE();
    case 369:
      if (lookahead == 'l') ADVANCE(378);
      END_STATE();
    case 370:
      if (lookahead == 'l') ADVANCE(600);
      END_STATE();
    case 371:
      if (lookahead == 'l') ADVANCE(230);
      END_STATE();
    case 372:
      if (lookahead == 'l') ADVANCE(607);
      END_STATE();
    case 373:
      if (lookahead == 'l') ADVANCE(211);
      END_STATE();
    case 374:
      if (lookahead == 'l') ADVANCE(315);
      END_STATE();
    case 375:
      if (lookahead == 'l') ADVANCE(314);
      END_STATE();
    case 376:
      if (lookahead == 'l') ADVANCE(316);
      END_STATE();
    case 377:
      if (lookahead == 'l') ADVANCE(376);
      END_STATE();
    case 378:
      if (lookahead == 'l') ADVANCE(91);
      END_STATE();
    case 379:
      if (lookahead == 'm') ADVANCE(43);
      END_STATE();
    case 380:
      if (lookahead == 'm') ADVANCE(490);
      END_STATE();
    case 381:
      if (lookahead == 'm') ADVANCE(153);
      END_STATE();
    case 382:
      if (lookahead == 'm') ADVANCE(156);
      END_STATE();
    case 383:
      if (lookahead == 'm') ADVANCE(80);
      END_STATE();
    case 384:
      if (lookahead == 'm') ADVANCE(206);
      END_STATE();
    case 385:
      if (lookahead == 'n') ADVANCE(670);
      END_STATE();
    case 386:
      if (lookahead == 'n') ADVANCE(775);
      END_STATE();
    case 387:
      if (lookahead == 'n') ADVANCE(235);
      END_STATE();
    case 388:
      if (lookahead == 'n') ADVANCE(106);
      END_STATE();
    case 389:
      if (lookahead == 'n') ADVANCE(716);
      END_STATE();
    case 390:
      if (lookahead == 'n') ADVANCE(729);
      END_STATE();
    case 391:
      if (lookahead == 'n') ADVANCE(719);
      END_STATE();
    case 392:
      if (lookahead == 'n') ADVANCE(722);
      END_STATE();
    case 393:
      if (lookahead == 'n') ADVANCE(731);
      END_STATE();
    case 394:
      if (lookahead == 'n') ADVANCE(720);
      END_STATE();
    case 395:
      if (lookahead == 'n') ADVANCE(736);
      END_STATE();
    case 396:
      if (lookahead == 'n') ADVANCE(713);
      END_STATE();
    case 397:
      if (lookahead == 'n') ADVANCE(707);
      END_STATE();
    case 398:
      if (lookahead == 'n') ADVANCE(732);
      END_STATE();
    case 399:
      if (lookahead == 'n') ADVANCE(734);
      END_STATE();
    case 400:
      if (lookahead == 'n') ADVANCE(735);
      END_STATE();
    case 401:
      if (lookahead == 'n') ADVANCE(557);
      END_STATE();
    case 402:
      if (lookahead == 'n') ADVANCE(262);
      END_STATE();
    case 403:
      if (lookahead == 'n') ADVANCE(321);
      END_STATE();
    case 404:
      if (lookahead == 'n') ADVANCE(258);
      END_STATE();
    case 405:
      if (lookahead == 'n') ADVANCE(589);
      END_STATE();
    case 406:
      if (lookahead == 'n') ADVANCE(148);
      END_STATE();
    case 407:
      if (lookahead == 'n') ADVANCE(263);
      END_STATE();
    case 408:
      if (lookahead == 'n') ADVANCE(144);
      END_STATE();
    case 409:
      if (lookahead == 'n') ADVANCE(452);
      END_STATE();
    case 410:
      if (lookahead == 'n') ADVANCE(248);
      END_STATE();
    case 411:
      if (lookahead == 'n') ADVANCE(150);
      END_STATE();
    case 412:
      if (lookahead == 'n') ADVANCE(249);
      END_STATE();
    case 413:
      if (lookahead == 'n') ADVANCE(583);
      END_STATE();
    case 414:
      if (lookahead == 'n') ADVANCE(115);
      END_STATE();
    case 415:
      if (lookahead == 'n') ADVANCE(250);
      END_STATE();
    case 416:
      if (lookahead == 'n') ADVANCE(125);
      END_STATE();
    case 417:
      if (lookahead == 'n') ADVANCE(251);
      END_STATE();
    case 418:
      if (lookahead == 'n') ADVANCE(252);
      END_STATE();
    case 419:
      if (lookahead == 'n') ADVANCE(545);
      END_STATE();
    case 420:
      if (lookahead == 'n') ADVANCE(210);
      END_STATE();
    case 421:
      if (lookahead == 'n') ADVANCE(253);
      END_STATE();
    case 422:
      if (lookahead == 'n') ADVANCE(254);
      END_STATE();
    case 423:
      if (lookahead == 'n') ADVANCE(366);
      END_STATE();
    case 424:
      if (lookahead == 'n') ADVANCE(255);
      END_STATE();
    case 425:
      if (lookahead == 'n') ADVANCE(563);
      END_STATE();
    case 426:
      if (lookahead == 'n') ADVANCE(612);
      END_STATE();
    case 427:
      if (lookahead == 'n') ADVANCE(228);
      END_STATE();
    case 428:
      if (lookahead == 'n') ADVANCE(159);
      END_STATE();
    case 429:
      if (lookahead == 'n') ADVANCE(196);
      END_STATE();
    case 430:
      if (lookahead == 'n') ADVANCE(117);
      END_STATE();
    case 431:
      if (lookahead == 'n') ADVANCE(303);
      END_STATE();
    case 432:
      if (lookahead == 'n') ADVANCE(259);
      END_STATE();
    case 433:
      if (lookahead == 'n') ADVANCE(299);
      END_STATE();
    case 434:
      if (lookahead == 'n') ADVANCE(215);
      END_STATE();
    case 435:
      if (lookahead == 'n') ADVANCE(127);
      END_STATE();
    case 436:
      if (lookahead == 'n') ADVANCE(217);
      END_STATE();
    case 437:
      if (lookahead == 'n') ADVANCE(311);
      END_STATE();
    case 438:
      if (lookahead == 'n') ADVANCE(220);
      END_STATE();
    case 439:
      if (lookahead == 'n') ADVANCE(318);
      END_STATE();
    case 440:
      if (lookahead == 'n') ADVANCE(149);
      END_STATE();
    case 441:
      if (lookahead == 'n') ADVANCE(105);
      END_STATE();
    case 442:
      if (lookahead == 'n') ADVANCE(221);
      END_STATE();
    case 443:
      if (lookahead == 'n') ADVANCE(225);
      END_STATE();
    case 444:
      if (lookahead == 'n') ADVANCE(234);
      END_STATE();
    case 445:
      if (lookahead == 'n') ADVANCE(269);
      END_STATE();
    case 446:
      if (lookahead == 'o') ADVANCE(296);
      if (lookahead == 'r') ADVANCE(447);
      if (lookahead == 'u') ADVANCE(430);
      END_STATE();
    case 447:
      if (lookahead == 'o') ADVANCE(346);
      END_STATE();
    case 448:
      if (lookahead == 'o') ADVANCE(293);
      END_STATE();
    case 449:
      if (lookahead == 'o') ADVANCE(660);
      END_STATE();
    case 450:
      if (lookahead == 'o') ADVANCE(386);
      END_STATE();
    case 451:
      if (lookahead == 'o') ADVANCE(656);
      END_STATE();
    case 452:
      if (lookahead == 'o') ADVANCE(112);
      END_STATE();
    case 453:
      if (lookahead == 'o') ADVANCE(659);
      END_STATE();
    case 454:
      if (lookahead == 'o') ADVANCE(362);
      END_STATE();
    case 455:
      if (lookahead == 'o') ADVANCE(463);
      if (lookahead == 'u') ADVANCE(119);
      END_STATE();
    case 456:
      if (lookahead == 'o') ADVANCE(237);
      END_STATE();
    case 457:
      if (lookahead == 'o') ADVANCE(243);
      END_STATE();
    case 458:
      if (lookahead == 'o') ADVANCE(521);
      END_STATE();
    case 459:
      if (lookahead == 'o') ADVANCE(531);
      END_STATE();
    case 460:
      if (lookahead == 'o') ADVANCE(146);
      END_STATE();
    case 461:
      if (lookahead == 'o') ADVANCE(408);
      END_STATE();
    case 462:
      if (lookahead == 'o') ADVANCE(646);
      END_STATE();
    case 463:
      if (lookahead == 'o') ADVANCE(623);
      END_STATE();
    case 464:
      if (lookahead == 'o') ADVANCE(295);
      END_STATE();
    case 465:
      if (lookahead == 'o') ADVANCE(389);
      END_STATE();
    case 466:
      if (lookahead == 'o') ADVANCE(391);
      END_STATE();
    case 467:
      if (lookahead == 'o') ADVANCE(392);
      END_STATE();
    case 468:
      if (lookahead == 'o') ADVANCE(393);
      END_STATE();
    case 469:
      if (lookahead == 'o') ADVANCE(634);
      END_STATE();
    case 470:
      if (lookahead == 'o') ADVANCE(394);
      END_STATE();
    case 471:
      if (lookahead == 'o') ADVANCE(395);
      END_STATE();
    case 472:
      if (lookahead == 'o') ADVANCE(396);
      END_STATE();
    case 473:
      if (lookahead == 'o') ADVANCE(397);
      END_STATE();
    case 474:
      if (lookahead == 'o') ADVANCE(398);
      END_STATE();
    case 475:
      if (lookahead == 'o') ADVANCE(399);
      END_STATE();
    case 476:
      if (lookahead == 'o') ADVANCE(400);
      END_STATE();
    case 477:
      if (lookahead == 'o') ADVANCE(657);
      if (lookahead == 'r') ADVANCE(447);
      if (lookahead == 'u') ADVANCE(430);
      END_STATE();
    case 478:
      if (lookahead == 'o') ADVANCE(495);
      END_STATE();
    case 479:
      if (lookahead == 'o') ADVANCE(25);
      END_STATE();
    case 480:
      if (lookahead == 'o') ADVANCE(384);
      END_STATE();
    case 481:
      if (lookahead == 'o') ADVANCE(561);
      END_STATE();
    case 482:
      if (lookahead == 'o') ADVANCE(567);
      END_STATE();
    case 483:
      if (lookahead == 'o') ADVANCE(568);
      END_STATE();
    case 484:
      if (lookahead == 'o') ADVANCE(482);
      END_STATE();
    case 485:
      if (lookahead == 'o') ADVANCE(483);
      END_STATE();
    case 486:
      if (lookahead == 'o') ADVANCE(245);
      END_STATE();
    case 487:
      if (lookahead == 'o') ADVANCE(635);
      END_STATE();
    case 488:
      if (lookahead == 'o') ADVANCE(636);
      END_STATE();
    case 489:
      if (lookahead == 'p') ADVANCE(44);
      END_STATE();
    case 490:
      if (lookahead == 'p') ADVANCE(10);
      END_STATE();
    case 491:
      if (lookahead == 'p') ADVANCE(286);
      END_STATE();
    case 492:
      if (lookahead == 'p') ADVANCE(345);
      END_STATE();
    case 493:
      if (lookahead == 'p') ADVANCE(595);
      END_STATE();
    case 494:
      if (lookahead == 'p') ADVANCE(453);
      END_STATE();
    case 495:
      if (lookahead == 'p') ADVANCE(460);
      END_STATE();
    case 496:
      if (lookahead == 'p') ADVANCE(604);
      END_STATE();
    case 497:
      if (lookahead == 'p') ADVANCE(208);
      END_STATE();
    case 498:
      if (lookahead == 'p') ADVANCE(218);
      END_STATE();
    case 499:
      if (lookahead == 'p') ADVANCE(309);
      END_STATE();
    case 500:
      if (lookahead == 'p') ADVANCE(438);
      END_STATE();
    case 501:
      if (lookahead == 'p') ADVANCE(541);
      END_STATE();
    case 502:
      if (lookahead == 'p') ADVANCE(542);
      END_STATE();
    case 503:
      if (lookahead == 'q') ADVANCE(641);
      END_STATE();
    case 504:
      if (lookahead == 'r') ADVANCE(693);
      END_STATE();
    case 505:
      if (lookahead == 'r') ADVANCE(694);
      END_STATE();
    case 506:
      if (lookahead == 'r') ADVANCE(753);
      END_STATE();
    case 507:
      if (lookahead == 'r') ADVANCE(714);
      END_STATE();
    case 508:
      if (lookahead == 'r') ADVANCE(717);
      END_STATE();
    case 509:
      if (lookahead == 'r') ADVANCE(740);
      END_STATE();
    case 510:
      if (lookahead == 'r') ADVANCE(727);
      END_STATE();
    case 511:
      if (lookahead == 'r') ADVANCE(739);
      END_STATE();
    case 512:
      if (lookahead == 'r') ADVANCE(663);
      END_STATE();
    case 513:
      if (lookahead == 'r') ADVANCE(45);
      END_STATE();
    case 514:
      if (lookahead == 'r') ADVANCE(664);
      END_STATE();
    case 515:
      if (lookahead == 'r') ADVANCE(52);
      END_STATE();
    case 516:
      if (lookahead == 'r') ADVANCE(500);
      END_STATE();
    case 517:
      if (lookahead == 'r') ADVANCE(14);
      END_STATE();
    case 518:
      if (lookahead == 'r') ADVANCE(177);
      END_STATE();
    case 519:
      if (lookahead == 'r') ADVANCE(599);
      END_STATE();
    case 520:
      if (lookahead == 'r') ADVANCE(298);
      END_STATE();
    case 521:
      if (lookahead == 'r') ADVANCE(419);
      END_STATE();
    case 522:
      if (lookahead == 'r') ADVANCE(193);
      END_STATE();
    case 523:
      if (lookahead == 'r') ADVANCE(478);
      END_STATE();
    case 524:
      if (lookahead == 'r') ADVANCE(86);
      END_STATE();
    case 525:
      if (lookahead == 'r') ADVANCE(190);
      END_STATE();
    case 526:
      if (lookahead == 'r') ADVANCE(213);
      END_STATE();
    case 527:
      if (lookahead == 'r') ADVANCE(605);
      END_STATE();
    case 528:
      if (lookahead == 'r') ADVANCE(219);
      END_STATE();
    case 529:
      if (lookahead == 'r') ADVANCE(201);
      END_STATE();
    case 530:
      if (lookahead == 'r') ADVANCE(204);
      END_STATE();
    case 531:
      if (lookahead == 'r') ADVANCE(496);
      END_STATE();
    case 532:
      if (lookahead == 'r') ADVANCE(90);
      END_STATE();
    case 533:
      if (lookahead == 'r') ADVANCE(569);
      END_STATE();
    case 534:
      if (lookahead == 'r') ADVANCE(469);
      END_STATE();
    case 535:
      if (lookahead == 'r') ADVANCE(38);
      END_STATE();
    case 536:
      if (lookahead == 'r') ADVANCE(571);
      END_STATE();
    case 537:
      if (lookahead == 'r') ADVANCE(93);
      END_STATE();
    case 538:
      if (lookahead == 'r') ADVANCE(97);
      END_STATE();
    case 539:
      if (lookahead == 'r') ADVANCE(98);
      END_STATE();
    case 540:
      if (lookahead == 'r') ADVANCE(99);
      END_STATE();
    case 541:
      if (lookahead == 'r') ADVANCE(487);
      END_STATE();
    case 542:
      if (lookahead == 'r') ADVANCE(488);
      END_STATE();
    case 543:
      if (lookahead == 's') ADVANCE(685);
      END_STATE();
    case 544:
      if (lookahead == 's') ADVANCE(686);
      END_STATE();
    case 545:
      if (lookahead == 's') ADVANCE(738);
      END_STATE();
    case 546:
      if (lookahead == 's') ADVANCE(699);
      END_STATE();
    case 547:
      if (lookahead == 's') ADVANCE(715);
      END_STATE();
    case 548:
      if (lookahead == 's') ADVANCE(712);
      END_STATE();
    case 549:
      if (lookahead == 's') ADVANCE(742);
      END_STATE();
    case 550:
      if (lookahead == 's') ADVANCE(744);
      END_STATE();
    case 551:
      if (lookahead == 's') ADVANCE(497);
      END_STATE();
    case 552:
      if (lookahead == 's') ADVANCE(498);
      END_STATE();
    case 553:
      if (lookahead == 's') ADVANCE(292);
      END_STATE();
    case 554:
      if (lookahead == 's') ADVANCE(459);
      END_STATE();
    case 555:
      if (lookahead == 's') ADVANCE(492);
      END_STATE();
    case 556:
      if (lookahead == 's') ADVANCE(335);
      END_STATE();
    case 557:
      if (lookahead == 's') ADVANCE(624);
      if (lookahead == 'v') ADVANCE(297);
      END_STATE();
    case 558:
      if (lookahead == 's') ADVANCE(546);
      END_STATE();
    case 559:
      if (lookahead == 's') ADVANCE(597);
      END_STATE();
    case 560:
      if (lookahead == 's') ADVANCE(547);
      END_STATE();
    case 561:
      if (lookahead == 's') ADVANCE(601);
      END_STATE();
    case 562:
      if (lookahead == 's') ADVANCE(548);
      END_STATE();
    case 563:
      if (lookahead == 's') ADVANCE(18);
      END_STATE();
    case 564:
      if (lookahead == 's') ADVANCE(549);
      END_STATE();
    case 565:
      if (lookahead == 's') ADVANCE(187);
      END_STATE();
    case 566:
      if (lookahead == 's') ADVANCE(608);
      END_STATE();
    case 567:
      if (lookahead == 's') ADVANCE(586);
      END_STATE();
    case 568:
      if (lookahead == 's') ADVANCE(588);
      END_STATE();
    case 569:
      if (lookahead == 's') ADVANCE(161);
      END_STATE();
    case 570:
      if (lookahead == 's') ADVANCE(205);
      END_STATE();
    case 571:
      if (lookahead == 's') ADVANCE(166);
      END_STATE();
    case 572:
      if (lookahead == 's') ADVANCE(614);
      END_STATE();
    case 573:
      if (lookahead == 's') ADVANCE(616);
      END_STATE();
    case 574:
      if (lookahead == 's') ADVANCE(465);
      END_STATE();
    case 575:
      if (lookahead == 's') ADVANCE(620);
      END_STATE();
    case 576:
      if (lookahead == 's') ADVANCE(429);
      END_STATE();
    case 577:
      if (lookahead == 's') ADVANCE(289);
      END_STATE();
    case 578:
      if (lookahead == 's') ADVANCE(337);
      END_STATE();
    case 579:
      if (lookahead == 't') ADVANCE(767);
      END_STATE();
    case 580:
      if (lookahead == 't') ADVANCE(774);
      END_STATE();
    case 581:
      if (lookahead == 't') ADVANCE(689);
      END_STATE();
    case 582:
      if (lookahead == 't') ADVANCE(696);
      END_STATE();
    case 583:
      if (lookahead == 't') ADVANCE(697);
      END_STATE();
    case 584:
      if (lookahead == 't') ADVANCE(776);
      END_STATE();
    case 585:
      if (lookahead == 't') ADVANCE(6);
      END_STATE();
    case 586:
      if (lookahead == 't') ADVANCE(705);
      END_STATE();
    case 587:
      if (lookahead == 't') ADVANCE(746);
      END_STATE();
    case 588:
      if (lookahead == 't') ADVANCE(718);
      END_STATE();
    case 589:
      if (lookahead == 't') ADVANCE(544);
      END_STATE();
    case 590:
      if (lookahead == 't') ADVANCE(7);
      END_STATE();
    case 591:
      if (lookahead == 't') ADVANCE(8);
      END_STATE();
    case 592:
      if (lookahead == 't') ADVANCE(665);
      END_STATE();
    case 593:
      if (lookahead == 't') ADVANCE(281);
      END_STATE();
    case 594:
      if (lookahead == 't') ADVANCE(647);
      END_STATE();
    case 595:
      if (lookahead == 't') ADVANCE(277);
      END_STATE();
    case 596:
      if (lookahead == 't') ADVANCE(667);
      END_STATE();
    case 597:
      if (lookahead == 't') ADVANCE(17);
      END_STATE();
    case 598:
      if (lookahead == 't') ADVANCE(668);
      END_STATE();
    case 599:
      if (lookahead == 't') ADVANCE(649);
      END_STATE();
    case 600:
      if (lookahead == 't') ADVANCE(288);
      END_STATE();
    case 601:
      if (lookahead == 't') ADVANCE(13);
      END_STATE();
    case 602:
      if (lookahead == 't') ADVANCE(271);
      END_STATE();
    case 603:
      if (lookahead == 't') ADVANCE(27);
      END_STATE();
    case 604:
      if (lookahead == 't') ADVANCE(328);
      END_STATE();
    case 605:
      if (lookahead == 't') ADVANCE(279);
      END_STATE();
    case 606:
      if (lookahead == 't') ADVANCE(320);
      END_STATE();
    case 607:
      if (lookahead == 't') ADVANCE(273);
      END_STATE();
    case 608:
      if (lookahead == 't') ADVANCE(520);
      END_STATE();
    case 609:
      if (lookahead == 't') ADVANCE(20);
      END_STATE();
    case 610:
      if (lookahead == 't') ADVANCE(301);
      END_STATE();
    case 611:
      if (lookahead == 't') ADVANCE(462);
      END_STATE();
    case 612:
      if (lookahead == 't') ADVANCE(11);
      END_STATE();
    case 613:
      if (lookahead == 't') ADVANCE(186);
      END_STATE();
    case 614:
      if (lookahead == 't') ADVANCE(157);
      END_STATE();
    case 615:
      if (lookahead == 't') ADVANCE(158);
      END_STATE();
    case 616:
      if (lookahead == 't') ADVANCE(74);
      END_STATE();
    case 617:
      if (lookahead == 't') ADVANCE(284);
      END_STATE();
    case 618:
      if (lookahead == 't') ADVANCE(24);
      END_STATE();
    case 619:
      if (lookahead == 't') ADVANCE(95);
      END_STATE();
    case 620:
      if (lookahead == 't') ADVANCE(88);
      END_STATE();
    case 621:
      if (lookahead == 't') ADVANCE(282);
      END_STATE();
    case 622:
      if (lookahead == 't') ADVANCE(283);
      END_STATE();
    case 623:
      if (lookahead == 't') ADVANCE(305);
      END_STATE();
    case 624:
      if (lookahead == 't') ADVANCE(83);
      END_STATE();
    case 625:
      if (lookahead == 't') ADVANCE(331);
      END_STATE();
    case 626:
      if (lookahead == 't') ADVANCE(287);
      END_STATE();
    case 627:
      if (lookahead == 't') ADVANCE(332);
      END_STATE();
    case 628:
      if (lookahead == 't') ADVANCE(334);
      END_STATE();
    case 629:
      if (lookahead == 't') ADVANCE(336);
      END_STATE();
    case 630:
      if (lookahead == 't') ADVANCE(338);
      END_STATE();
    case 631:
      if (lookahead == 't') ADVANCE(339);
      END_STATE();
    case 632:
      if (lookahead == 't') ADVANCE(340);
      END_STATE();
    case 633:
      if (lookahead == 't') ADVANCE(341);
      END_STATE();
    case 634:
      if (lookahead == 't') ADVANCE(226);
      END_STATE();
    case 635:
      if (lookahead == 't') ADVANCE(227);
      END_STATE();
    case 636:
      if (lookahead == 't') ADVANCE(229);
      END_STATE();
    case 637:
      if (lookahead == 'u') ADVANCE(380);
      END_STATE();
    case 638:
      if (lookahead == 'u') ADVANCE(152);
      END_STATE();
    case 639:
      if (lookahead == 'u') ADVANCE(565);
      END_STATE();
    case 640:
      if (lookahead == 'u') ADVANCE(365);
      END_STATE();
    case 641:
      if (lookahead == 'u') ADVANCE(55);
      END_STATE();
    case 642:
      if (lookahead == 'u') ADVANCE(533);
      END_STATE();
    case 643:
      if (lookahead == 'u') ADVANCE(164);
      END_STATE();
    case 644:
      if (lookahead == 'u') ADVANCE(209);
      END_STATE();
    case 645:
      if (lookahead == 'u') ADVANCE(114);
      END_STATE();
    case 646:
      if (lookahead == 'u') ADVANCE(121);
      END_STATE();
    case 647:
      if (lookahead == 'u') ADVANCE(538);
      END_STATE();
    case 648:
      if (lookahead == 'u') ADVANCE(310);
      END_STATE();
    case 649:
      if (lookahead == 'u') ADVANCE(428);
      END_STATE();
    case 650:
      if (lookahead == 'u') ADVANCE(536);
      END_STATE();
    case 651:
      if (lookahead == 'v') ADVANCE(178);
      END_STATE();
    case 652:
      if (lookahead == 'v') ADVANCE(324);
      END_STATE();
    case 653:
      if (lookahead == 'v') ADVANCE(306);
      END_STATE();
    case 654:
      if (lookahead == 'v') ADVANCE(325);
      END_STATE();
    case 655:
      if (lookahead == 'w') ADVANCE(695);
      END_STATE();
    case 656:
      if (lookahead == 'w') ADVANCE(15);
      END_STATE();
    case 657:
      if (lookahead == 'w') ADVANCE(185);
      END_STATE();
    case 658:
      if (lookahead == 'w') ADVANCE(69);
      END_STATE();
    case 659:
      if (lookahead == 'w') ADVANCE(200);
      END_STATE();
    case 660:
      if (lookahead == 'w') ADVANCE(304);
      END_STATE();
    case 661:
      if (lookahead == 'y') ADVANCE(773);
      END_STATE();
    case 662:
      if (lookahead == 'y') ADVANCE(692);
      END_STATE();
    case 663:
      if (lookahead == 'y') ADVANCE(768);
      END_STATE();
    case 664:
      if (lookahead == 'y') ADVANCE(690);
      END_STATE();
    case 665:
      if (lookahead == 'y') ADVANCE(756);
      END_STATE();
    case 666:
      if (lookahead == 'y') ADVANCE(749);
      END_STATE();
    case 667:
      if (lookahead == 'y') ADVANCE(711);
      END_STATE();
    case 668:
      if (lookahead == 'y') ADVANCE(737);
      END_STATE();
    case 669:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 670:
      ACCEPT_TOKEN(anon_sym_fn);
      END_STATE();
    case 671:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 672:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 673:
      ACCEPT_TOKEN(aux_sym_block_token1);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(673);
      END_STATE();
    case 674:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 675:
      ACCEPT_TOKEN(anon_sym_ATs);
      END_STATE();
    case 676:
      ACCEPT_TOKEN(anon_sym_ATa);
      END_STATE();
    case 677:
      ACCEPT_TOKEN(anon_sym_ATp);
      END_STATE();
    case 678:
      ACCEPT_TOKEN(anon_sym_ATr);
      END_STATE();
    case 679:
      ACCEPT_TOKEN(anon_sym_ATe);
      END_STATE();
    case 680:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 681:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(765);
      END_STATE();
    case 682:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 683:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 684:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 685:
      ACCEPT_TOKEN(anon_sym_levels);
      END_STATE();
    case 686:
      ACCEPT_TOKEN(anon_sym_points);
      END_STATE();
    case 687:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 688:
      ACCEPT_TOKEN(anon_sym_xpadd);
      END_STATE();
    case 689:
      ACCEPT_TOKEN(anon_sym_xpset);
      END_STATE();
    case 690:
      ACCEPT_TOKEN(anon_sym_xpquery);
      END_STATE();
    case 691:
      ACCEPT_TOKEN(aux_sym_quoted_string_token1);
      END_STATE();
    case 692:
      ACCEPT_TOKEN(anon_sym_say);
      END_STATE();
    case 693:
      ACCEPT_TOKEN(anon_sym_clear);
      END_STATE();
    case 694:
      ACCEPT_TOKEN(anon_sym_eclear);
      END_STATE();
    case 695:
      ACCEPT_TOKEN(anon_sym_tellraw);
      END_STATE();
    case 696:
      ACCEPT_TOKEN(anon_sym_effect);
      END_STATE();
    case 697:
      ACCEPT_TOKEN(anon_sym_enchant);
      END_STATE();
    case 698:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONspeed);
      END_STATE();
    case 699:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslowness);
      END_STATE();
    case 700:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhaste);
      END_STATE();
    case 701:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmining_fatigue);
      END_STATE();
    case 702:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONstrength);
      END_STATE();
    case 703:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_health);
      END_STATE();
    case 704:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_damage);
      END_STATE();
    case 705:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONjump_boost);
      END_STATE();
    case 706:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnausea);
      END_STATE();
    case 707:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONregeneration);
      END_STATE();
    case 708:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONresistance);
      END_STATE();
    case 709:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_resistance);
      END_STATE();
    case 710:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwater_breathing);
      END_STATE();
    case 711:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinvisibility);
      END_STATE();
    case 712:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblindness);
      END_STATE();
    case 713:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnight_vision);
      END_STATE();
    case 714:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhunger);
      END_STATE();
    case 715:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONweakness);
      END_STATE();
    case 716:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpoison);
      END_STATE();
    case 717:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwither);
      END_STATE();
    case 718:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhealth_boost);
      END_STATE();
    case 719:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONabsorption);
      END_STATE();
    case 720:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsaturation);
      END_STATE();
    case 721:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONglowing);
      END_STATE();
    case 722:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlevitation);
      END_STATE();
    case 723:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      END_STATE();
    case 724:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      if (lookahead == '_') ADVANCE(486);
      END_STATE();
    case 725:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunluck);
      END_STATE();
    case 726:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslow_falling);
      END_STATE();
    case 727:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONconduit_power);
      END_STATE();
    case 728:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdolphins_grace);
      END_STATE();
    case 729:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbad_omen);
      END_STATE();
    case 730:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhero_of_the_village);
      END_STATE();
    case 731:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONprotection);
      END_STATE();
    case 732:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_protection);
      END_STATE();
    case 733:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfeather_falling);
      END_STATE();
    case 734:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblast_protection);
      END_STATE();
    case 735:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONprojectile_protection);
      END_STATE();
    case 736:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONrespiration);
      END_STATE();
    case 737:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONaqua_affinity);
      END_STATE();
    case 738:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONthorns);
      END_STATE();
    case 739:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdepth_strider);
      END_STATE();
    case 740:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfrost_walker);
      END_STATE();
    case 741:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbinding_curse);
      END_STATE();
    case 742:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsharpness);
      END_STATE();
    case 743:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsmite);
      END_STATE();
    case 744:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbane_of_arthropods);
      END_STATE();
    case 745:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONknockback);
      END_STATE();
    case 746:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_aspect);
      END_STATE();
    case 747:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlooting);
      END_STATE();
    case 748:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsweeping);
      END_STATE();
    case 749:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONefficiency);
      END_STATE();
    case 750:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsilk_touch);
      END_STATE();
    case 751:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunbreaking);
      END_STATE();
    case 752:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfortune);
      END_STATE();
    case 753:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpower);
      END_STATE();
    case 754:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpunch);
      END_STATE();
    case 755:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONflame);
      END_STATE();
    case 756:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinfinity);
      END_STATE();
    case 757:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck_of_the_sea);
      END_STATE();
    case 758:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlure);
      END_STATE();
    case 759:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmending);
      END_STATE();
    case 760:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONvanishing_curse);
      END_STATE();
    case 761:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsoul_speed);
      END_STATE();
    case 762:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONswift_sneak);
      END_STATE();
    case 763:
      ACCEPT_TOKEN(sym_text);
      if (lookahead == '[') ADVANCE(681);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(763);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(765);
      END_STATE();
    case 764:
      ACCEPT_TOKEN(sym_text);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(764);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(765);
      END_STATE();
    case 765:
      ACCEPT_TOKEN(sym_text);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(765);
      END_STATE();
    case 766:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 767:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 768:
      ACCEPT_TOKEN(anon_sym_query);
      END_STATE();
    case 769:
      ACCEPT_TOKEN(anon_sym_gms);
      if (lookahead == 'p') ADVANCE(771);
      END_STATE();
    case 770:
      ACCEPT_TOKEN(anon_sym_gma);
      END_STATE();
    case 771:
      ACCEPT_TOKEN(anon_sym_gmsp);
      END_STATE();
    case 772:
      ACCEPT_TOKEN(anon_sym_gmc);
      END_STATE();
    case 773:
      ACCEPT_TOKEN(anon_sym_day);
      END_STATE();
    case 774:
      ACCEPT_TOKEN(anon_sym_night);
      END_STATE();
    case 775:
      ACCEPT_TOKEN(anon_sym_noon);
      END_STATE();
    case 776:
      ACCEPT_TOKEN(anon_sym_midnight);
      END_STATE();
    case 777:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(777);
      END_STATE();
    case 778:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(778);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 0},
  [3] = {.lex_state = 0},
  [4] = {.lex_state = 0},
  [5] = {.lex_state = 0},
  [6] = {.lex_state = 0},
  [7] = {.lex_state = 2},
  [8] = {.lex_state = 3},
  [9] = {.lex_state = 109},
  [10] = {.lex_state = 109},
  [11] = {.lex_state = 109},
  [12] = {.lex_state = 0},
  [13] = {.lex_state = 109},
  [14] = {.lex_state = 109},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 0},
  [22] = {.lex_state = 0},
  [23] = {.lex_state = 0},
  [24] = {.lex_state = 0},
  [25] = {.lex_state = 0},
  [26] = {.lex_state = 0},
  [27] = {.lex_state = 0},
  [28] = {.lex_state = 1},
  [29] = {.lex_state = 0},
  [30] = {.lex_state = 0},
  [31] = {.lex_state = 1},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 1},
  [35] = {.lex_state = 1},
  [36] = {.lex_state = 1},
  [37] = {.lex_state = 1},
  [38] = {.lex_state = 1},
  [39] = {.lex_state = 763},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 1},
  [44] = {.lex_state = 1},
  [45] = {.lex_state = 1},
  [46] = {.lex_state = 1},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 0},
  [49] = {.lex_state = 1},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 0},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 763},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 0},
  [58] = {.lex_state = 0},
  [59] = {.lex_state = 0},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 764},
  [63] = {.lex_state = 1},
  [64] = {.lex_state = 0},
  [65] = {.lex_state = 0},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 0},
  [69] = {.lex_state = 0},
  [70] = {.lex_state = 1},
  [71] = {.lex_state = 0},
  [72] = {.lex_state = 0},
  [73] = {.lex_state = 0},
  [74] = {.lex_state = 0},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 0},
  [79] = {.lex_state = 0},
  [80] = {.lex_state = 0},
  [81] = {.lex_state = 0},
  [82] = {.lex_state = 0},
  [83] = {.lex_state = 764},
  [84] = {.lex_state = 0},
  [85] = {.lex_state = 0},
  [86] = {.lex_state = 764},
  [87] = {.lex_state = 764},
  [88] = {.lex_state = 764},
  [89] = {.lex_state = 0},
  [90] = {.lex_state = 0},
  [91] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [anon_sym_fn] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_ATs] = ACTIONS(1),
    [anon_sym_ATa] = ACTIONS(1),
    [anon_sym_ATp] = ACTIONS(1),
    [anon_sym_ATr] = ACTIONS(1),
    [anon_sym_ATe] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_levels] = ACTIONS(1),
    [anon_sym_points] = ACTIONS(1),
    [anon_sym_DOT_DOT] = ACTIONS(1),
    [anon_sym_xpadd] = ACTIONS(1),
    [anon_sym_xpset] = ACTIONS(1),
    [anon_sym_xpquery] = ACTIONS(1),
    [aux_sym_quoted_string_token1] = ACTIONS(1),
    [anon_sym_say] = ACTIONS(1),
    [anon_sym_clear] = ACTIONS(1),
    [anon_sym_eclear] = ACTIONS(1),
    [anon_sym_tellraw] = ACTIONS(1),
    [anon_sym_effect] = ACTIONS(1),
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
    [anon_sym_minecraft_COLONprotection] = ACTIONS(1),
    [anon_sym_minecraft_COLONfire_protection] = ACTIONS(1),
    [anon_sym_minecraft_COLONfeather_falling] = ACTIONS(1),
    [anon_sym_minecraft_COLONblast_protection] = ACTIONS(1),
    [anon_sym_minecraft_COLONprojectile_protection] = ACTIONS(1),
    [anon_sym_minecraft_COLONrespiration] = ACTIONS(1),
    [anon_sym_minecraft_COLONaqua_affinity] = ACTIONS(1),
    [anon_sym_minecraft_COLONthorns] = ACTIONS(1),
    [anon_sym_minecraft_COLONdepth_strider] = ACTIONS(1),
    [anon_sym_minecraft_COLONfrost_walker] = ACTIONS(1),
    [anon_sym_minecraft_COLONbinding_curse] = ACTIONS(1),
    [anon_sym_minecraft_COLONsharpness] = ACTIONS(1),
    [anon_sym_minecraft_COLONsmite] = ACTIONS(1),
    [anon_sym_minecraft_COLONbane_of_arthropods] = ACTIONS(1),
    [anon_sym_minecraft_COLONknockback] = ACTIONS(1),
    [anon_sym_minecraft_COLONfire_aspect] = ACTIONS(1),
    [anon_sym_minecraft_COLONlooting] = ACTIONS(1),
    [anon_sym_minecraft_COLONsweeping] = ACTIONS(1),
    [anon_sym_minecraft_COLONefficiency] = ACTIONS(1),
    [anon_sym_minecraft_COLONsilk_touch] = ACTIONS(1),
    [anon_sym_minecraft_COLONunbreaking] = ACTIONS(1),
    [anon_sym_minecraft_COLONfortune] = ACTIONS(1),
    [anon_sym_minecraft_COLONpower] = ACTIONS(1),
    [anon_sym_minecraft_COLONpunch] = ACTIONS(1),
    [anon_sym_minecraft_COLONflame] = ACTIONS(1),
    [anon_sym_minecraft_COLONinfinity] = ACTIONS(1),
    [anon_sym_minecraft_COLONluck_of_the_sea] = ACTIONS(1),
    [anon_sym_minecraft_COLONlure] = ACTIONS(1),
    [anon_sym_minecraft_COLONmending] = ACTIONS(1),
    [anon_sym_minecraft_COLONvanishing_curse] = ACTIONS(1),
    [anon_sym_minecraft_COLONsoul_speed] = ACTIONS(1),
    [anon_sym_minecraft_COLONswift_sneak] = ACTIONS(1),
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
    [sym_source_file] = STATE(90),
    [sym__definition] = STATE(29),
    [sym_function_definition] = STATE(29),
    [aux_sym_source_file_repeat1] = STATE(29),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_fn] = ACTIONS(5),
  },
  [2] = {
    [sym_selector_arguments] = STATE(5),
    [anon_sym_SEMI] = ACTIONS(7),
    [anon_sym_LBRACK] = ACTIONS(9),
    [anon_sym_levels] = ACTIONS(7),
    [anon_sym_points] = ACTIONS(7),
    [aux_sym_quoted_string_token1] = ACTIONS(7),
    [anon_sym_minecraft_COLONspeed] = ACTIONS(7),
    [anon_sym_minecraft_COLONslowness] = ACTIONS(7),
    [anon_sym_minecraft_COLONhaste] = ACTIONS(7),
    [anon_sym_minecraft_COLONmining_fatigue] = ACTIONS(7),
    [anon_sym_minecraft_COLONstrength] = ACTIONS(7),
    [anon_sym_minecraft_COLONinstant_health] = ACTIONS(7),
    [anon_sym_minecraft_COLONinstant_damage] = ACTIONS(7),
    [anon_sym_minecraft_COLONjump_boost] = ACTIONS(7),
    [anon_sym_minecraft_COLONnausea] = ACTIONS(7),
    [anon_sym_minecraft_COLONregeneration] = ACTIONS(7),
    [anon_sym_minecraft_COLONresistance] = ACTIONS(7),
    [anon_sym_minecraft_COLONfire_resistance] = ACTIONS(7),
    [anon_sym_minecraft_COLONwater_breathing] = ACTIONS(7),
    [anon_sym_minecraft_COLONinvisibility] = ACTIONS(7),
    [anon_sym_minecraft_COLONblindness] = ACTIONS(7),
    [anon_sym_minecraft_COLONnight_vision] = ACTIONS(7),
    [anon_sym_minecraft_COLONhunger] = ACTIONS(7),
    [anon_sym_minecraft_COLONweakness] = ACTIONS(7),
    [anon_sym_minecraft_COLONpoison] = ACTIONS(7),
    [anon_sym_minecraft_COLONwither] = ACTIONS(7),
    [anon_sym_minecraft_COLONhealth_boost] = ACTIONS(7),
    [anon_sym_minecraft_COLONabsorption] = ACTIONS(7),
    [anon_sym_minecraft_COLONsaturation] = ACTIONS(7),
    [anon_sym_minecraft_COLONglowing] = ACTIONS(7),
    [anon_sym_minecraft_COLONlevitation] = ACTIONS(7),
    [anon_sym_minecraft_COLONluck] = ACTIONS(11),
    [anon_sym_minecraft_COLONunluck] = ACTIONS(7),
    [anon_sym_minecraft_COLONslow_falling] = ACTIONS(7),
    [anon_sym_minecraft_COLONconduit_power] = ACTIONS(7),
    [anon_sym_minecraft_COLONdolphins_grace] = ACTIONS(7),
    [anon_sym_minecraft_COLONbad_omen] = ACTIONS(7),
    [anon_sym_minecraft_COLONhero_of_the_village] = ACTIONS(7),
    [anon_sym_minecraft_COLONprotection] = ACTIONS(7),
    [anon_sym_minecraft_COLONfire_protection] = ACTIONS(7),
    [anon_sym_minecraft_COLONfeather_falling] = ACTIONS(7),
    [anon_sym_minecraft_COLONblast_protection] = ACTIONS(7),
    [anon_sym_minecraft_COLONprojectile_protection] = ACTIONS(7),
    [anon_sym_minecraft_COLONrespiration] = ACTIONS(7),
    [anon_sym_minecraft_COLONaqua_affinity] = ACTIONS(7),
    [anon_sym_minecraft_COLONthorns] = ACTIONS(7),
    [anon_sym_minecraft_COLONdepth_strider] = ACTIONS(7),
    [anon_sym_minecraft_COLONfrost_walker] = ACTIONS(7),
    [anon_sym_minecraft_COLONbinding_curse] = ACTIONS(7),
    [anon_sym_minecraft_COLONsharpness] = ACTIONS(7),
    [anon_sym_minecraft_COLONsmite] = ACTIONS(7),
    [anon_sym_minecraft_COLONbane_of_arthropods] = ACTIONS(7),
    [anon_sym_minecraft_COLONknockback] = ACTIONS(7),
    [anon_sym_minecraft_COLONfire_aspect] = ACTIONS(7),
    [anon_sym_minecraft_COLONlooting] = ACTIONS(7),
    [anon_sym_minecraft_COLONsweeping] = ACTIONS(7),
    [anon_sym_minecraft_COLONefficiency] = ACTIONS(7),
    [anon_sym_minecraft_COLONsilk_touch] = ACTIONS(7),
    [anon_sym_minecraft_COLONunbreaking] = ACTIONS(7),
    [anon_sym_minecraft_COLONfortune] = ACTIONS(7),
    [anon_sym_minecraft_COLONpower] = ACTIONS(7),
    [anon_sym_minecraft_COLONpunch] = ACTIONS(7),
    [anon_sym_minecraft_COLONflame] = ACTIONS(7),
    [anon_sym_minecraft_COLONinfinity] = ACTIONS(7),
    [anon_sym_minecraft_COLONluck_of_the_sea] = ACTIONS(7),
    [anon_sym_minecraft_COLONlure] = ACTIONS(7),
    [anon_sym_minecraft_COLONmending] = ACTIONS(7),
    [anon_sym_minecraft_COLONvanishing_curse] = ACTIONS(7),
    [anon_sym_minecraft_COLONsoul_speed] = ACTIONS(7),
    [anon_sym_minecraft_COLONswift_sneak] = ACTIONS(7),
  },
  [3] = {
    [anon_sym_SEMI] = ACTIONS(13),
    [anon_sym_LBRACK] = ACTIONS(13),
    [anon_sym_levels] = ACTIONS(13),
    [anon_sym_points] = ACTIONS(13),
    [aux_sym_quoted_string_token1] = ACTIONS(13),
    [anon_sym_minecraft_COLONspeed] = ACTIONS(13),
    [anon_sym_minecraft_COLONslowness] = ACTIONS(13),
    [anon_sym_minecraft_COLONhaste] = ACTIONS(13),
    [anon_sym_minecraft_COLONmining_fatigue] = ACTIONS(13),
    [anon_sym_minecraft_COLONstrength] = ACTIONS(13),
    [anon_sym_minecraft_COLONinstant_health] = ACTIONS(13),
    [anon_sym_minecraft_COLONinstant_damage] = ACTIONS(13),
    [anon_sym_minecraft_COLONjump_boost] = ACTIONS(13),
    [anon_sym_minecraft_COLONnausea] = ACTIONS(13),
    [anon_sym_minecraft_COLONregeneration] = ACTIONS(13),
    [anon_sym_minecraft_COLONresistance] = ACTIONS(13),
    [anon_sym_minecraft_COLONfire_resistance] = ACTIONS(13),
    [anon_sym_minecraft_COLONwater_breathing] = ACTIONS(13),
    [anon_sym_minecraft_COLONinvisibility] = ACTIONS(13),
    [anon_sym_minecraft_COLONblindness] = ACTIONS(13),
    [anon_sym_minecraft_COLONnight_vision] = ACTIONS(13),
    [anon_sym_minecraft_COLONhunger] = ACTIONS(13),
    [anon_sym_minecraft_COLONweakness] = ACTIONS(13),
    [anon_sym_minecraft_COLONpoison] = ACTIONS(13),
    [anon_sym_minecraft_COLONwither] = ACTIONS(13),
    [anon_sym_minecraft_COLONhealth_boost] = ACTIONS(13),
    [anon_sym_minecraft_COLONabsorption] = ACTIONS(13),
    [anon_sym_minecraft_COLONsaturation] = ACTIONS(13),
    [anon_sym_minecraft_COLONglowing] = ACTIONS(13),
    [anon_sym_minecraft_COLONlevitation] = ACTIONS(13),
    [anon_sym_minecraft_COLONluck] = ACTIONS(15),
    [anon_sym_minecraft_COLONunluck] = ACTIONS(13),
    [anon_sym_minecraft_COLONslow_falling] = ACTIONS(13),
    [anon_sym_minecraft_COLONconduit_power] = ACTIONS(13),
    [anon_sym_minecraft_COLONdolphins_grace] = ACTIONS(13),
    [anon_sym_minecraft_COLONbad_omen] = ACTIONS(13),
    [anon_sym_minecraft_COLONhero_of_the_village] = ACTIONS(13),
    [anon_sym_minecraft_COLONprotection] = ACTIONS(13),
    [anon_sym_minecraft_COLONfire_protection] = ACTIONS(13),
    [anon_sym_minecraft_COLONfeather_falling] = ACTIONS(13),
    [anon_sym_minecraft_COLONblast_protection] = ACTIONS(13),
    [anon_sym_minecraft_COLONprojectile_protection] = ACTIONS(13),
    [anon_sym_minecraft_COLONrespiration] = ACTIONS(13),
    [anon_sym_minecraft_COLONaqua_affinity] = ACTIONS(13),
    [anon_sym_minecraft_COLONthorns] = ACTIONS(13),
    [anon_sym_minecraft_COLONdepth_strider] = ACTIONS(13),
    [anon_sym_minecraft_COLONfrost_walker] = ACTIONS(13),
    [anon_sym_minecraft_COLONbinding_curse] = ACTIONS(13),
    [anon_sym_minecraft_COLONsharpness] = ACTIONS(13),
    [anon_sym_minecraft_COLONsmite] = ACTIONS(13),
    [anon_sym_minecraft_COLONbane_of_arthropods] = ACTIONS(13),
    [anon_sym_minecraft_COLONknockback] = ACTIONS(13),
    [anon_sym_minecraft_COLONfire_aspect] = ACTIONS(13),
    [anon_sym_minecraft_COLONlooting] = ACTIONS(13),
    [anon_sym_minecraft_COLONsweeping] = ACTIONS(13),
    [anon_sym_minecraft_COLONefficiency] = ACTIONS(13),
    [anon_sym_minecraft_COLONsilk_touch] = ACTIONS(13),
    [anon_sym_minecraft_COLONunbreaking] = ACTIONS(13),
    [anon_sym_minecraft_COLONfortune] = ACTIONS(13),
    [anon_sym_minecraft_COLONpower] = ACTIONS(13),
    [anon_sym_minecraft_COLONpunch] = ACTIONS(13),
    [anon_sym_minecraft_COLONflame] = ACTIONS(13),
    [anon_sym_minecraft_COLONinfinity] = ACTIONS(13),
    [anon_sym_minecraft_COLONluck_of_the_sea] = ACTIONS(13),
    [anon_sym_minecraft_COLONlure] = ACTIONS(13),
    [anon_sym_minecraft_COLONmending] = ACTIONS(13),
    [anon_sym_minecraft_COLONvanishing_curse] = ACTIONS(13),
    [anon_sym_minecraft_COLONsoul_speed] = ACTIONS(13),
    [anon_sym_minecraft_COLONswift_sneak] = ACTIONS(13),
  },
  [4] = {
    [anon_sym_SEMI] = ACTIONS(17),
    [anon_sym_levels] = ACTIONS(17),
    [anon_sym_points] = ACTIONS(17),
    [aux_sym_quoted_string_token1] = ACTIONS(17),
    [anon_sym_minecraft_COLONspeed] = ACTIONS(17),
    [anon_sym_minecraft_COLONslowness] = ACTIONS(17),
    [anon_sym_minecraft_COLONhaste] = ACTIONS(17),
    [anon_sym_minecraft_COLONmining_fatigue] = ACTIONS(17),
    [anon_sym_minecraft_COLONstrength] = ACTIONS(17),
    [anon_sym_minecraft_COLONinstant_health] = ACTIONS(17),
    [anon_sym_minecraft_COLONinstant_damage] = ACTIONS(17),
    [anon_sym_minecraft_COLONjump_boost] = ACTIONS(17),
    [anon_sym_minecraft_COLONnausea] = ACTIONS(17),
    [anon_sym_minecraft_COLONregeneration] = ACTIONS(17),
    [anon_sym_minecraft_COLONresistance] = ACTIONS(17),
    [anon_sym_minecraft_COLONfire_resistance] = ACTIONS(17),
    [anon_sym_minecraft_COLONwater_breathing] = ACTIONS(17),
    [anon_sym_minecraft_COLONinvisibility] = ACTIONS(17),
    [anon_sym_minecraft_COLONblindness] = ACTIONS(17),
    [anon_sym_minecraft_COLONnight_vision] = ACTIONS(17),
    [anon_sym_minecraft_COLONhunger] = ACTIONS(17),
    [anon_sym_minecraft_COLONweakness] = ACTIONS(17),
    [anon_sym_minecraft_COLONpoison] = ACTIONS(17),
    [anon_sym_minecraft_COLONwither] = ACTIONS(17),
    [anon_sym_minecraft_COLONhealth_boost] = ACTIONS(17),
    [anon_sym_minecraft_COLONabsorption] = ACTIONS(17),
    [anon_sym_minecraft_COLONsaturation] = ACTIONS(17),
    [anon_sym_minecraft_COLONglowing] = ACTIONS(17),
    [anon_sym_minecraft_COLONlevitation] = ACTIONS(17),
    [anon_sym_minecraft_COLONluck] = ACTIONS(19),
    [anon_sym_minecraft_COLONunluck] = ACTIONS(17),
    [anon_sym_minecraft_COLONslow_falling] = ACTIONS(17),
    [anon_sym_minecraft_COLONconduit_power] = ACTIONS(17),
    [anon_sym_minecraft_COLONdolphins_grace] = ACTIONS(17),
    [anon_sym_minecraft_COLONbad_omen] = ACTIONS(17),
    [anon_sym_minecraft_COLONhero_of_the_village] = ACTIONS(17),
    [anon_sym_minecraft_COLONprotection] = ACTIONS(17),
    [anon_sym_minecraft_COLONfire_protection] = ACTIONS(17),
    [anon_sym_minecraft_COLONfeather_falling] = ACTIONS(17),
    [anon_sym_minecraft_COLONblast_protection] = ACTIONS(17),
    [anon_sym_minecraft_COLONprojectile_protection] = ACTIONS(17),
    [anon_sym_minecraft_COLONrespiration] = ACTIONS(17),
    [anon_sym_minecraft_COLONaqua_affinity] = ACTIONS(17),
    [anon_sym_minecraft_COLONthorns] = ACTIONS(17),
    [anon_sym_minecraft_COLONdepth_strider] = ACTIONS(17),
    [anon_sym_minecraft_COLONfrost_walker] = ACTIONS(17),
    [anon_sym_minecraft_COLONbinding_curse] = ACTIONS(17),
    [anon_sym_minecraft_COLONsharpness] = ACTIONS(17),
    [anon_sym_minecraft_COLONsmite] = ACTIONS(17),
    [anon_sym_minecraft_COLONbane_of_arthropods] = ACTIONS(17),
    [anon_sym_minecraft_COLONknockback] = ACTIONS(17),
    [anon_sym_minecraft_COLONfire_aspect] = ACTIONS(17),
    [anon_sym_minecraft_COLONlooting] = ACTIONS(17),
    [anon_sym_minecraft_COLONsweeping] = ACTIONS(17),
    [anon_sym_minecraft_COLONefficiency] = ACTIONS(17),
    [anon_sym_minecraft_COLONsilk_touch] = ACTIONS(17),
    [anon_sym_minecraft_COLONunbreaking] = ACTIONS(17),
    [anon_sym_minecraft_COLONfortune] = ACTIONS(17),
    [anon_sym_minecraft_COLONpower] = ACTIONS(17),
    [anon_sym_minecraft_COLONpunch] = ACTIONS(17),
    [anon_sym_minecraft_COLONflame] = ACTIONS(17),
    [anon_sym_minecraft_COLONinfinity] = ACTIONS(17),
    [anon_sym_minecraft_COLONluck_of_the_sea] = ACTIONS(17),
    [anon_sym_minecraft_COLONlure] = ACTIONS(17),
    [anon_sym_minecraft_COLONmending] = ACTIONS(17),
    [anon_sym_minecraft_COLONvanishing_curse] = ACTIONS(17),
    [anon_sym_minecraft_COLONsoul_speed] = ACTIONS(17),
    [anon_sym_minecraft_COLONswift_sneak] = ACTIONS(17),
  },
  [5] = {
    [anon_sym_SEMI] = ACTIONS(21),
    [anon_sym_levels] = ACTIONS(21),
    [anon_sym_points] = ACTIONS(21),
    [aux_sym_quoted_string_token1] = ACTIONS(21),
    [anon_sym_minecraft_COLONspeed] = ACTIONS(21),
    [anon_sym_minecraft_COLONslowness] = ACTIONS(21),
    [anon_sym_minecraft_COLONhaste] = ACTIONS(21),
    [anon_sym_minecraft_COLONmining_fatigue] = ACTIONS(21),
    [anon_sym_minecraft_COLONstrength] = ACTIONS(21),
    [anon_sym_minecraft_COLONinstant_health] = ACTIONS(21),
    [anon_sym_minecraft_COLONinstant_damage] = ACTIONS(21),
    [anon_sym_minecraft_COLONjump_boost] = ACTIONS(21),
    [anon_sym_minecraft_COLONnausea] = ACTIONS(21),
    [anon_sym_minecraft_COLONregeneration] = ACTIONS(21),
    [anon_sym_minecraft_COLONresistance] = ACTIONS(21),
    [anon_sym_minecraft_COLONfire_resistance] = ACTIONS(21),
    [anon_sym_minecraft_COLONwater_breathing] = ACTIONS(21),
    [anon_sym_minecraft_COLONinvisibility] = ACTIONS(21),
    [anon_sym_minecraft_COLONblindness] = ACTIONS(21),
    [anon_sym_minecraft_COLONnight_vision] = ACTIONS(21),
    [anon_sym_minecraft_COLONhunger] = ACTIONS(21),
    [anon_sym_minecraft_COLONweakness] = ACTIONS(21),
    [anon_sym_minecraft_COLONpoison] = ACTIONS(21),
    [anon_sym_minecraft_COLONwither] = ACTIONS(21),
    [anon_sym_minecraft_COLONhealth_boost] = ACTIONS(21),
    [anon_sym_minecraft_COLONabsorption] = ACTIONS(21),
    [anon_sym_minecraft_COLONsaturation] = ACTIONS(21),
    [anon_sym_minecraft_COLONglowing] = ACTIONS(21),
    [anon_sym_minecraft_COLONlevitation] = ACTIONS(21),
    [anon_sym_minecraft_COLONluck] = ACTIONS(23),
    [anon_sym_minecraft_COLONunluck] = ACTIONS(21),
    [anon_sym_minecraft_COLONslow_falling] = ACTIONS(21),
    [anon_sym_minecraft_COLONconduit_power] = ACTIONS(21),
    [anon_sym_minecraft_COLONdolphins_grace] = ACTIONS(21),
    [anon_sym_minecraft_COLONbad_omen] = ACTIONS(21),
    [anon_sym_minecraft_COLONhero_of_the_village] = ACTIONS(21),
    [anon_sym_minecraft_COLONprotection] = ACTIONS(21),
    [anon_sym_minecraft_COLONfire_protection] = ACTIONS(21),
    [anon_sym_minecraft_COLONfeather_falling] = ACTIONS(21),
    [anon_sym_minecraft_COLONblast_protection] = ACTIONS(21),
    [anon_sym_minecraft_COLONprojectile_protection] = ACTIONS(21),
    [anon_sym_minecraft_COLONrespiration] = ACTIONS(21),
    [anon_sym_minecraft_COLONaqua_affinity] = ACTIONS(21),
    [anon_sym_minecraft_COLONthorns] = ACTIONS(21),
    [anon_sym_minecraft_COLONdepth_strider] = ACTIONS(21),
    [anon_sym_minecraft_COLONfrost_walker] = ACTIONS(21),
    [anon_sym_minecraft_COLONbinding_curse] = ACTIONS(21),
    [anon_sym_minecraft_COLONsharpness] = ACTIONS(21),
    [anon_sym_minecraft_COLONsmite] = ACTIONS(21),
    [anon_sym_minecraft_COLONbane_of_arthropods] = ACTIONS(21),
    [anon_sym_minecraft_COLONknockback] = ACTIONS(21),
    [anon_sym_minecraft_COLONfire_aspect] = ACTIONS(21),
    [anon_sym_minecraft_COLONlooting] = ACTIONS(21),
    [anon_sym_minecraft_COLONsweeping] = ACTIONS(21),
    [anon_sym_minecraft_COLONefficiency] = ACTIONS(21),
    [anon_sym_minecraft_COLONsilk_touch] = ACTIONS(21),
    [anon_sym_minecraft_COLONunbreaking] = ACTIONS(21),
    [anon_sym_minecraft_COLONfortune] = ACTIONS(21),
    [anon_sym_minecraft_COLONpower] = ACTIONS(21),
    [anon_sym_minecraft_COLONpunch] = ACTIONS(21),
    [anon_sym_minecraft_COLONflame] = ACTIONS(21),
    [anon_sym_minecraft_COLONinfinity] = ACTIONS(21),
    [anon_sym_minecraft_COLONluck_of_the_sea] = ACTIONS(21),
    [anon_sym_minecraft_COLONlure] = ACTIONS(21),
    [anon_sym_minecraft_COLONmending] = ACTIONS(21),
    [anon_sym_minecraft_COLONvanishing_curse] = ACTIONS(21),
    [anon_sym_minecraft_COLONsoul_speed] = ACTIONS(21),
    [anon_sym_minecraft_COLONswift_sneak] = ACTIONS(21),
  },
  [6] = {
    [anon_sym_SEMI] = ACTIONS(25),
    [anon_sym_levels] = ACTIONS(25),
    [anon_sym_points] = ACTIONS(25),
    [aux_sym_quoted_string_token1] = ACTIONS(25),
    [anon_sym_minecraft_COLONspeed] = ACTIONS(25),
    [anon_sym_minecraft_COLONslowness] = ACTIONS(25),
    [anon_sym_minecraft_COLONhaste] = ACTIONS(25),
    [anon_sym_minecraft_COLONmining_fatigue] = ACTIONS(25),
    [anon_sym_minecraft_COLONstrength] = ACTIONS(25),
    [anon_sym_minecraft_COLONinstant_health] = ACTIONS(25),
    [anon_sym_minecraft_COLONinstant_damage] = ACTIONS(25),
    [anon_sym_minecraft_COLONjump_boost] = ACTIONS(25),
    [anon_sym_minecraft_COLONnausea] = ACTIONS(25),
    [anon_sym_minecraft_COLONregeneration] = ACTIONS(25),
    [anon_sym_minecraft_COLONresistance] = ACTIONS(25),
    [anon_sym_minecraft_COLONfire_resistance] = ACTIONS(25),
    [anon_sym_minecraft_COLONwater_breathing] = ACTIONS(25),
    [anon_sym_minecraft_COLONinvisibility] = ACTIONS(25),
    [anon_sym_minecraft_COLONblindness] = ACTIONS(25),
    [anon_sym_minecraft_COLONnight_vision] = ACTIONS(25),
    [anon_sym_minecraft_COLONhunger] = ACTIONS(25),
    [anon_sym_minecraft_COLONweakness] = ACTIONS(25),
    [anon_sym_minecraft_COLONpoison] = ACTIONS(25),
    [anon_sym_minecraft_COLONwither] = ACTIONS(25),
    [anon_sym_minecraft_COLONhealth_boost] = ACTIONS(25),
    [anon_sym_minecraft_COLONabsorption] = ACTIONS(25),
    [anon_sym_minecraft_COLONsaturation] = ACTIONS(25),
    [anon_sym_minecraft_COLONglowing] = ACTIONS(25),
    [anon_sym_minecraft_COLONlevitation] = ACTIONS(25),
    [anon_sym_minecraft_COLONluck] = ACTIONS(27),
    [anon_sym_minecraft_COLONunluck] = ACTIONS(25),
    [anon_sym_minecraft_COLONslow_falling] = ACTIONS(25),
    [anon_sym_minecraft_COLONconduit_power] = ACTIONS(25),
    [anon_sym_minecraft_COLONdolphins_grace] = ACTIONS(25),
    [anon_sym_minecraft_COLONbad_omen] = ACTIONS(25),
    [anon_sym_minecraft_COLONhero_of_the_village] = ACTIONS(25),
    [anon_sym_minecraft_COLONprotection] = ACTIONS(25),
    [anon_sym_minecraft_COLONfire_protection] = ACTIONS(25),
    [anon_sym_minecraft_COLONfeather_falling] = ACTIONS(25),
    [anon_sym_minecraft_COLONblast_protection] = ACTIONS(25),
    [anon_sym_minecraft_COLONprojectile_protection] = ACTIONS(25),
    [anon_sym_minecraft_COLONrespiration] = ACTIONS(25),
    [anon_sym_minecraft_COLONaqua_affinity] = ACTIONS(25),
    [anon_sym_minecraft_COLONthorns] = ACTIONS(25),
    [anon_sym_minecraft_COLONdepth_strider] = ACTIONS(25),
    [anon_sym_minecraft_COLONfrost_walker] = ACTIONS(25),
    [anon_sym_minecraft_COLONbinding_curse] = ACTIONS(25),
    [anon_sym_minecraft_COLONsharpness] = ACTIONS(25),
    [anon_sym_minecraft_COLONsmite] = ACTIONS(25),
    [anon_sym_minecraft_COLONbane_of_arthropods] = ACTIONS(25),
    [anon_sym_minecraft_COLONknockback] = ACTIONS(25),
    [anon_sym_minecraft_COLONfire_aspect] = ACTIONS(25),
    [anon_sym_minecraft_COLONlooting] = ACTIONS(25),
    [anon_sym_minecraft_COLONsweeping] = ACTIONS(25),
    [anon_sym_minecraft_COLONefficiency] = ACTIONS(25),
    [anon_sym_minecraft_COLONsilk_touch] = ACTIONS(25),
    [anon_sym_minecraft_COLONunbreaking] = ACTIONS(25),
    [anon_sym_minecraft_COLONfortune] = ACTIONS(25),
    [anon_sym_minecraft_COLONpower] = ACTIONS(25),
    [anon_sym_minecraft_COLONpunch] = ACTIONS(25),
    [anon_sym_minecraft_COLONflame] = ACTIONS(25),
    [anon_sym_minecraft_COLONinfinity] = ACTIONS(25),
    [anon_sym_minecraft_COLONluck_of_the_sea] = ACTIONS(25),
    [anon_sym_minecraft_COLONlure] = ACTIONS(25),
    [anon_sym_minecraft_COLONmending] = ACTIONS(25),
    [anon_sym_minecraft_COLONvanishing_curse] = ACTIONS(25),
    [anon_sym_minecraft_COLONsoul_speed] = ACTIONS(25),
    [anon_sym_minecraft_COLONswift_sneak] = ACTIONS(25),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 3,
    ACTIONS(29), 1,
      aux_sym_quoted_string_token1,
    STATE(59), 2,
      sym_vanilla_enchant,
      sym_custom_enchant,
    ACTIONS(31), 32,
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
  [42] = 3,
    ACTIONS(33), 1,
      aux_sym_quoted_string_token1,
    STATE(56), 2,
      sym_vanilla_effect,
      sym_custom_effect,
    ACTIONS(35), 32,
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
  [84] = 18,
    ACTIONS(37), 1,
      aux_sym_block_token1,
    ACTIONS(39), 1,
      anon_sym_RBRACE,
    ACTIONS(41), 1,
      anon_sym_xpadd,
    ACTIONS(43), 1,
      anon_sym_xpset,
    ACTIONS(45), 1,
      anon_sym_xpquery,
    ACTIONS(47), 1,
      anon_sym_say,
    ACTIONS(49), 1,
      anon_sym_clear,
    ACTIONS(51), 1,
      anon_sym_eclear,
    ACTIONS(53), 1,
      anon_sym_tellraw,
    ACTIONS(55), 1,
      anon_sym_effect,
    ACTIONS(57), 1,
      anon_sym_enchant,
    ACTIONS(59), 1,
      anon_sym_time,
    ACTIONS(61), 1,
      anon_sym_gms,
    ACTIONS(63), 1,
      anon_sym_gma,
    ACTIONS(65), 1,
      anon_sym_gmsp,
    ACTIONS(67), 1,
      anon_sym_gmc,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(68), 15,
      sym__command,
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
  [153] = 18,
    ACTIONS(69), 1,
      aux_sym_block_token1,
    ACTIONS(72), 1,
      anon_sym_RBRACE,
    ACTIONS(74), 1,
      anon_sym_xpadd,
    ACTIONS(77), 1,
      anon_sym_xpset,
    ACTIONS(80), 1,
      anon_sym_xpquery,
    ACTIONS(83), 1,
      anon_sym_say,
    ACTIONS(86), 1,
      anon_sym_clear,
    ACTIONS(89), 1,
      anon_sym_eclear,
    ACTIONS(92), 1,
      anon_sym_tellraw,
    ACTIONS(95), 1,
      anon_sym_effect,
    ACTIONS(98), 1,
      anon_sym_enchant,
    ACTIONS(101), 1,
      anon_sym_time,
    ACTIONS(104), 1,
      anon_sym_gms,
    ACTIONS(107), 1,
      anon_sym_gma,
    ACTIONS(110), 1,
      anon_sym_gmsp,
    ACTIONS(113), 1,
      anon_sym_gmc,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(68), 15,
      sym__command,
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
  [222] = 18,
    ACTIONS(37), 1,
      aux_sym_block_token1,
    ACTIONS(41), 1,
      anon_sym_xpadd,
    ACTIONS(43), 1,
      anon_sym_xpset,
    ACTIONS(45), 1,
      anon_sym_xpquery,
    ACTIONS(47), 1,
      anon_sym_say,
    ACTIONS(49), 1,
      anon_sym_clear,
    ACTIONS(51), 1,
      anon_sym_eclear,
    ACTIONS(53), 1,
      anon_sym_tellraw,
    ACTIONS(55), 1,
      anon_sym_effect,
    ACTIONS(57), 1,
      anon_sym_enchant,
    ACTIONS(59), 1,
      anon_sym_time,
    ACTIONS(61), 1,
      anon_sym_gms,
    ACTIONS(63), 1,
      anon_sym_gma,
    ACTIONS(65), 1,
      anon_sym_gmsp,
    ACTIONS(67), 1,
      anon_sym_gmc,
    ACTIONS(116), 1,
      anon_sym_RBRACE,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(68), 15,
      sym__command,
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
  [291] = 15,
    ACTIONS(61), 1,
      anon_sym_gms,
    ACTIONS(118), 1,
      anon_sym_xpadd,
    ACTIONS(120), 1,
      anon_sym_xpset,
    ACTIONS(122), 1,
      anon_sym_xpquery,
    ACTIONS(124), 1,
      anon_sym_say,
    ACTIONS(126), 1,
      anon_sym_clear,
    ACTIONS(128), 1,
      anon_sym_eclear,
    ACTIONS(130), 1,
      anon_sym_tellraw,
    ACTIONS(132), 1,
      anon_sym_effect,
    ACTIONS(134), 1,
      anon_sym_enchant,
    ACTIONS(136), 1,
      anon_sym_time,
    ACTIONS(138), 1,
      anon_sym_gma,
    ACTIONS(140), 1,
      anon_sym_gmsp,
    ACTIONS(142), 1,
      anon_sym_gmc,
    STATE(81), 14,
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
  [350] = 2,
    ACTIONS(144), 1,
      aux_sym_block_token1,
    ACTIONS(72), 15,
      anon_sym_RBRACE,
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
  [371] = 2,
    ACTIONS(147), 1,
      aux_sym_block_token1,
    ACTIONS(149), 15,
      anon_sym_RBRACE,
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
  [392] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(80), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [406] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(82), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [420] = 3,
    STATE(39), 1,
      sym_selector_type,
    STATE(83), 1,
      sym_target_selector,
    ACTIONS(153), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [434] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(8), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [448] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(7), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [462] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(66), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [476] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(72), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [490] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(73), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [504] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(79), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [518] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(41), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [532] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(42), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [546] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(40), 1,
      sym_target_selector,
    ACTIONS(151), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [560] = 3,
    ACTIONS(157), 1,
      sym_number,
    STATE(61), 1,
      sym_time_unit,
    ACTIONS(155), 4,
      anon_sym_day,
      anon_sym_night,
      anon_sym_noon,
      anon_sym_midnight,
  [573] = 5,
    ACTIONS(159), 1,
      anon_sym_DOT_DOT,
    ACTIONS(161), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(163), 1,
      sym_identifier,
    ACTIONS(165), 1,
      sym_number,
    STATE(45), 2,
      sym_range_value,
      sym_quoted_string,
  [590] = 3,
    ACTIONS(5), 1,
      anon_sym_fn,
    ACTIONS(167), 1,
      ts_builtin_sym_end,
    STATE(30), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [602] = 3,
    ACTIONS(169), 1,
      ts_builtin_sym_end,
    ACTIONS(171), 1,
      anon_sym_fn,
    STATE(30), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [614] = 4,
    ACTIONS(174), 1,
      anon_sym_RBRACK,
    ACTIONS(176), 1,
      sym_identifier,
    STATE(35), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(43), 1,
      sym_selector_argument,
  [627] = 4,
    ACTIONS(178), 1,
      anon_sym_RBRACK,
    ACTIONS(180), 1,
      sym_identifier,
    STATE(32), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(43), 1,
      sym_selector_argument,
  [640] = 2,
    ACTIONS(185), 1,
      anon_sym_DOT_DOT,
    ACTIONS(183), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [649] = 2,
    ACTIONS(189), 1,
      sym_number,
    ACTIONS(187), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [658] = 4,
    ACTIONS(176), 1,
      sym_identifier,
    ACTIONS(191), 1,
      anon_sym_RBRACK,
    STATE(32), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(43), 1,
      sym_selector_argument,
  [671] = 4,
    ACTIONS(176), 1,
      sym_identifier,
    ACTIONS(193), 1,
      anon_sym_RBRACK,
    STATE(37), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(43), 1,
      sym_selector_argument,
  [684] = 4,
    ACTIONS(176), 1,
      sym_identifier,
    ACTIONS(195), 1,
      anon_sym_RBRACK,
    STATE(32), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(43), 1,
      sym_selector_argument,
  [697] = 1,
    ACTIONS(197), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [703] = 3,
    ACTIONS(11), 1,
      sym_text,
    ACTIONS(199), 1,
      anon_sym_LBRACK,
    STATE(86), 1,
      sym_selector_arguments,
  [713] = 2,
    STATE(78), 1,
      sym_xp_type,
    ACTIONS(201), 2,
      anon_sym_levels,
      anon_sym_points,
  [721] = 2,
    STATE(64), 1,
      sym_xp_type,
    ACTIONS(201), 2,
      anon_sym_levels,
      anon_sym_points,
  [729] = 2,
    STATE(65), 1,
      sym_xp_type,
    ACTIONS(201), 2,
      anon_sym_levels,
      anon_sym_points,
  [737] = 2,
    ACTIONS(203), 1,
      anon_sym_COMMA,
    ACTIONS(205), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [745] = 1,
    ACTIONS(207), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [751] = 1,
    ACTIONS(209), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [757] = 1,
    ACTIONS(187), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [763] = 1,
    ACTIONS(211), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [768] = 1,
    ACTIONS(213), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [773] = 1,
    ACTIONS(178), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [778] = 2,
    ACTIONS(215), 1,
      anon_sym_set,
    ACTIONS(217), 1,
      anon_sym_query,
  [785] = 1,
    ACTIONS(219), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [790] = 2,
    ACTIONS(221), 1,
      anon_sym_LBRACE,
    STATE(48), 1,
      sym_block,
  [797] = 1,
    ACTIONS(15), 2,
      anon_sym_LBRACK,
      sym_text,
  [802] = 1,
    ACTIONS(223), 1,
      anon_sym_SEMI,
  [806] = 1,
    ACTIONS(225), 1,
      sym_number,
  [810] = 1,
    ACTIONS(227), 1,
      sym_number,
  [814] = 1,
    ACTIONS(229), 1,
      sym_number,
  [818] = 1,
    ACTIONS(231), 1,
      sym_number,
  [822] = 1,
    ACTIONS(233), 1,
      sym_number,
  [826] = 1,
    ACTIONS(235), 1,
      anon_sym_SEMI,
  [830] = 1,
    ACTIONS(237), 1,
      anon_sym_SEMI,
  [834] = 1,
    ACTIONS(239), 1,
      sym_text,
  [838] = 1,
    ACTIONS(241), 1,
      sym_identifier,
  [842] = 1,
    ACTIONS(243), 1,
      anon_sym_SEMI,
  [846] = 1,
    ACTIONS(245), 1,
      anon_sym_SEMI,
  [850] = 1,
    ACTIONS(247), 1,
      anon_sym_SEMI,
  [854] = 1,
    ACTIONS(249), 1,
      anon_sym_EQ,
  [858] = 1,
    ACTIONS(251), 1,
      anon_sym_SEMI,
  [862] = 1,
    ACTIONS(253), 1,
      sym_number,
  [866] = 1,
    ACTIONS(255), 1,
      sym_identifier,
  [870] = 1,
    ACTIONS(257), 1,
      anon_sym_SEMI,
  [874] = 1,
    ACTIONS(259), 1,
      anon_sym_SEMI,
  [878] = 1,
    ACTIONS(261), 1,
      anon_sym_SEMI,
  [882] = 1,
    ACTIONS(263), 1,
      anon_sym_SEMI,
  [886] = 1,
    ACTIONS(265), 1,
      anon_sym_SEMI,
  [890] = 1,
    ACTIONS(267), 1,
      anon_sym_SEMI,
  [894] = 1,
    ACTIONS(269), 1,
      sym_number,
  [898] = 1,
    ACTIONS(271), 1,
      anon_sym_SEMI,
  [902] = 1,
    ACTIONS(273), 1,
      anon_sym_SEMI,
  [906] = 1,
    ACTIONS(275), 1,
      anon_sym_SEMI,
  [910] = 1,
    ACTIONS(277), 1,
      anon_sym_SEMI,
  [914] = 1,
    ACTIONS(279), 1,
      anon_sym_SEMI,
  [918] = 1,
    ACTIONS(281), 1,
      sym_text,
  [922] = 1,
    ACTIONS(283), 1,
      anon_sym_SEMI,
  [926] = 1,
    ACTIONS(285), 1,
      sym_number,
  [930] = 1,
    ACTIONS(21), 1,
      sym_text,
  [934] = 1,
    ACTIONS(17), 1,
      sym_text,
  [938] = 1,
    ACTIONS(25), 1,
      sym_text,
  [942] = 1,
    ACTIONS(287), 1,
      sym_number,
  [946] = 1,
    ACTIONS(289), 1,
      ts_builtin_sym_end,
  [950] = 1,
    ACTIONS(291), 1,
      sym_number,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(7)] = 0,
  [SMALL_STATE(8)] = 42,
  [SMALL_STATE(9)] = 84,
  [SMALL_STATE(10)] = 153,
  [SMALL_STATE(11)] = 222,
  [SMALL_STATE(12)] = 291,
  [SMALL_STATE(13)] = 350,
  [SMALL_STATE(14)] = 371,
  [SMALL_STATE(15)] = 392,
  [SMALL_STATE(16)] = 406,
  [SMALL_STATE(17)] = 420,
  [SMALL_STATE(18)] = 434,
  [SMALL_STATE(19)] = 448,
  [SMALL_STATE(20)] = 462,
  [SMALL_STATE(21)] = 476,
  [SMALL_STATE(22)] = 490,
  [SMALL_STATE(23)] = 504,
  [SMALL_STATE(24)] = 518,
  [SMALL_STATE(25)] = 532,
  [SMALL_STATE(26)] = 546,
  [SMALL_STATE(27)] = 560,
  [SMALL_STATE(28)] = 573,
  [SMALL_STATE(29)] = 590,
  [SMALL_STATE(30)] = 602,
  [SMALL_STATE(31)] = 614,
  [SMALL_STATE(32)] = 627,
  [SMALL_STATE(33)] = 640,
  [SMALL_STATE(34)] = 649,
  [SMALL_STATE(35)] = 658,
  [SMALL_STATE(36)] = 671,
  [SMALL_STATE(37)] = 684,
  [SMALL_STATE(38)] = 697,
  [SMALL_STATE(39)] = 703,
  [SMALL_STATE(40)] = 713,
  [SMALL_STATE(41)] = 721,
  [SMALL_STATE(42)] = 729,
  [SMALL_STATE(43)] = 737,
  [SMALL_STATE(44)] = 745,
  [SMALL_STATE(45)] = 751,
  [SMALL_STATE(46)] = 757,
  [SMALL_STATE(47)] = 763,
  [SMALL_STATE(48)] = 768,
  [SMALL_STATE(49)] = 773,
  [SMALL_STATE(50)] = 778,
  [SMALL_STATE(51)] = 785,
  [SMALL_STATE(52)] = 790,
  [SMALL_STATE(53)] = 797,
  [SMALL_STATE(54)] = 802,
  [SMALL_STATE(55)] = 806,
  [SMALL_STATE(56)] = 810,
  [SMALL_STATE(57)] = 814,
  [SMALL_STATE(58)] = 818,
  [SMALL_STATE(59)] = 822,
  [SMALL_STATE(60)] = 826,
  [SMALL_STATE(61)] = 830,
  [SMALL_STATE(62)] = 834,
  [SMALL_STATE(63)] = 838,
  [SMALL_STATE(64)] = 842,
  [SMALL_STATE(65)] = 846,
  [SMALL_STATE(66)] = 850,
  [SMALL_STATE(67)] = 854,
  [SMALL_STATE(68)] = 858,
  [SMALL_STATE(69)] = 862,
  [SMALL_STATE(70)] = 866,
  [SMALL_STATE(71)] = 870,
  [SMALL_STATE(72)] = 874,
  [SMALL_STATE(73)] = 878,
  [SMALL_STATE(74)] = 882,
  [SMALL_STATE(75)] = 886,
  [SMALL_STATE(76)] = 890,
  [SMALL_STATE(77)] = 894,
  [SMALL_STATE(78)] = 898,
  [SMALL_STATE(79)] = 902,
  [SMALL_STATE(80)] = 906,
  [SMALL_STATE(81)] = 910,
  [SMALL_STATE(82)] = 914,
  [SMALL_STATE(83)] = 918,
  [SMALL_STATE(84)] = 922,
  [SMALL_STATE(85)] = 926,
  [SMALL_STATE(86)] = 930,
  [SMALL_STATE(87)] = 934,
  [SMALL_STATE(88)] = 938,
  [SMALL_STATE(89)] = 942,
  [SMALL_STATE(90)] = 946,
  [SMALL_STATE(91)] = 950,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [7] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 1, 0, 2),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [11] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 1, 0, 2),
  [13] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_type, 1, 0, 0),
  [15] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_type, 1, 0, 0),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [19] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 2, 0, 2),
  [23] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 2, 0, 2),
  [25] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [27] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(85),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(47),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(69),
  [43] = {.entry = {.count = 1, .reusable = false}}, SHIFT(89),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [49] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [51] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [57] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [59] = {.entry = {.count = 1, .reusable = false}}, SHIFT(50),
  [61] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [65] = {.entry = {.count = 1, .reusable = false}}, SHIFT(22),
  [67] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [69] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [72] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [74] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(69),
  [77] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(89),
  [80] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(26),
  [83] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(62),
  [86] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(15),
  [89] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [92] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [95] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(18),
  [98] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(19),
  [101] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(50),
  [104] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [107] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [110] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(22),
  [113] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(23),
  [116] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [118] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [120] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [122] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [124] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [126] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [128] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [130] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [132] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [134] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [136] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [138] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [140] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [142] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [144] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(14),
  [147] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [149] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [151] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [153] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [155] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [157] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [165] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [167] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [169] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [171] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(63),
  [174] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [176] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [178] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0),
  [180] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0), SHIFT_REPEAT(67),
  [183] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 1, 0, 0),
  [185] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [187] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 2, 0, 0),
  [189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [195] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [197] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 3, 0, 0),
  [199] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [203] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [205] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 1, 0, 0),
  [207] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted_string, 1, 0, 0),
  [209] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_argument, 3, 0, 12),
  [211] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [213] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 3, 0, 1),
  [215] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [217] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [219] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [221] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [223] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 8),
  [225] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_effect, 1, 0, 0),
  [227] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [229] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_enchant, 1, 0, 0),
  [231] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_enchant, 1, 0, 0),
  [233] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [235] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_unit, 1, 0, 0),
  [237] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 7),
  [239] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [241] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [243] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_add_command, 4, 0, 9),
  [245] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_set_command, 4, 0, 9),
  [247] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_survival_command, 2, 0, 4),
  [249] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [251] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [253] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [255] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [257] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enchant_command, 4, 0, 10),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_adventure_command, 2, 0, 4),
  [261] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_spectator_command, 2, 0, 4),
  [263] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_type, 1, 0, 0),
  [265] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_say_command, 2, 0, 3),
  [267] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 5, 0, 11),
  [269] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_query_command, 3, 0, 5),
  [273] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_creative_command, 2, 0, 4),
  [275] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inv_clear_command, 2, 0, 4),
  [277] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command, 2, 0, 0),
  [279] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_clear_command, 2, 0, 4),
  [281] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [283] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tellraw_command, 3, 0, 6),
  [285] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_effect, 1, 0, 0),
  [287] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [289] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [291] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
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
