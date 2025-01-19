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
#define STATE_COUNT 112
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 142
#define ALIAS_COUNT 0
#define TOKEN_COUNT 108
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 15
#define MAX_ALIAS_SEQUENCE_LENGTH 7
#define PRODUCTION_ID_COUNT 16

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
  anon_sym_give = 27,
  anon_sym_minecraft_COLON = 28,
  anon_sym_enchant = 29,
  anon_sym_minecraft_COLONspeed = 30,
  anon_sym_minecraft_COLONslowness = 31,
  anon_sym_minecraft_COLONhaste = 32,
  anon_sym_minecraft_COLONmining_fatigue = 33,
  anon_sym_minecraft_COLONstrength = 34,
  anon_sym_minecraft_COLONinstant_health = 35,
  anon_sym_minecraft_COLONinstant_damage = 36,
  anon_sym_minecraft_COLONjump_boost = 37,
  anon_sym_minecraft_COLONnausea = 38,
  anon_sym_minecraft_COLONregeneration = 39,
  anon_sym_minecraft_COLONresistance = 40,
  anon_sym_minecraft_COLONfire_resistance = 41,
  anon_sym_minecraft_COLONwater_breathing = 42,
  anon_sym_minecraft_COLONinvisibility = 43,
  anon_sym_minecraft_COLONblindness = 44,
  anon_sym_minecraft_COLONnight_vision = 45,
  anon_sym_minecraft_COLONhunger = 46,
  anon_sym_minecraft_COLONweakness = 47,
  anon_sym_minecraft_COLONpoison = 48,
  anon_sym_minecraft_COLONwither = 49,
  anon_sym_minecraft_COLONhealth_boost = 50,
  anon_sym_minecraft_COLONabsorption = 51,
  anon_sym_minecraft_COLONsaturation = 52,
  anon_sym_minecraft_COLONglowing = 53,
  anon_sym_minecraft_COLONlevitation = 54,
  anon_sym_minecraft_COLONluck = 55,
  anon_sym_minecraft_COLONunluck = 56,
  anon_sym_minecraft_COLONslow_falling = 57,
  anon_sym_minecraft_COLONconduit_power = 58,
  anon_sym_minecraft_COLONdolphins_grace = 59,
  anon_sym_minecraft_COLONbad_omen = 60,
  anon_sym_minecraft_COLONhero_of_the_village = 61,
  anon_sym_minecraft_COLONprotection = 62,
  anon_sym_minecraft_COLONfire_protection = 63,
  anon_sym_minecraft_COLONfeather_falling = 64,
  anon_sym_minecraft_COLONblast_protection = 65,
  anon_sym_minecraft_COLONprojectile_protection = 66,
  anon_sym_minecraft_COLONrespiration = 67,
  anon_sym_minecraft_COLONaqua_affinity = 68,
  anon_sym_minecraft_COLONthorns = 69,
  anon_sym_minecraft_COLONdepth_strider = 70,
  anon_sym_minecraft_COLONfrost_walker = 71,
  anon_sym_minecraft_COLONbinding_curse = 72,
  anon_sym_minecraft_COLONsharpness = 73,
  anon_sym_minecraft_COLONsmite = 74,
  anon_sym_minecraft_COLONbane_of_arthropods = 75,
  anon_sym_minecraft_COLONknockback = 76,
  anon_sym_minecraft_COLONfire_aspect = 77,
  anon_sym_minecraft_COLONlooting = 78,
  anon_sym_minecraft_COLONsweeping = 79,
  anon_sym_minecraft_COLONefficiency = 80,
  anon_sym_minecraft_COLONsilk_touch = 81,
  anon_sym_minecraft_COLONunbreaking = 82,
  anon_sym_minecraft_COLONfortune = 83,
  anon_sym_minecraft_COLONpower = 84,
  anon_sym_minecraft_COLONpunch = 85,
  anon_sym_minecraft_COLONflame = 86,
  anon_sym_minecraft_COLONinfinity = 87,
  anon_sym_minecraft_COLONluck_of_the_sea = 88,
  anon_sym_minecraft_COLONlure = 89,
  anon_sym_minecraft_COLONmending = 90,
  anon_sym_minecraft_COLONvanishing_curse = 91,
  anon_sym_minecraft_COLONsoul_speed = 92,
  anon_sym_minecraft_COLONswift_sneak = 93,
  sym_text = 94,
  anon_sym_time = 95,
  anon_sym_set = 96,
  anon_sym_query = 97,
  anon_sym_gms = 98,
  anon_sym_gma = 99,
  anon_sym_gmsp = 100,
  anon_sym_gmc = 101,
  anon_sym_day = 102,
  anon_sym_night = 103,
  anon_sym_noon = 104,
  anon_sym_midnight = 105,
  sym_identifier = 106,
  sym_number = 107,
  sym_source_file = 108,
  sym__definition = 109,
  sym_function_definition = 110,
  sym_block = 111,
  sym__command = 112,
  sym_target_selector = 113,
  sym_selector_type = 114,
  sym_selector_arguments = 115,
  sym_selector_argument = 116,
  sym_xp_type = 117,
  sym_range_value = 118,
  sym_xp_add_command = 119,
  sym_xp_set_command = 120,
  sym_xp_query_command = 121,
  sym_quoted_string = 122,
  sym_say_command = 123,
  sym_inv_clear_command = 124,
  sym_effect_clear_command = 125,
  sym_tellraw_command = 126,
  sym_effect_command = 127,
  sym_enchant_command = 128,
  sym_vanilla_effect = 129,
  sym_vanilla_enchant = 130,
  sym_custom_effect = 131,
  sym_custom_enchant = 132,
  sym_time_command = 133,
  sym_gm_survival_command = 134,
  sym_gm_adventure_command = 135,
  sym_gm_spectator_command = 136,
  sym_gm_creative_command = 137,
  sym_time_unit = 138,
  aux_sym_source_file_repeat1 = 139,
  aux_sym_block_repeat1 = 140,
  aux_sym_selector_arguments_repeat1 = 141,
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
  [anon_sym_give] = "give",
  [anon_sym_minecraft_COLON] = "minecraft:",
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
  [anon_sym_give] = anon_sym_give,
  [anon_sym_minecraft_COLON] = anon_sym_minecraft_COLON,
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
  [anon_sym_give] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minecraft_COLON] = {
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
  [13] = {.index = 23, .length = 4},
  [14] = {.index = 27, .length = 5},
  [15] = {.index = 32, .length = 5},
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
  [23] =
    {field_amplifier, 5},
    {field_duration, 4},
    {field_effect_type, 3},
    {field_target, 2},
  [27] =
    {field_amplifier, 5},
    {field_duration, 4},
    {field_effect_type, 2},
    {field_effect_type, 3},
    {field_target, 1},
  [32] =
    {field_amplifier, 6},
    {field_duration, 5},
    {field_effect_type, 3},
    {field_effect_type, 4},
    {field_target, 2},
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
  [12] = 6,
  [13] = 13,
  [14] = 14,
  [15] = 5,
  [16] = 4,
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
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 44,
  [46] = 43,
  [47] = 43,
  [48] = 44,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 2,
  [57] = 57,
  [58] = 3,
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
  [99] = 5,
  [100] = 4,
  [101] = 6,
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
        'c', 358,
        'd', 42,
        'e', 128,
        'f', 384,
        'g', 288,
        'l', 207,
        'm', 289,
        'n', 290,
        'p', 447,
        'q', 637,
        's', 50,
        't', 166,
        'x', 488,
        '{', 671,
        '}', 674,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(781);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == ',') ADVANCE(682);
      if (lookahead == '.') ADVANCE(5);
      if (lookahead == ']') ADVANCE(683);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(781);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(780);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == ';') ADVANCE(672);
      if (lookahead == '[') ADVANCE(680);
      if (lookahead == 'l') ADVANCE(207);
      if (lookahead == 'm') ADVANCE(317);
      if (lookahead == 'p') ADVANCE(447);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == '[') ADVANCE(680);
      if (lookahead == 'm') ADVANCE(342);
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
      if (lookahead == ':') ADVANCE(698);
      END_STATE();
    case 7:
      if (lookahead == ':') ADVANCE(699);
      END_STATE();
    case 8:
      if (lookahead == ':') ADVANCE(48);
      END_STATE();
    case 9:
      if (lookahead == '_') ADVANCE(71);
      END_STATE();
    case 10:
      if (lookahead == '_') ADVANCE(102);
      END_STATE();
    case 11:
      if (lookahead == '_') ADVANCE(143);
      END_STATE();
    case 12:
      if (lookahead == '_') ADVANCE(70);
      END_STATE();
    case 13:
      if (lookahead == '_') ADVANCE(658);
      END_STATE();
    case 14:
      if (lookahead == '_') ADVANCE(105);
      END_STATE();
    case 15:
      if (lookahead == '_') ADVANCE(235);
      if (lookahead == 'n') ADVANCE(187);
      END_STATE();
    case 16:
      if (lookahead == '_') ADVANCE(118);
      END_STATE();
    case 17:
      if (lookahead == '_') ADVANCE(500);
      END_STATE();
    case 18:
      if (lookahead == '_') ADVANCE(266);
      END_STATE();
    case 19:
      if (lookahead == '_') ADVANCE(550);
      END_STATE();
    case 20:
      if (lookahead == '_') ADVANCE(493);
      END_STATE();
    case 21:
      if (lookahead == '_') ADVANCE(79);
      END_STATE();
    case 22:
      if (lookahead == '_') ADVANCE(478);
      END_STATE();
    case 23:
      if (lookahead == '_') ADVANCE(455);
      END_STATE();
    case 24:
      if (lookahead == '_') ADVANCE(575);
      END_STATE();
    case 25:
      if (lookahead == '_') ADVANCE(456);
      END_STATE();
    case 26:
      if (lookahead == '_') ADVANCE(610);
      END_STATE();
    case 27:
      if (lookahead == '_') ADVANCE(654);
      END_STATE();
    case 28:
      if (lookahead == '_') ADVANCE(240);
      END_STATE();
    case 29:
      if (lookahead == '_') ADVANCE(652);
      END_STATE();
    case 30:
      if (lookahead == '_') ADVANCE(569);
      END_STATE();
    case 31:
      if (lookahead == '_') ADVANCE(620);
      END_STATE();
    case 32:
      if (lookahead == '_') ADVANCE(83);
      END_STATE();
    case 33:
      if (lookahead == '_') ADVANCE(621);
      END_STATE();
    case 34:
      if (lookahead == '_') ADVANCE(565);
      END_STATE();
    case 35:
      if (lookahead == '_') ADVANCE(525);
      END_STATE();
    case 36:
      if (lookahead == '_') ADVANCE(485);
      END_STATE();
    case 37:
      if (lookahead == '_') ADVANCE(106);
      END_STATE();
    case 38:
      if (lookahead == '_') ADVANCE(245);
      END_STATE();
    case 39:
      if (lookahead == '_') ADVANCE(131);
      END_STATE();
    case 40:
      if (lookahead == '_') ADVANCE(501);
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
      if (lookahead == 'a') ADVANCE(773);
      if (lookahead == 'c') ADVANCE(775);
      if (lookahead == 's') ADVANCE(772);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(141);
      if (lookahead == 'q') ADVANCE(643);
      if (lookahead == 's') ADVANCE(177);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(655);
      END_STATE();
    case 46:
      if (lookahead == 'a') ADVANCE(709);
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(760);
      END_STATE();
    case 48:
      ADVANCE_MAP(
        'a', 502,
        'b', 75,
        'd', 168,
        'e', 230,
        'f', 181,
        'i', 443,
        'k', 408,
        'l', 454,
        'm', 221,
        'p', 476,
        'r', 201,
        's', 283,
        't', 276,
        'u', 440,
        'v', 88,
      );
      END_STATE();
    case 49:
      if (lookahead == 'a') ADVANCE(662);
      END_STATE();
    case 50:
      if (lookahead == 'a') ADVANCE(662);
      if (lookahead == 'e') ADVANCE(578);
      END_STATE();
    case 51:
      if (lookahead == 'a') ADVANCE(233);
      END_STATE();
    case 52:
      if (lookahead == 'a') ADVANCE(137);
      if (lookahead == 'i') ADVANCE(405);
      if (lookahead == 'l') ADVANCE(81);
      END_STATE();
    case 53:
      if (lookahead == 'a') ADVANCE(503);
      END_STATE();
    case 54:
      if (lookahead == 'a') ADVANCE(21);
      END_STATE();
    case 55:
      if (lookahead == 'a') ADVANCE(381);
      END_STATE();
    case 56:
      if (lookahead == 'a') ADVANCE(354);
      END_STATE();
    case 57:
      if (lookahead == 'a') ADVANCE(638);
      if (lookahead == 'i') ADVANCE(265);
      END_STATE();
    case 58:
      if (lookahead == 'a') ADVANCE(504);
      END_STATE();
    case 59:
      if (lookahead == 'a') ADVANCE(571);
      if (lookahead == 'e') ADVANCE(63);
      if (lookahead == 'u') ADVANCE(401);
      END_STATE();
    case 60:
      if (lookahead == 'a') ADVANCE(412);
      END_STATE();
    case 61:
      if (lookahead == 'a') ADVANCE(355);
      END_STATE();
    case 62:
      if (lookahead == 'a') ADVANCE(515);
      END_STATE();
    case 63:
      if (lookahead == 'a') ADVANCE(369);
      if (lookahead == 'r') ADVANCE(479);
      END_STATE();
    case 64:
      ADVANCE_MAP(
        'a', 593,
        'h', 62,
        'i', 359,
        'l', 450,
        'm', 325,
        'o', 639,
        'p', 192,
        't', 521,
        'w', 197,
      );
      END_STATE();
    case 65:
      if (lookahead == 'a') ADVANCE(593);
      if (lookahead == 'l') ADVANCE(450);
      if (lookahead == 'p') ADVANCE(192);
      if (lookahead == 't') ADVANCE(521);
      END_STATE();
    case 66:
      if (lookahead == 'a') ADVANCE(349);
      END_STATE();
    case 67:
      if (lookahead == 'a') ADVANCE(366);
      END_STATE();
    case 68:
      if (lookahead == 'a') ADVANCE(612);
      if (lookahead == 'e') ADVANCE(56);
      if (lookahead == 'i') ADVANCE(592);
      END_STATE();
    case 69:
      if (lookahead == 'a') ADVANCE(136);
      if (lookahead == 'l') ADVANCE(301);
      END_STATE();
    case 70:
      if (lookahead == 'a') ADVANCE(551);
      if (lookahead == 'p') ADVANCE(533);
      END_STATE();
    case 71:
      if (lookahead == 'a') ADVANCE(551);
      if (lookahead == 'p') ADVANCE(533);
      if (lookahead == 'r') ADVANCE(212);
      END_STATE();
    case 72:
      if (lookahead == 'a') ADVANCE(415);
      END_STATE();
    case 73:
      if (lookahead == 'a') ADVANCE(609);
      END_STATE();
    case 74:
      if (lookahead == 'a') ADVANCE(625);
      END_STATE();
    case 75:
      if (lookahead == 'a') ADVANCE(419);
      if (lookahead == 'i') ADVANCE(405);
      if (lookahead == 'l') ADVANCE(80);
      END_STATE();
    case 76:
      if (lookahead == 'a') ADVANCE(382);
      END_STATE();
    case 77:
      if (lookahead == 'a') ADVANCE(367);
      END_STATE();
    case 78:
      if (lookahead == 'a') ADVANCE(262);
      END_STATE();
    case 79:
      if (lookahead == 'a') ADVANCE(232);
      END_STATE();
    case 80:
      if (lookahead == 'a') ADVANCE(558);
      END_STATE();
    case 81:
      if (lookahead == 'a') ADVANCE(558);
      if (lookahead == 'i') ADVANCE(410);
      END_STATE();
    case 82:
      if (lookahead == 'a') ADVANCE(425);
      END_STATE();
    case 83:
      if (lookahead == 'a') ADVANCE(526);
      END_STATE();
    case 84:
      if (lookahead == 'a') ADVANCE(371);
      END_STATE();
    case 85:
      if (lookahead == 'a') ADVANCE(124);
      END_STATE();
    case 86:
      if (lookahead == 'a') ADVANCE(616);
      END_STATE();
    case 87:
      if (lookahead == 'a') ADVANCE(434);
      END_STATE();
    case 88:
      if (lookahead == 'a') ADVANCE(432);
      END_STATE();
    case 89:
      if (lookahead == 'a') ADVANCE(237);
      END_STATE();
    case 90:
      if (lookahead == 'a') ADVANCE(263);
      END_STATE();
    case 91:
      if (lookahead == 'a') ADVANCE(114);
      END_STATE();
    case 92:
      if (lookahead == 'a') ADVANCE(238);
      END_STATE();
    case 93:
      if (lookahead == 'a') ADVANCE(624);
      END_STATE();
    case 94:
      if (lookahead == 'a') ADVANCE(376);
      END_STATE();
    case 95:
      if (lookahead == 'a') ADVANCE(627);
      END_STATE();
    case 96:
      if (lookahead == 'a') ADVANCE(628);
      END_STATE();
    case 97:
      if (lookahead == 'a') ADVANCE(629);
      END_STATE();
    case 98:
      if (lookahead == 'b') ADVANCE(553);
      END_STATE();
    case 99:
      if (lookahead == 'b') ADVANCE(553);
      if (lookahead == 'q') ADVANCE(640);
      END_STATE();
    case 100:
      if (lookahead == 'b') ADVANCE(300);
      END_STATE();
    case 101:
      if (lookahead == 'b') ADVANCE(91);
      END_STATE();
    case 102:
      if (lookahead == 'b') ADVANCE(483);
      END_STATE();
    case 103:
      if (lookahead == 'b') ADVANCE(524);
      END_STATE();
    case 104:
      if (lookahead == 'b') ADVANCE(524);
      if (lookahead == 'l') ADVANCE(644);
      END_STATE();
    case 105:
      if (lookahead == 'b') ADVANCE(527);
      END_STATE();
    case 106:
      if (lookahead == 'b') ADVANCE(484);
      END_STATE();
    case 107:
      if (lookahead == 'c') ADVANCE(358);
      if (lookahead == 'e') ADVANCE(128);
      if (lookahead == 'g') ADVANCE(378);
      if (lookahead == 's') ADVANCE(49);
      if (lookahead == 't') ADVANCE(166);
      if (lookahead == 'x') ADVANCE(488);
      if (lookahead == '}') ADVANCE(674);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(673);
      END_STATE();
    case 108:
      if (lookahead == 'c') ADVANCE(273);
      END_STATE();
    case 109:
      if (lookahead == 'c') ADVANCE(346);
      if (lookahead == 'r') ADVANCE(153);
      END_STATE();
    case 110:
      if (lookahead == 'c') ADVANCE(351);
      END_STATE();
    case 111:
      if (lookahead == 'c') ADVANCE(581);
      END_STATE();
    case 112:
      if (lookahead == 'c') ADVANCE(347);
      END_STATE();
    case 113:
      if (lookahead == 'c') ADVANCE(666);
      END_STATE();
    case 114:
      if (lookahead == 'c') ADVANCE(348);
      END_STATE();
    case 115:
      if (lookahead == 'c') ADVANCE(268);
      END_STATE();
    case 116:
      if (lookahead == 'c') ADVANCE(311);
      END_STATE();
    case 117:
      if (lookahead == 'c') ADVANCE(350);
      END_STATE();
    case 118:
      if (lookahead == 'c') ADVANCE(641);
      END_STATE();
    case 119:
      if (lookahead == 'c') ADVANCE(270);
      END_STATE();
    case 120:
      if (lookahead == 'c') ADVANCE(356);
      if (lookahead == 'r') ADVANCE(153);
      END_STATE();
    case 121:
      if (lookahead == 'c') ADVANCE(605);
      END_STATE();
    case 122:
      if (lookahead == 'c') ADVANCE(586);
      END_STATE();
    case 123:
      if (lookahead == 'c') ADVANCE(158);
      END_STATE();
    case 124:
      if (lookahead == 'c') ADVANCE(160);
      END_STATE();
    case 125:
      if (lookahead == 'c') ADVANCE(163);
      END_STATE();
    case 126:
      if (lookahead == 'c') ADVANCE(514);
      END_STATE();
    case 127:
      if (lookahead == 'c') ADVANCE(531);
      END_STATE();
    case 128:
      if (lookahead == 'c') ADVANCE(372);
      if (lookahead == 'f') ADVANCE(229);
      if (lookahead == 'n') ADVANCE(108);
      END_STATE();
    case 129:
      if (lookahead == 'c') ADVANCE(536);
      END_STATE();
    case 130:
      if (lookahead == 'c') ADVANCE(626);
      END_STATE();
    case 131:
      if (lookahead == 'c') ADVANCE(649);
      END_STATE();
    case 132:
      if (lookahead == 'c') ADVANCE(630);
      END_STATE();
    case 133:
      if (lookahead == 'c') ADVANCE(631);
      END_STATE();
    case 134:
      if (lookahead == 'c') ADVANCE(632);
      END_STATE();
    case 135:
      if (lookahead == 'd') ADVANCE(688);
      END_STATE();
    case 136:
      if (lookahead == 'd') ADVANCE(22);
      END_STATE();
    case 137:
      if (lookahead == 'd') ADVANCE(22);
      if (lookahead == 'n') ADVANCE(209);
      END_STATE();
    case 138:
      if (lookahead == 'd') ADVANCE(701);
      END_STATE();
    case 139:
      if (lookahead == 'd') ADVANCE(764);
      END_STATE();
    case 140:
      if (lookahead == 'd') ADVANCE(402);
      if (lookahead == 'n') ADVANCE(174);
      END_STATE();
    case 141:
      if (lookahead == 'd') ADVANCE(135);
      END_STATE();
    case 142:
      if (lookahead == 'd') ADVANCE(647);
      END_STATE();
    case 143:
      if (lookahead == 'd') ADVANCE(55);
      if (lookahead == 'h') ADVANCE(196);
      END_STATE();
    case 144:
      if (lookahead == 'd') ADVANCE(549);
      END_STATE();
    case 145:
      if (lookahead == 'd') ADVANCE(200);
      END_STATE();
    case 146:
      if (lookahead == 'd') ADVANCE(322);
      END_STATE();
    case 147:
      if (lookahead == 'd') ADVANCE(318);
      END_STATE();
    case 148:
      if (lookahead == 'd') ADVANCE(435);
      END_STATE();
    case 149:
      if (lookahead == 'e') ADVANCE(511);
      END_STATE();
    case 150:
      if (lookahead == 'e') ADVANCE(697);
      END_STATE();
    case 151:
      if (lookahead == 'e') ADVANCE(769);
      END_STATE();
    case 152:
      if (lookahead == 'e') ADVANCE(258);
      END_STATE();
    case 153:
      if (lookahead == 'e') ADVANCE(761);
      END_STATE();
    case 154:
      if (lookahead == 'e') ADVANCE(758);
      END_STATE();
    case 155:
      if (lookahead == 'e') ADVANCE(703);
      END_STATE();
    case 156:
      if (lookahead == 'e') ADVANCE(746);
      END_STATE();
    case 157:
      if (lookahead == 'e') ADVANCE(755);
      END_STATE();
    case 158:
      if (lookahead == 'e') ADVANCE(711);
      END_STATE();
    case 159:
      if (lookahead == 'e') ADVANCE(744);
      END_STATE();
    case 160:
      if (lookahead == 'e') ADVANCE(731);
      END_STATE();
    case 161:
      if (lookahead == 'e') ADVANCE(707);
      END_STATE();
    case 162:
      if (lookahead == 'e') ADVANCE(704);
      END_STATE();
    case 163:
      if (lookahead == 'e') ADVANCE(712);
      END_STATE();
    case 164:
      if (lookahead == 'e') ADVANCE(763);
      END_STATE();
    case 165:
      if (lookahead == 'e') ADVANCE(733);
      END_STATE();
    case 166:
      if (lookahead == 'e') ADVANCE(360);
      if (lookahead == 'i') ADVANCE(380);
      END_STATE();
    case 167:
      if (lookahead == 'e') ADVANCE(53);
      END_STATE();
    case 168:
      if (lookahead == 'e') ADVANCE(492);
      END_STATE();
    case 169:
      if (lookahead == 'e') ADVANCE(492);
      if (lookahead == 'o') ADVANCE(361);
      END_STATE();
    case 170:
      if (lookahead == 'e') ADVANCE(111);
      END_STATE();
    case 171:
      if (lookahead == 'e') ADVANCE(653);
      if (lookahead == 'o') ADVANCE(462);
      if (lookahead == 'u') ADVANCE(109);
      END_STATE();
    case 172:
      if (lookahead == 'e') ADVANCE(653);
      if (lookahead == 'u') ADVANCE(117);
      END_STATE();
    case 173:
      if (lookahead == 'e') ADVANCE(259);
      END_STATE();
    case 174:
      if (lookahead == 'e') ADVANCE(126);
      END_STATE();
    case 175:
      if (lookahead == 'e') ADVANCE(9);
      END_STATE();
    case 176:
      if (lookahead == 'e') ADVANCE(357);
      END_STATE();
    case 177:
      if (lookahead == 'e') ADVANCE(580);
      END_STATE();
    case 178:
      if (lookahead == 'e') ADVANCE(498);
      END_STATE();
    case 179:
      if (lookahead == 'e') ADVANCE(138);
      END_STATE();
    case 180:
      if (lookahead == 'e') ADVANCE(86);
      if (lookahead == 'i') ADVANCE(517);
      if (lookahead == 'l') ADVANCE(76);
      if (lookahead == 'o') ADVANCE(518);
      if (lookahead == 'r') ADVANCE(480);
      END_STATE();
    case 181:
      if (lookahead == 'e') ADVANCE(86);
      if (lookahead == 'i') ADVANCE(529);
      if (lookahead == 'l') ADVANCE(76);
      if (lookahead == 'o') ADVANCE(518);
      if (lookahead == 'r') ADVANCE(480);
      END_STATE();
    case 182:
      if (lookahead == 'e') ADVANCE(139);
      END_STATE();
    case 183:
      if (lookahead == 'e') ADVANCE(505);
      END_STATE();
    case 184:
      if (lookahead == 'e') ADVANCE(516);
      END_STATE();
    case 185:
      if (lookahead == 'e') ADVANCE(46);
      END_STATE();
    case 186:
      if (lookahead == 'e') ADVANCE(506);
      END_STATE();
    case 187:
      if (lookahead == 'e') ADVANCE(557);
      END_STATE();
    case 188:
      if (lookahead == 'e') ADVANCE(61);
      END_STATE();
    case 189:
      if (lookahead == 'e') ADVANCE(507);
      END_STATE();
    case 190:
      if (lookahead == 'e') ADVANCE(426);
      END_STATE();
    case 191:
      if (lookahead == 'e') ADVANCE(406);
      END_STATE();
    case 192:
      if (lookahead == 'e') ADVANCE(179);
      END_STATE();
    case 193:
      if (lookahead == 'e') ADVANCE(508);
      END_STATE();
    case 194:
      if (lookahead == 'e') ADVANCE(66);
      END_STATE();
    case 195:
      if (lookahead == 'e') ADVANCE(30);
      END_STATE();
    case 196:
      if (lookahead == 'e') ADVANCE(84);
      END_STATE();
    case 197:
      if (lookahead == 'e') ADVANCE(178);
      if (lookahead == 'i') ADVANCE(236);
      END_STATE();
    case 198:
      if (lookahead == 'e') ADVANCE(509);
      END_STATE();
    case 199:
      if (lookahead == 'e') ADVANCE(35);
      END_STATE();
    case 200:
      if (lookahead == 'e') ADVANCE(510);
      END_STATE();
    case 201:
      if (lookahead == 'e') ADVANCE(554);
      END_STATE();
    case 202:
      if (lookahead == 'e') ADVANCE(12);
      END_STATE();
    case 203:
      if (lookahead == 'e') ADVANCE(47);
      END_STATE();
    case 204:
      if (lookahead == 'e') ADVANCE(389);
      END_STATE();
    case 205:
      if (lookahead == 'e') ADVANCE(413);
      END_STATE();
    case 206:
      if (lookahead == 'e') ADVANCE(182);
      END_STATE();
    case 207:
      if (lookahead == 'e') ADVANCE(651);
      END_STATE();
    case 208:
      if (lookahead == 'e') ADVANCE(513);
      END_STATE();
    case 209:
      if (lookahead == 'e') ADVANCE(23);
      END_STATE();
    case 210:
      if (lookahead == 'e') ADVANCE(58);
      END_STATE();
    case 211:
      if (lookahead == 'e') ADVANCE(121);
      END_STATE();
    case 212:
      if (lookahead == 'e') ADVANCE(555);
      END_STATE();
    case 213:
      if (lookahead == 'e') ADVANCE(534);
      END_STATE();
    case 214:
      if (lookahead == 'e') ADVANCE(559);
      END_STATE();
    case 215:
      if (lookahead == 'e') ADVANCE(130);
      END_STATE();
    case 216:
      if (lookahead == 'e') ADVANCE(561);
      END_STATE();
    case 217:
      if (lookahead == 'e') ADVANCE(122);
      END_STATE();
    case 218:
      if (lookahead == 'e') ADVANCE(74);
      END_STATE();
    case 219:
      if (lookahead == 'e') ADVANCE(563);
      END_STATE();
    case 220:
      if (lookahead == 'e') ADVANCE(127);
      END_STATE();
    case 221:
      if (lookahead == 'e') ADVANCE(439);
      END_STATE();
    case 222:
      if (lookahead == 'e') ADVANCE(29);
      END_STATE();
    case 223:
      if (lookahead == 'e') ADVANCE(129);
      END_STATE();
    case 224:
      if (lookahead == 'e') ADVANCE(132);
      END_STATE();
    case 225:
      if (lookahead == 'e') ADVANCE(133);
      END_STATE();
    case 226:
      if (lookahead == 'e') ADVANCE(539);
      END_STATE();
    case 227:
      if (lookahead == 'e') ADVANCE(134);
      END_STATE();
    case 228:
      if (lookahead == 'e') ADVANCE(40);
      END_STATE();
    case 229:
      if (lookahead == 'f') ADVANCE(170);
      END_STATE();
    case 230:
      if (lookahead == 'f') ADVANCE(231);
      END_STATE();
    case 231:
      if (lookahead == 'f') ADVANCE(293);
      END_STATE();
    case 232:
      if (lookahead == 'f') ADVANCE(244);
      END_STATE();
    case 233:
      if (lookahead == 'f') ADVANCE(584);
      END_STATE();
    case 234:
      if (lookahead == 'f') ADVANCE(32);
      END_STATE();
    case 235:
      if (lookahead == 'f') ADVANCE(77);
      END_STATE();
    case 236:
      if (lookahead == 'f') ADVANCE(617);
      END_STATE();
    case 237:
      if (lookahead == 'f') ADVANCE(589);
      END_STATE();
    case 238:
      if (lookahead == 'f') ADVANCE(590);
      END_STATE();
    case 239:
      if (lookahead == 'f') ADVANCE(31);
      END_STATE();
    case 240:
      if (lookahead == 'f') ADVANCE(73);
      END_STATE();
    case 241:
      if (lookahead == 'f') ADVANCE(326);
      END_STATE();
    case 242:
      if (lookahead == 'f') ADVANCE(326);
      if (lookahead == 's') ADVANCE(623);
      if (lookahead == 'v') ADVANCE(296);
      END_STATE();
    case 243:
      if (lookahead == 'f') ADVANCE(33);
      END_STATE();
    case 244:
      if (lookahead == 'f') ADVANCE(341);
      END_STATE();
    case 245:
      if (lookahead == 'f') ADVANCE(94);
      END_STATE();
    case 246:
      if (lookahead == 'g') ADVANCE(724);
      END_STATE();
    case 247:
      if (lookahead == 'g') ADVANCE(750);
      END_STATE();
    case 248:
      if (lookahead == 'g') ADVANCE(751);
      END_STATE();
    case 249:
      if (lookahead == 'g') ADVANCE(754);
      END_STATE();
    case 250:
      if (lookahead == 'g') ADVANCE(729);
      END_STATE();
    case 251:
      if (lookahead == 'g') ADVANCE(736);
      END_STATE();
    case 252:
      if (lookahead == 'g') ADVANCE(713);
      END_STATE();
    case 253:
      if (lookahead == 'g') ADVANCE(762);
      END_STATE();
    case 254:
      if (lookahead == 'g') ADVANCE(272);
      END_STATE();
    case 255:
      if (lookahead == 'g') ADVANCE(274);
      END_STATE();
    case 256:
      if (lookahead == 'g') ADVANCE(28);
      END_STATE();
    case 257:
      if (lookahead == 'g') ADVANCE(16);
      END_STATE();
    case 258:
      if (lookahead == 'g') ADVANCE(190);
      if (lookahead == 's') ADVANCE(329);
      END_STATE();
    case 259:
      if (lookahead == 'g') ADVANCE(190);
      if (lookahead == 's') ADVANCE(328);
      END_STATE();
    case 260:
      if (lookahead == 'g') ADVANCE(186);
      END_STATE();
    case 261:
      if (lookahead == 'g') ADVANCE(601);
      END_STATE();
    case 262:
      if (lookahead == 'g') ADVANCE(161);
      END_STATE();
    case 263:
      if (lookahead == 'g') ADVANCE(165);
      END_STATE();
    case 264:
      if (lookahead == 'g') ADVANCE(642);
      END_STATE();
    case 265:
      if (lookahead == 'g') ADVANCE(278);
      END_STATE();
    case 266:
      if (lookahead == 'g') ADVANCE(523);
      END_STATE();
    case 267:
      if (lookahead == 'g') ADVANCE(39);
      END_STATE();
    case 268:
      if (lookahead == 'h') ADVANCE(757);
      END_STATE();
    case 269:
      if (lookahead == 'h') ADVANCE(705);
      END_STATE();
    case 270:
      if (lookahead == 'h') ADVANCE(753);
      END_STATE();
    case 271:
      if (lookahead == 'h') ADVANCE(706);
      END_STATE();
    case 272:
      if (lookahead == 'h') ADVANCE(579);
      END_STATE();
    case 273:
      if (lookahead == 'h') ADVANCE(60);
      END_STATE();
    case 274:
      if (lookahead == 'h') ADVANCE(583);
      END_STATE();
    case 275:
      if (lookahead == 'h') ADVANCE(34);
      END_STATE();
    case 276:
      if (lookahead == 'h') ADVANCE(457);
      END_STATE();
    case 277:
      if (lookahead == 'h') ADVANCE(522);
      END_STATE();
    case 278:
      if (lookahead == 'h') ADVANCE(602);
      END_STATE();
    case 279:
      if (lookahead == 'h') ADVANCE(189);
      END_STATE();
    case 280:
      if (lookahead == 'h') ADVANCE(222);
      END_STATE();
    case 281:
      if (lookahead == 'h') ADVANCE(195);
      END_STATE();
    case 282:
      if (lookahead == 'h') ADVANCE(213);
      END_STATE();
    case 283:
      if (lookahead == 'h') ADVANCE(62);
      if (lookahead == 'i') ADVANCE(359);
      if (lookahead == 'm') ADVANCE(325);
      if (lookahead == 'o') ADVANCE(639);
      if (lookahead == 'w') ADVANCE(197);
      END_STATE();
    case 284:
      if (lookahead == 'h') ADVANCE(306);
      END_STATE();
    case 285:
      if (lookahead == 'h') ADVANCE(315);
      END_STATE();
    case 286:
      if (lookahead == 'h') ADVANCE(37);
      END_STATE();
    case 287:
      if (lookahead == 'h') ADVANCE(343);
      END_STATE();
    case 288:
      if (lookahead == 'i') ADVANCE(650);
      if (lookahead == 'm') ADVANCE(43);
      END_STATE();
    case 289:
      if (lookahead == 'i') ADVANCE(140);
      END_STATE();
    case 290:
      if (lookahead == 'i') ADVANCE(254);
      if (lookahead == 'o') ADVANCE(449);
      END_STATE();
    case 291:
      if (lookahead == 'i') ADVANCE(100);
      END_STATE();
    case 292:
      if (lookahead == 'i') ADVANCE(404);
      END_STATE();
    case 293:
      if (lookahead == 'i') ADVANCE(116);
      END_STATE();
    case 294:
      if (lookahead == 'i') ADVANCE(573);
      END_STATE();
    case 295:
      if (lookahead == 'i') ADVANCE(573);
      if (lookahead == 'w') ADVANCE(183);
      END_STATE();
    case 296:
      if (lookahead == 'i') ADVANCE(552);
      END_STATE();
    case 297:
      if (lookahead == 'i') ADVANCE(145);
      END_STATE();
    case 298:
      if (lookahead == 'i') ADVANCE(576);
      END_STATE();
    case 299:
      if (lookahead == 'i') ADVANCE(264);
      END_STATE();
    case 300:
      if (lookahead == 'i') ADVANCE(373);
      END_STATE();
    case 301:
      if (lookahead == 'i') ADVANCE(410);
      END_STATE();
    case 302:
      if (lookahead == 'i') ADVANCE(403);
      END_STATE();
    case 303:
      if (lookahead == 'i') ADVANCE(409);
      END_STATE();
    case 304:
      if (lookahead == 'i') ADVANCE(411);
      END_STATE();
    case 305:
      if (lookahead == 'i') ADVANCE(618);
      END_STATE();
    case 306:
      if (lookahead == 'i') ADVANCE(424);
      END_STATE();
    case 307:
      if (lookahead == 'i') ADVANCE(414);
      END_STATE();
    case 308:
      if (lookahead == 'i') ADVANCE(608);
      END_STATE();
    case 309:
      if (lookahead == 'i') ADVANCE(591);
      END_STATE();
    case 310:
      if (lookahead == 'i') ADVANCE(416);
      END_STATE();
    case 311:
      if (lookahead == 'i') ADVANCE(205);
      END_STATE();
    case 312:
      if (lookahead == 'i') ADVANCE(418);
      END_STATE();
    case 313:
      if (lookahead == 'i') ADVANCE(420);
      END_STATE();
    case 314:
      if (lookahead == 'i') ADVANCE(595);
      END_STATE();
    case 315:
      if (lookahead == 'i') ADVANCE(421);
      END_STATE();
    case 316:
      if (lookahead == 'i') ADVANCE(597);
      END_STATE();
    case 317:
      if (lookahead == 'i') ADVANCE(441);
      END_STATE();
    case 318:
      if (lookahead == 'i') ADVANCE(423);
      END_STATE();
    case 319:
      if (lookahead == 'i') ADVANCE(370);
      END_STATE();
    case 320:
      if (lookahead == 'i') ADVANCE(255);
      END_STATE();
    case 321:
      if (lookahead == 'i') ADVANCE(430);
      END_STATE();
    case 322:
      if (lookahead == 'i') ADVANCE(431);
      END_STATE();
    case 323:
      if (lookahead == 'i') ADVANCE(368);
      END_STATE();
    case 324:
      if (lookahead == 'i') ADVANCE(577);
      END_STATE();
    case 325:
      if (lookahead == 'i') ADVANCE(614);
      END_STATE();
    case 326:
      if (lookahead == 'i') ADVANCE(436);
      END_STATE();
    case 327:
      if (lookahead == 'i') ADVANCE(465);
      END_STATE();
    case 328:
      if (lookahead == 'i') ADVANCE(572);
      END_STATE();
    case 329:
      if (lookahead == 'i') ADVANCE(572);
      if (lookahead == 'p') ADVANCE(344);
      END_STATE();
    case 330:
      if (lookahead == 'i') ADVANCE(466);
      END_STATE();
    case 331:
      if (lookahead == 'i') ADVANCE(528);
      END_STATE();
    case 332:
      if (lookahead == 'i') ADVANCE(467);
      END_STATE();
    case 333:
      if (lookahead == 'i') ADVANCE(468);
      END_STATE();
    case 334:
      if (lookahead == 'i') ADVANCE(574);
      END_STATE();
    case 335:
      if (lookahead == 'i') ADVANCE(470);
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
      if (lookahead == 'i') ADVANCE(438);
      END_STATE();
    case 342:
      if (lookahead == 'i') ADVANCE(442);
      END_STATE();
    case 343:
      if (lookahead == 'i') ADVANCE(444);
      END_STATE();
    case 344:
      if (lookahead == 'i') ADVANCE(538);
      END_STATE();
    case 345:
      if (lookahead == 'j') ADVANCE(211);
      if (lookahead == 't') ADVANCE(215);
      END_STATE();
    case 346:
      if (lookahead == 'k') ADVANCE(727);
      END_STATE();
    case 347:
      if (lookahead == 'k') ADVANCE(728);
      END_STATE();
    case 348:
      if (lookahead == 'k') ADVANCE(748);
      END_STATE();
    case 349:
      if (lookahead == 'k') ADVANCE(765);
      END_STATE();
    case 350:
      if (lookahead == 'k') ADVANCE(726);
      END_STATE();
    case 351:
      if (lookahead == 'k') ADVANCE(101);
      END_STATE();
    case 352:
      if (lookahead == 'k') ADVANCE(26);
      END_STATE();
    case 353:
      if (lookahead == 'k') ADVANCE(193);
      END_STATE();
    case 354:
      if (lookahead == 'k') ADVANCE(433);
      END_STATE();
    case 355:
      if (lookahead == 'k') ADVANCE(310);
      END_STATE();
    case 356:
      if (lookahead == 'k') ADVANCE(36);
      END_STATE();
    case 357:
      if (lookahead == 'l') ADVANCE(542);
      END_STATE();
    case 358:
      if (lookahead == 'l') ADVANCE(167);
      END_STATE();
    case 359:
      if (lookahead == 'l') ADVANCE(352);
      END_STATE();
    case 360:
      if (lookahead == 'l') ADVANCE(362);
      END_STATE();
    case 361:
      if (lookahead == 'l') ADVANCE(490);
      END_STATE();
    case 362:
      if (lookahead == 'l') ADVANCE(512);
      END_STATE();
    case 363:
      if (lookahead == 'l') ADVANCE(448);
      END_STATE();
    case 364:
      if (lookahead == 'l') ADVANCE(19);
      END_STATE();
    case 365:
      if (lookahead == 'l') ADVANCE(644);
      END_STATE();
    case 366:
      if (lookahead == 'l') ADVANCE(353);
      END_STATE();
    case 367:
      if (lookahead == 'l') ADVANCE(374);
      END_STATE();
    case 368:
      if (lookahead == 'l') ADVANCE(377);
      END_STATE();
    case 369:
      if (lookahead == 'l') ADVANCE(599);
      END_STATE();
    case 370:
      if (lookahead == 'l') ADVANCE(228);
      END_STATE();
    case 371:
      if (lookahead == 'l') ADVANCE(606);
      END_STATE();
    case 372:
      if (lookahead == 'l') ADVANCE(210);
      END_STATE();
    case 373:
      if (lookahead == 'l') ADVANCE(314);
      END_STATE();
    case 374:
      if (lookahead == 'l') ADVANCE(312);
      END_STATE();
    case 375:
      if (lookahead == 'l') ADVANCE(313);
      END_STATE();
    case 376:
      if (lookahead == 'l') ADVANCE(375);
      END_STATE();
    case 377:
      if (lookahead == 'l') ADVANCE(90);
      END_STATE();
    case 378:
      if (lookahead == 'm') ADVANCE(43);
      END_STATE();
    case 379:
      if (lookahead == 'm') ADVANCE(489);
      END_STATE();
    case 380:
      if (lookahead == 'm') ADVANCE(151);
      END_STATE();
    case 381:
      if (lookahead == 'm') ADVANCE(78);
      END_STATE();
    case 382:
      if (lookahead == 'm') ADVANCE(154);
      END_STATE();
    case 383:
      if (lookahead == 'm') ADVANCE(204);
      END_STATE();
    case 384:
      if (lookahead == 'n') ADVANCE(670);
      END_STATE();
    case 385:
      if (lookahead == 'n') ADVANCE(778);
      END_STATE();
    case 386:
      if (lookahead == 'n') ADVANCE(242);
      END_STATE();
    case 387:
      if (lookahead == 'n') ADVANCE(104);
      END_STATE();
    case 388:
      if (lookahead == 'n') ADVANCE(719);
      END_STATE();
    case 389:
      if (lookahead == 'n') ADVANCE(732);
      END_STATE();
    case 390:
      if (lookahead == 'n') ADVANCE(722);
      END_STATE();
    case 391:
      if (lookahead == 'n') ADVANCE(725);
      END_STATE();
    case 392:
      if (lookahead == 'n') ADVANCE(734);
      END_STATE();
    case 393:
      if (lookahead == 'n') ADVANCE(723);
      END_STATE();
    case 394:
      if (lookahead == 'n') ADVANCE(739);
      END_STATE();
    case 395:
      if (lookahead == 'n') ADVANCE(716);
      END_STATE();
    case 396:
      if (lookahead == 'n') ADVANCE(710);
      END_STATE();
    case 397:
      if (lookahead == 'n') ADVANCE(735);
      END_STATE();
    case 398:
      if (lookahead == 'n') ADVANCE(737);
      END_STATE();
    case 399:
      if (lookahead == 'n') ADVANCE(738);
      END_STATE();
    case 400:
      if (lookahead == 'n') ADVANCE(556);
      END_STATE();
    case 401:
      if (lookahead == 'n') ADVANCE(260);
      END_STATE();
    case 402:
      if (lookahead == 'n') ADVANCE(320);
      END_STATE();
    case 403:
      if (lookahead == 'n') ADVANCE(256);
      END_STATE();
    case 404:
      if (lookahead == 'n') ADVANCE(588);
      END_STATE();
    case 405:
      if (lookahead == 'n') ADVANCE(146);
      END_STATE();
    case 406:
      if (lookahead == 'n') ADVANCE(261);
      END_STATE();
    case 407:
      if (lookahead == 'n') ADVANCE(142);
      END_STATE();
    case 408:
      if (lookahead == 'n') ADVANCE(451);
      END_STATE();
    case 409:
      if (lookahead == 'n') ADVANCE(246);
      END_STATE();
    case 410:
      if (lookahead == 'n') ADVANCE(148);
      END_STATE();
    case 411:
      if (lookahead == 'n') ADVANCE(247);
      END_STATE();
    case 412:
      if (lookahead == 'n') ADVANCE(582);
      END_STATE();
    case 413:
      if (lookahead == 'n') ADVANCE(113);
      END_STATE();
    case 414:
      if (lookahead == 'n') ADVANCE(248);
      END_STATE();
    case 415:
      if (lookahead == 'n') ADVANCE(123);
      END_STATE();
    case 416:
      if (lookahead == 'n') ADVANCE(249);
      END_STATE();
    case 417:
      if (lookahead == 'n') ADVANCE(544);
      END_STATE();
    case 418:
      if (lookahead == 'n') ADVANCE(250);
      END_STATE();
    case 419:
      if (lookahead == 'n') ADVANCE(209);
      END_STATE();
    case 420:
      if (lookahead == 'n') ADVANCE(251);
      END_STATE();
    case 421:
      if (lookahead == 'n') ADVANCE(252);
      END_STATE();
    case 422:
      if (lookahead == 'n') ADVANCE(365);
      END_STATE();
    case 423:
      if (lookahead == 'n') ADVANCE(253);
      END_STATE();
    case 424:
      if (lookahead == 'n') ADVANCE(562);
      END_STATE();
    case 425:
      if (lookahead == 'n') ADVANCE(611);
      END_STATE();
    case 426:
      if (lookahead == 'n') ADVANCE(226);
      END_STATE();
    case 427:
      if (lookahead == 'n') ADVANCE(157);
      END_STATE();
    case 428:
      if (lookahead == 'n') ADVANCE(194);
      END_STATE();
    case 429:
      if (lookahead == 'n') ADVANCE(115);
      END_STATE();
    case 430:
      if (lookahead == 'n') ADVANCE(302);
      END_STATE();
    case 431:
      if (lookahead == 'n') ADVANCE(257);
      END_STATE();
    case 432:
      if (lookahead == 'n') ADVANCE(298);
      END_STATE();
    case 433:
      if (lookahead == 'n') ADVANCE(214);
      END_STATE();
    case 434:
      if (lookahead == 'n') ADVANCE(125);
      END_STATE();
    case 435:
      if (lookahead == 'n') ADVANCE(216);
      END_STATE();
    case 436:
      if (lookahead == 'n') ADVANCE(309);
      END_STATE();
    case 437:
      if (lookahead == 'n') ADVANCE(219);
      END_STATE();
    case 438:
      if (lookahead == 'n') ADVANCE(316);
      END_STATE();
    case 439:
      if (lookahead == 'n') ADVANCE(147);
      END_STATE();
    case 440:
      if (lookahead == 'n') ADVANCE(103);
      END_STATE();
    case 441:
      if (lookahead == 'n') ADVANCE(220);
      END_STATE();
    case 442:
      if (lookahead == 'n') ADVANCE(223);
      END_STATE();
    case 443:
      if (lookahead == 'n') ADVANCE(241);
      END_STATE();
    case 444:
      if (lookahead == 'n') ADVANCE(267);
      END_STATE();
    case 445:
      if (lookahead == 'o') ADVANCE(295);
      if (lookahead == 'r') ADVANCE(446);
      if (lookahead == 'u') ADVANCE(429);
      END_STATE();
    case 446:
      if (lookahead == 'o') ADVANCE(345);
      END_STATE();
    case 447:
      if (lookahead == 'o') ADVANCE(292);
      END_STATE();
    case 448:
      if (lookahead == 'o') ADVANCE(660);
      END_STATE();
    case 449:
      if (lookahead == 'o') ADVANCE(385);
      END_STATE();
    case 450:
      if (lookahead == 'o') ADVANCE(656);
      END_STATE();
    case 451:
      if (lookahead == 'o') ADVANCE(110);
      END_STATE();
    case 452:
      if (lookahead == 'o') ADVANCE(659);
      END_STATE();
    case 453:
      if (lookahead == 'o') ADVANCE(361);
      END_STATE();
    case 454:
      if (lookahead == 'o') ADVANCE(462);
      if (lookahead == 'u') ADVANCE(120);
      END_STATE();
    case 455:
      if (lookahead == 'o') ADVANCE(234);
      END_STATE();
    case 456:
      if (lookahead == 'o') ADVANCE(239);
      END_STATE();
    case 457:
      if (lookahead == 'o') ADVANCE(520);
      END_STATE();
    case 458:
      if (lookahead == 'o') ADVANCE(530);
      END_STATE();
    case 459:
      if (lookahead == 'o') ADVANCE(144);
      END_STATE();
    case 460:
      if (lookahead == 'o') ADVANCE(407);
      END_STATE();
    case 461:
      if (lookahead == 'o') ADVANCE(645);
      END_STATE();
    case 462:
      if (lookahead == 'o') ADVANCE(622);
      END_STATE();
    case 463:
      if (lookahead == 'o') ADVANCE(294);
      END_STATE();
    case 464:
      if (lookahead == 'o') ADVANCE(388);
      END_STATE();
    case 465:
      if (lookahead == 'o') ADVANCE(390);
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
      if (lookahead == 'o') ADVANCE(633);
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
      if (lookahead == 'o') ADVANCE(657);
      if (lookahead == 'r') ADVANCE(446);
      if (lookahead == 'u') ADVANCE(429);
      END_STATE();
    case 477:
      if (lookahead == 'o') ADVANCE(494);
      END_STATE();
    case 478:
      if (lookahead == 'o') ADVANCE(383);
      END_STATE();
    case 479:
      if (lookahead == 'o') ADVANCE(25);
      END_STATE();
    case 480:
      if (lookahead == 'o') ADVANCE(560);
      END_STATE();
    case 481:
      if (lookahead == 'o') ADVANCE(566);
      END_STATE();
    case 482:
      if (lookahead == 'o') ADVANCE(567);
      END_STATE();
    case 483:
      if (lookahead == 'o') ADVANCE(481);
      END_STATE();
    case 484:
      if (lookahead == 'o') ADVANCE(482);
      END_STATE();
    case 485:
      if (lookahead == 'o') ADVANCE(243);
      END_STATE();
    case 486:
      if (lookahead == 'o') ADVANCE(634);
      END_STATE();
    case 487:
      if (lookahead == 'o') ADVANCE(635);
      END_STATE();
    case 488:
      if (lookahead == 'p') ADVANCE(44);
      END_STATE();
    case 489:
      if (lookahead == 'p') ADVANCE(10);
      END_STATE();
    case 490:
      if (lookahead == 'p') ADVANCE(284);
      END_STATE();
    case 491:
      if (lookahead == 'p') ADVANCE(344);
      END_STATE();
    case 492:
      if (lookahead == 'p') ADVANCE(594);
      END_STATE();
    case 493:
      if (lookahead == 'p') ADVANCE(452);
      END_STATE();
    case 494:
      if (lookahead == 'p') ADVANCE(459);
      END_STATE();
    case 495:
      if (lookahead == 'p') ADVANCE(603);
      END_STATE();
    case 496:
      if (lookahead == 'p') ADVANCE(206);
      END_STATE();
    case 497:
      if (lookahead == 'p') ADVANCE(217);
      END_STATE();
    case 498:
      if (lookahead == 'p') ADVANCE(307);
      END_STATE();
    case 499:
      if (lookahead == 'p') ADVANCE(437);
      END_STATE();
    case 500:
      if (lookahead == 'p') ADVANCE(540);
      END_STATE();
    case 501:
      if (lookahead == 'p') ADVANCE(541);
      END_STATE();
    case 502:
      if (lookahead == 'q') ADVANCE(640);
      END_STATE();
    case 503:
      if (lookahead == 'r') ADVANCE(693);
      END_STATE();
    case 504:
      if (lookahead == 'r') ADVANCE(694);
      END_STATE();
    case 505:
      if (lookahead == 'r') ADVANCE(756);
      END_STATE();
    case 506:
      if (lookahead == 'r') ADVANCE(717);
      END_STATE();
    case 507:
      if (lookahead == 'r') ADVANCE(720);
      END_STATE();
    case 508:
      if (lookahead == 'r') ADVANCE(743);
      END_STATE();
    case 509:
      if (lookahead == 'r') ADVANCE(730);
      END_STATE();
    case 510:
      if (lookahead == 'r') ADVANCE(742);
      END_STATE();
    case 511:
      if (lookahead == 'r') ADVANCE(663);
      END_STATE();
    case 512:
      if (lookahead == 'r') ADVANCE(45);
      END_STATE();
    case 513:
      if (lookahead == 'r') ADVANCE(664);
      END_STATE();
    case 514:
      if (lookahead == 'r') ADVANCE(51);
      END_STATE();
    case 515:
      if (lookahead == 'r') ADVANCE(499);
      END_STATE();
    case 516:
      if (lookahead == 'r') ADVANCE(14);
      END_STATE();
    case 517:
      if (lookahead == 'r') ADVANCE(175);
      END_STATE();
    case 518:
      if (lookahead == 'r') ADVANCE(598);
      END_STATE();
    case 519:
      if (lookahead == 'r') ADVANCE(297);
      END_STATE();
    case 520:
      if (lookahead == 'r') ADVANCE(417);
      END_STATE();
    case 521:
      if (lookahead == 'r') ADVANCE(191);
      END_STATE();
    case 522:
      if (lookahead == 'r') ADVANCE(477);
      END_STATE();
    case 523:
      if (lookahead == 'r') ADVANCE(85);
      END_STATE();
    case 524:
      if (lookahead == 'r') ADVANCE(188);
      END_STATE();
    case 525:
      if (lookahead == 'r') ADVANCE(212);
      END_STATE();
    case 526:
      if (lookahead == 'r') ADVANCE(604);
      END_STATE();
    case 527:
      if (lookahead == 'r') ADVANCE(218);
      END_STATE();
    case 528:
      if (lookahead == 'r') ADVANCE(199);
      END_STATE();
    case 529:
      if (lookahead == 'r') ADVANCE(202);
      END_STATE();
    case 530:
      if (lookahead == 'r') ADVANCE(495);
      END_STATE();
    case 531:
      if (lookahead == 'r') ADVANCE(89);
      END_STATE();
    case 532:
      if (lookahead == 'r') ADVANCE(568);
      END_STATE();
    case 533:
      if (lookahead == 'r') ADVANCE(469);
      END_STATE();
    case 534:
      if (lookahead == 'r') ADVANCE(38);
      END_STATE();
    case 535:
      if (lookahead == 'r') ADVANCE(570);
      END_STATE();
    case 536:
      if (lookahead == 'r') ADVANCE(92);
      END_STATE();
    case 537:
      if (lookahead == 'r') ADVANCE(95);
      END_STATE();
    case 538:
      if (lookahead == 'r') ADVANCE(96);
      END_STATE();
    case 539:
      if (lookahead == 'r') ADVANCE(97);
      END_STATE();
    case 540:
      if (lookahead == 'r') ADVANCE(486);
      END_STATE();
    case 541:
      if (lookahead == 'r') ADVANCE(487);
      END_STATE();
    case 542:
      if (lookahead == 's') ADVANCE(685);
      END_STATE();
    case 543:
      if (lookahead == 's') ADVANCE(686);
      END_STATE();
    case 544:
      if (lookahead == 's') ADVANCE(741);
      END_STATE();
    case 545:
      if (lookahead == 's') ADVANCE(702);
      END_STATE();
    case 546:
      if (lookahead == 's') ADVANCE(718);
      END_STATE();
    case 547:
      if (lookahead == 's') ADVANCE(715);
      END_STATE();
    case 548:
      if (lookahead == 's') ADVANCE(745);
      END_STATE();
    case 549:
      if (lookahead == 's') ADVANCE(747);
      END_STATE();
    case 550:
      if (lookahead == 's') ADVANCE(496);
      END_STATE();
    case 551:
      if (lookahead == 's') ADVANCE(497);
      END_STATE();
    case 552:
      if (lookahead == 's') ADVANCE(291);
      END_STATE();
    case 553:
      if (lookahead == 's') ADVANCE(458);
      END_STATE();
    case 554:
      if (lookahead == 's') ADVANCE(491);
      END_STATE();
    case 555:
      if (lookahead == 's') ADVANCE(334);
      END_STATE();
    case 556:
      if (lookahead == 's') ADVANCE(623);
      if (lookahead == 'v') ADVANCE(296);
      END_STATE();
    case 557:
      if (lookahead == 's') ADVANCE(545);
      END_STATE();
    case 558:
      if (lookahead == 's') ADVANCE(596);
      END_STATE();
    case 559:
      if (lookahead == 's') ADVANCE(546);
      END_STATE();
    case 560:
      if (lookahead == 's') ADVANCE(600);
      END_STATE();
    case 561:
      if (lookahead == 's') ADVANCE(547);
      END_STATE();
    case 562:
      if (lookahead == 's') ADVANCE(18);
      END_STATE();
    case 563:
      if (lookahead == 's') ADVANCE(548);
      END_STATE();
    case 564:
      if (lookahead == 's') ADVANCE(185);
      END_STATE();
    case 565:
      if (lookahead == 's') ADVANCE(607);
      END_STATE();
    case 566:
      if (lookahead == 's') ADVANCE(585);
      END_STATE();
    case 567:
      if (lookahead == 's') ADVANCE(587);
      END_STATE();
    case 568:
      if (lookahead == 's') ADVANCE(159);
      END_STATE();
    case 569:
      if (lookahead == 's') ADVANCE(203);
      END_STATE();
    case 570:
      if (lookahead == 's') ADVANCE(164);
      END_STATE();
    case 571:
      if (lookahead == 's') ADVANCE(613);
      END_STATE();
    case 572:
      if (lookahead == 's') ADVANCE(615);
      END_STATE();
    case 573:
      if (lookahead == 's') ADVANCE(464);
      END_STATE();
    case 574:
      if (lookahead == 's') ADVANCE(619);
      END_STATE();
    case 575:
      if (lookahead == 's') ADVANCE(428);
      END_STATE();
    case 576:
      if (lookahead == 's') ADVANCE(287);
      END_STATE();
    case 577:
      if (lookahead == 's') ADVANCE(336);
      END_STATE();
    case 578:
      if (lookahead == 't') ADVANCE(770);
      END_STATE();
    case 579:
      if (lookahead == 't') ADVANCE(777);
      END_STATE();
    case 580:
      if (lookahead == 't') ADVANCE(689);
      END_STATE();
    case 581:
      if (lookahead == 't') ADVANCE(696);
      END_STATE();
    case 582:
      if (lookahead == 't') ADVANCE(700);
      END_STATE();
    case 583:
      if (lookahead == 't') ADVANCE(779);
      END_STATE();
    case 584:
      if (lookahead == 't') ADVANCE(6);
      END_STATE();
    case 585:
      if (lookahead == 't') ADVANCE(708);
      END_STATE();
    case 586:
      if (lookahead == 't') ADVANCE(749);
      END_STATE();
    case 587:
      if (lookahead == 't') ADVANCE(721);
      END_STATE();
    case 588:
      if (lookahead == 't') ADVANCE(543);
      END_STATE();
    case 589:
      if (lookahead == 't') ADVANCE(7);
      END_STATE();
    case 590:
      if (lookahead == 't') ADVANCE(8);
      END_STATE();
    case 591:
      if (lookahead == 't') ADVANCE(665);
      END_STATE();
    case 592:
      if (lookahead == 't') ADVANCE(279);
      END_STATE();
    case 593:
      if (lookahead == 't') ADVANCE(646);
      END_STATE();
    case 594:
      if (lookahead == 't') ADVANCE(275);
      END_STATE();
    case 595:
      if (lookahead == 't') ADVANCE(667);
      END_STATE();
    case 596:
      if (lookahead == 't') ADVANCE(17);
      END_STATE();
    case 597:
      if (lookahead == 't') ADVANCE(668);
      END_STATE();
    case 598:
      if (lookahead == 't') ADVANCE(648);
      END_STATE();
    case 599:
      if (lookahead == 't') ADVANCE(286);
      END_STATE();
    case 600:
      if (lookahead == 't') ADVANCE(13);
      END_STATE();
    case 601:
      if (lookahead == 't') ADVANCE(269);
      END_STATE();
    case 602:
      if (lookahead == 't') ADVANCE(27);
      END_STATE();
    case 603:
      if (lookahead == 't') ADVANCE(327);
      END_STATE();
    case 604:
      if (lookahead == 't') ADVANCE(277);
      END_STATE();
    case 605:
      if (lookahead == 't') ADVANCE(319);
      END_STATE();
    case 606:
      if (lookahead == 't') ADVANCE(271);
      END_STATE();
    case 607:
      if (lookahead == 't') ADVANCE(519);
      END_STATE();
    case 608:
      if (lookahead == 't') ADVANCE(20);
      END_STATE();
    case 609:
      if (lookahead == 't') ADVANCE(299);
      END_STATE();
    case 610:
      if (lookahead == 't') ADVANCE(461);
      END_STATE();
    case 611:
      if (lookahead == 't') ADVANCE(11);
      END_STATE();
    case 612:
      if (lookahead == 't') ADVANCE(184);
      END_STATE();
    case 613:
      if (lookahead == 't') ADVANCE(155);
      END_STATE();
    case 614:
      if (lookahead == 't') ADVANCE(156);
      END_STATE();
    case 615:
      if (lookahead == 't') ADVANCE(72);
      END_STATE();
    case 616:
      if (lookahead == 't') ADVANCE(282);
      END_STATE();
    case 617:
      if (lookahead == 't') ADVANCE(24);
      END_STATE();
    case 618:
      if (lookahead == 't') ADVANCE(93);
      END_STATE();
    case 619:
      if (lookahead == 't') ADVANCE(87);
      END_STATE();
    case 620:
      if (lookahead == 't') ADVANCE(280);
      END_STATE();
    case 621:
      if (lookahead == 't') ADVANCE(281);
      END_STATE();
    case 622:
      if (lookahead == 't') ADVANCE(304);
      END_STATE();
    case 623:
      if (lookahead == 't') ADVANCE(82);
      END_STATE();
    case 624:
      if (lookahead == 't') ADVANCE(330);
      END_STATE();
    case 625:
      if (lookahead == 't') ADVANCE(285);
      END_STATE();
    case 626:
      if (lookahead == 't') ADVANCE(332);
      END_STATE();
    case 627:
      if (lookahead == 't') ADVANCE(333);
      END_STATE();
    case 628:
      if (lookahead == 't') ADVANCE(335);
      END_STATE();
    case 629:
      if (lookahead == 't') ADVANCE(337);
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
      if (lookahead == 't') ADVANCE(224);
      END_STATE();
    case 634:
      if (lookahead == 't') ADVANCE(225);
      END_STATE();
    case 635:
      if (lookahead == 't') ADVANCE(227);
      END_STATE();
    case 636:
      if (lookahead == 'u') ADVANCE(379);
      END_STATE();
    case 637:
      if (lookahead == 'u') ADVANCE(149);
      END_STATE();
    case 638:
      if (lookahead == 'u') ADVANCE(564);
      END_STATE();
    case 639:
      if (lookahead == 'u') ADVANCE(364);
      END_STATE();
    case 640:
      if (lookahead == 'u') ADVANCE(54);
      END_STATE();
    case 641:
      if (lookahead == 'u') ADVANCE(532);
      END_STATE();
    case 642:
      if (lookahead == 'u') ADVANCE(162);
      END_STATE();
    case 643:
      if (lookahead == 'u') ADVANCE(208);
      END_STATE();
    case 644:
      if (lookahead == 'u') ADVANCE(112);
      END_STATE();
    case 645:
      if (lookahead == 'u') ADVANCE(119);
      END_STATE();
    case 646:
      if (lookahead == 'u') ADVANCE(537);
      END_STATE();
    case 647:
      if (lookahead == 'u') ADVANCE(308);
      END_STATE();
    case 648:
      if (lookahead == 'u') ADVANCE(427);
      END_STATE();
    case 649:
      if (lookahead == 'u') ADVANCE(535);
      END_STATE();
    case 650:
      if (lookahead == 'v') ADVANCE(150);
      END_STATE();
    case 651:
      if (lookahead == 'v') ADVANCE(176);
      END_STATE();
    case 652:
      if (lookahead == 'v') ADVANCE(323);
      END_STATE();
    case 653:
      if (lookahead == 'v') ADVANCE(305);
      END_STATE();
    case 654:
      if (lookahead == 'v') ADVANCE(324);
      END_STATE();
    case 655:
      if (lookahead == 'w') ADVANCE(695);
      END_STATE();
    case 656:
      if (lookahead == 'w') ADVANCE(15);
      END_STATE();
    case 657:
      if (lookahead == 'w') ADVANCE(183);
      END_STATE();
    case 658:
      if (lookahead == 'w') ADVANCE(67);
      END_STATE();
    case 659:
      if (lookahead == 'w') ADVANCE(198);
      END_STATE();
    case 660:
      if (lookahead == 'w') ADVANCE(303);
      END_STATE();
    case 661:
      if (lookahead == 'y') ADVANCE(776);
      END_STATE();
    case 662:
      if (lookahead == 'y') ADVANCE(692);
      END_STATE();
    case 663:
      if (lookahead == 'y') ADVANCE(771);
      END_STATE();
    case 664:
      if (lookahead == 'y') ADVANCE(690);
      END_STATE();
    case 665:
      if (lookahead == 'y') ADVANCE(759);
      END_STATE();
    case 666:
      if (lookahead == 'y') ADVANCE(752);
      END_STATE();
    case 667:
      if (lookahead == 'y') ADVANCE(714);
      END_STATE();
    case 668:
      if (lookahead == 'y') ADVANCE(740);
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
          lookahead != ';') ADVANCE(768);
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
      ACCEPT_TOKEN(anon_sym_give);
      END_STATE();
    case 698:
      ACCEPT_TOKEN(anon_sym_minecraft_COLON);
      ADVANCE_MAP(
        'a', 99,
        'b', 52,
        'c', 460,
        'd', 169,
        'e', 230,
        'f', 180,
        'g', 363,
        'h', 59,
        'i', 386,
        'j', 636,
        'k', 408,
        'l', 171,
        'm', 321,
        'n', 57,
        'p', 445,
        'r', 152,
        's', 64,
        't', 276,
        'u', 387,
        'v', 88,
        'w', 68,
      );
      END_STATE();
    case 699:
      ACCEPT_TOKEN(anon_sym_minecraft_COLON);
      ADVANCE_MAP(
        'a', 98,
        'b', 69,
        'c', 460,
        'd', 453,
        'f', 331,
        'g', 363,
        'h', 59,
        'i', 400,
        'j', 636,
        'l', 172,
        'm', 321,
        'n', 57,
        'p', 463,
        'r', 173,
        's', 65,
        'u', 422,
        'w', 68,
      );
      END_STATE();
    case 700:
      ACCEPT_TOKEN(anon_sym_enchant);
      END_STATE();
    case 701:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONspeed);
      END_STATE();
    case 702:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslowness);
      END_STATE();
    case 703:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhaste);
      END_STATE();
    case 704:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmining_fatigue);
      END_STATE();
    case 705:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONstrength);
      END_STATE();
    case 706:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_health);
      END_STATE();
    case 707:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_damage);
      END_STATE();
    case 708:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONjump_boost);
      END_STATE();
    case 709:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnausea);
      END_STATE();
    case 710:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONregeneration);
      END_STATE();
    case 711:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONresistance);
      END_STATE();
    case 712:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_resistance);
      END_STATE();
    case 713:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwater_breathing);
      END_STATE();
    case 714:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinvisibility);
      END_STATE();
    case 715:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblindness);
      END_STATE();
    case 716:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnight_vision);
      END_STATE();
    case 717:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhunger);
      END_STATE();
    case 718:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONweakness);
      END_STATE();
    case 719:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpoison);
      END_STATE();
    case 720:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwither);
      END_STATE();
    case 721:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhealth_boost);
      END_STATE();
    case 722:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONabsorption);
      END_STATE();
    case 723:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsaturation);
      END_STATE();
    case 724:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONglowing);
      END_STATE();
    case 725:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlevitation);
      END_STATE();
    case 726:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      END_STATE();
    case 727:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      if (lookahead == '_') ADVANCE(485);
      END_STATE();
    case 728:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunluck);
      END_STATE();
    case 729:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslow_falling);
      END_STATE();
    case 730:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONconduit_power);
      END_STATE();
    case 731:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdolphins_grace);
      END_STATE();
    case 732:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbad_omen);
      END_STATE();
    case 733:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhero_of_the_village);
      END_STATE();
    case 734:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONprotection);
      END_STATE();
    case 735:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_protection);
      END_STATE();
    case 736:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfeather_falling);
      END_STATE();
    case 737:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblast_protection);
      END_STATE();
    case 738:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONprojectile_protection);
      END_STATE();
    case 739:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONrespiration);
      END_STATE();
    case 740:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONaqua_affinity);
      END_STATE();
    case 741:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONthorns);
      END_STATE();
    case 742:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdepth_strider);
      END_STATE();
    case 743:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfrost_walker);
      END_STATE();
    case 744:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbinding_curse);
      END_STATE();
    case 745:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsharpness);
      END_STATE();
    case 746:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsmite);
      END_STATE();
    case 747:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbane_of_arthropods);
      END_STATE();
    case 748:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONknockback);
      END_STATE();
    case 749:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_aspect);
      END_STATE();
    case 750:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlooting);
      END_STATE();
    case 751:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsweeping);
      END_STATE();
    case 752:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONefficiency);
      END_STATE();
    case 753:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsilk_touch);
      END_STATE();
    case 754:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunbreaking);
      END_STATE();
    case 755:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfortune);
      END_STATE();
    case 756:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpower);
      END_STATE();
    case 757:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpunch);
      END_STATE();
    case 758:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONflame);
      END_STATE();
    case 759:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinfinity);
      END_STATE();
    case 760:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck_of_the_sea);
      END_STATE();
    case 761:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlure);
      END_STATE();
    case 762:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmending);
      END_STATE();
    case 763:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONvanishing_curse);
      END_STATE();
    case 764:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsoul_speed);
      END_STATE();
    case 765:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONswift_sneak);
      END_STATE();
    case 766:
      ACCEPT_TOKEN(sym_text);
      if (lookahead == '[') ADVANCE(681);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(766);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(768);
      END_STATE();
    case 767:
      ACCEPT_TOKEN(sym_text);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(767);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(768);
      END_STATE();
    case 768:
      ACCEPT_TOKEN(sym_text);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(768);
      END_STATE();
    case 769:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 770:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 771:
      ACCEPT_TOKEN(anon_sym_query);
      END_STATE();
    case 772:
      ACCEPT_TOKEN(anon_sym_gms);
      if (lookahead == 'p') ADVANCE(774);
      END_STATE();
    case 773:
      ACCEPT_TOKEN(anon_sym_gma);
      END_STATE();
    case 774:
      ACCEPT_TOKEN(anon_sym_gmsp);
      END_STATE();
    case 775:
      ACCEPT_TOKEN(anon_sym_gmc);
      END_STATE();
    case 776:
      ACCEPT_TOKEN(anon_sym_day);
      END_STATE();
    case 777:
      ACCEPT_TOKEN(anon_sym_night);
      END_STATE();
    case 778:
      ACCEPT_TOKEN(anon_sym_noon);
      END_STATE();
    case 779:
      ACCEPT_TOKEN(anon_sym_midnight);
      END_STATE();
    case 780:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(780);
      END_STATE();
    case 781:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(781);
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
  [7] = {.lex_state = 2},
  [8] = {.lex_state = 2},
  [9] = {.lex_state = 3},
  [10] = {.lex_state = 3},
  [11] = {.lex_state = 3},
  [12] = {.lex_state = 3},
  [13] = {.lex_state = 2},
  [14] = {.lex_state = 2},
  [15] = {.lex_state = 3},
  [16] = {.lex_state = 3},
  [17] = {.lex_state = 107},
  [18] = {.lex_state = 107},
  [19] = {.lex_state = 107},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 107},
  [22] = {.lex_state = 107},
  [23] = {.lex_state = 0},
  [24] = {.lex_state = 0},
  [25] = {.lex_state = 0},
  [26] = {.lex_state = 0},
  [27] = {.lex_state = 0},
  [28] = {.lex_state = 0},
  [29] = {.lex_state = 0},
  [30] = {.lex_state = 0},
  [31] = {.lex_state = 0},
  [32] = {.lex_state = 0},
  [33] = {.lex_state = 0},
  [34] = {.lex_state = 0},
  [35] = {.lex_state = 0},
  [36] = {.lex_state = 1},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 1},
  [41] = {.lex_state = 1},
  [42] = {.lex_state = 1},
  [43] = {.lex_state = 1},
  [44] = {.lex_state = 1},
  [45] = {.lex_state = 1},
  [46] = {.lex_state = 1},
  [47] = {.lex_state = 1},
  [48] = {.lex_state = 1},
  [49] = {.lex_state = 0},
  [50] = {.lex_state = 1},
  [51] = {.lex_state = 1},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 1},
  [54] = {.lex_state = 1},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 766},
  [57] = {.lex_state = 1},
  [58] = {.lex_state = 766},
  [59] = {.lex_state = 0},
  [60] = {.lex_state = 1},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 0},
  [63] = {.lex_state = 0},
  [64] = {.lex_state = 0},
  [65] = {.lex_state = 0},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 0},
  [69] = {.lex_state = 0},
  [70] = {.lex_state = 0},
  [71] = {.lex_state = 767},
  [72] = {.lex_state = 0},
  [73] = {.lex_state = 767},
  [74] = {.lex_state = 0},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 1},
  [79] = {.lex_state = 0},
  [80] = {.lex_state = 0},
  [81] = {.lex_state = 1},
  [82] = {.lex_state = 0},
  [83] = {.lex_state = 0},
  [84] = {.lex_state = 0},
  [85] = {.lex_state = 0},
  [86] = {.lex_state = 0},
  [87] = {.lex_state = 0},
  [88] = {.lex_state = 0},
  [89] = {.lex_state = 0},
  [90] = {.lex_state = 0},
  [91] = {.lex_state = 0},
  [92] = {.lex_state = 0},
  [93] = {.lex_state = 0},
  [94] = {.lex_state = 0},
  [95] = {.lex_state = 0},
  [96] = {.lex_state = 0},
  [97] = {.lex_state = 0},
  [98] = {.lex_state = 0},
  [99] = {.lex_state = 767},
  [100] = {.lex_state = 767},
  [101] = {.lex_state = 767},
  [102] = {.lex_state = 0},
  [103] = {.lex_state = 0},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 0},
  [108] = {.lex_state = 0},
  [109] = {.lex_state = 0},
  [110] = {.lex_state = 0},
  [111] = {.lex_state = 0},
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
    [anon_sym_give] = ACTIONS(1),
    [anon_sym_minecraft_COLON] = ACTIONS(1),
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
    [sym_source_file] = STATE(72),
    [sym__definition] = STATE(39),
    [sym_function_definition] = STATE(39),
    [aux_sym_source_file_repeat1] = STATE(39),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_fn] = ACTIONS(5),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 4,
    ACTIONS(9), 1,
      anon_sym_LBRACK,
    ACTIONS(11), 1,
      anon_sym_minecraft_COLON,
    STATE(5), 1,
      sym_selector_arguments,
    ACTIONS(7), 36,
      anon_sym_SEMI,
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
  [48] = 2,
    ACTIONS(15), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(13), 37,
      anon_sym_SEMI,
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
  [91] = 2,
    ACTIONS(19), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(17), 36,
      anon_sym_SEMI,
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
  [133] = 2,
    ACTIONS(23), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(21), 36,
      anon_sym_SEMI,
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
    ACTIONS(27), 1,
      anon_sym_minecraft_COLON,
    ACTIONS(25), 36,
      anon_sym_SEMI,
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
  [217] = 4,
    ACTIONS(29), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(31), 1,
      anon_sym_minecraft_COLON,
    STATE(74), 2,
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
  [262] = 4,
    ACTIONS(29), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(35), 1,
      anon_sym_minecraft_COLON,
    STATE(85), 2,
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
  [307] = 3,
    ACTIONS(37), 1,
      anon_sym_LBRACK,
    STATE(15), 1,
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
  [349] = 3,
    ACTIONS(39), 1,
      aux_sym_quoted_string_token1,
    STATE(96), 2,
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
  [391] = 1,
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
  [428] = 1,
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
  [464] = 2,
    STATE(75), 1,
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
  [502] = 2,
    STATE(65), 1,
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
  [540] = 1,
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
  [576] = 1,
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
  [612] = 18,
    ACTIONS(43), 1,
      aux_sym_block_token1,
    ACTIONS(46), 1,
      anon_sym_RBRACE,
    ACTIONS(48), 1,
      anon_sym_xpadd,
    ACTIONS(51), 1,
      anon_sym_xpset,
    ACTIONS(54), 1,
      anon_sym_xpquery,
    ACTIONS(57), 1,
      anon_sym_say,
    ACTIONS(60), 1,
      anon_sym_clear,
    ACTIONS(63), 1,
      anon_sym_eclear,
    ACTIONS(66), 1,
      anon_sym_tellraw,
    ACTIONS(69), 1,
      anon_sym_effect,
    ACTIONS(72), 1,
      anon_sym_enchant,
    ACTIONS(75), 1,
      anon_sym_time,
    ACTIONS(78), 1,
      anon_sym_gms,
    ACTIONS(81), 1,
      anon_sym_gma,
    ACTIONS(84), 1,
      anon_sym_gmsp,
    ACTIONS(87), 1,
      anon_sym_gmc,
    STATE(17), 1,
      aux_sym_block_repeat1,
    STATE(80), 15,
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
  [681] = 18,
    ACTIONS(90), 1,
      aux_sym_block_token1,
    ACTIONS(92), 1,
      anon_sym_RBRACE,
    ACTIONS(94), 1,
      anon_sym_xpadd,
    ACTIONS(96), 1,
      anon_sym_xpset,
    ACTIONS(98), 1,
      anon_sym_xpquery,
    ACTIONS(100), 1,
      anon_sym_say,
    ACTIONS(102), 1,
      anon_sym_clear,
    ACTIONS(104), 1,
      anon_sym_eclear,
    ACTIONS(106), 1,
      anon_sym_tellraw,
    ACTIONS(108), 1,
      anon_sym_effect,
    ACTIONS(110), 1,
      anon_sym_enchant,
    ACTIONS(112), 1,
      anon_sym_time,
    ACTIONS(114), 1,
      anon_sym_gms,
    ACTIONS(116), 1,
      anon_sym_gma,
    ACTIONS(118), 1,
      anon_sym_gmsp,
    ACTIONS(120), 1,
      anon_sym_gmc,
    STATE(17), 1,
      aux_sym_block_repeat1,
    STATE(80), 15,
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
  [750] = 18,
    ACTIONS(90), 1,
      aux_sym_block_token1,
    ACTIONS(94), 1,
      anon_sym_xpadd,
    ACTIONS(96), 1,
      anon_sym_xpset,
    ACTIONS(98), 1,
      anon_sym_xpquery,
    ACTIONS(100), 1,
      anon_sym_say,
    ACTIONS(102), 1,
      anon_sym_clear,
    ACTIONS(104), 1,
      anon_sym_eclear,
    ACTIONS(106), 1,
      anon_sym_tellraw,
    ACTIONS(108), 1,
      anon_sym_effect,
    ACTIONS(110), 1,
      anon_sym_enchant,
    ACTIONS(112), 1,
      anon_sym_time,
    ACTIONS(114), 1,
      anon_sym_gms,
    ACTIONS(116), 1,
      anon_sym_gma,
    ACTIONS(118), 1,
      anon_sym_gmsp,
    ACTIONS(120), 1,
      anon_sym_gmc,
    ACTIONS(122), 1,
      anon_sym_RBRACE,
    STATE(18), 1,
      aux_sym_block_repeat1,
    STATE(80), 15,
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
  [819] = 15,
    ACTIONS(114), 1,
      anon_sym_gms,
    ACTIONS(124), 1,
      anon_sym_xpadd,
    ACTIONS(126), 1,
      anon_sym_xpset,
    ACTIONS(128), 1,
      anon_sym_xpquery,
    ACTIONS(130), 1,
      anon_sym_say,
    ACTIONS(132), 1,
      anon_sym_clear,
    ACTIONS(134), 1,
      anon_sym_eclear,
    ACTIONS(136), 1,
      anon_sym_tellraw,
    ACTIONS(138), 1,
      anon_sym_effect,
    ACTIONS(140), 1,
      anon_sym_enchant,
    ACTIONS(142), 1,
      anon_sym_time,
    ACTIONS(144), 1,
      anon_sym_gma,
    ACTIONS(146), 1,
      anon_sym_gmsp,
    ACTIONS(148), 1,
      anon_sym_gmc,
    STATE(97), 14,
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
  [878] = 2,
    ACTIONS(150), 1,
      aux_sym_block_token1,
    ACTIONS(152), 15,
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
  [899] = 2,
    ACTIONS(154), 1,
      aux_sym_block_token1,
    ACTIONS(46), 15,
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
  [920] = 4,
    ACTIONS(159), 1,
      anon_sym_give,
    STATE(2), 1,
      sym_selector_type,
    STATE(8), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [937] = 3,
    STATE(9), 1,
      sym_selector_type,
    STATE(10), 1,
      sym_target_selector,
    ACTIONS(161), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [951] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(52), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [965] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(66), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [979] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(69), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [993] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(93), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1007] = 3,
    STATE(56), 1,
      sym_selector_type,
    STATE(71), 1,
      sym_target_selector,
    ACTIONS(163), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1021] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(7), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1035] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(55), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1049] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(87), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1063] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(88), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1077] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(49), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1091] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(89), 1,
      sym_target_selector,
    ACTIONS(157), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [1105] = 5,
    ACTIONS(165), 1,
      anon_sym_DOT_DOT,
    ACTIONS(167), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(169), 1,
      sym_identifier,
    ACTIONS(171), 1,
      sym_number,
    STATE(51), 2,
      sym_range_value,
      sym_quoted_string,
  [1122] = 3,
    ACTIONS(175), 1,
      sym_number,
    STATE(102), 1,
      sym_time_unit,
    ACTIONS(173), 4,
      anon_sym_day,
      anon_sym_night,
      anon_sym_noon,
      anon_sym_midnight,
  [1135] = 3,
    ACTIONS(177), 1,
      ts_builtin_sym_end,
    ACTIONS(179), 1,
      anon_sym_fn,
    STATE(38), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [1147] = 3,
    ACTIONS(5), 1,
      anon_sym_fn,
    ACTIONS(182), 1,
      ts_builtin_sym_end,
    STATE(38), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [1159] = 2,
    ACTIONS(186), 1,
      anon_sym_DOT_DOT,
    ACTIONS(184), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [1168] = 4,
    ACTIONS(188), 1,
      anon_sym_RBRACK,
    ACTIONS(190), 1,
      sym_identifier,
    STATE(41), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1181] = 2,
    ACTIONS(195), 1,
      sym_number,
    ACTIONS(193), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [1190] = 4,
    ACTIONS(197), 1,
      anon_sym_RBRACK,
    ACTIONS(199), 1,
      sym_identifier,
    STATE(45), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1203] = 4,
    ACTIONS(199), 1,
      sym_identifier,
    ACTIONS(201), 1,
      anon_sym_RBRACK,
    STATE(41), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1216] = 4,
    ACTIONS(199), 1,
      sym_identifier,
    ACTIONS(203), 1,
      anon_sym_RBRACK,
    STATE(41), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1229] = 4,
    ACTIONS(199), 1,
      sym_identifier,
    ACTIONS(205), 1,
      anon_sym_RBRACK,
    STATE(44), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1242] = 4,
    ACTIONS(199), 1,
      sym_identifier,
    ACTIONS(207), 1,
      anon_sym_RBRACK,
    STATE(48), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1255] = 4,
    ACTIONS(199), 1,
      sym_identifier,
    ACTIONS(209), 1,
      anon_sym_RBRACK,
    STATE(41), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(57), 1,
      sym_selector_argument,
  [1268] = 2,
    STATE(104), 1,
      sym_xp_type,
    ACTIONS(211), 2,
      anon_sym_levels,
      anon_sym_points,
  [1276] = 1,
    ACTIONS(213), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [1282] = 1,
    ACTIONS(184), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [1288] = 2,
    STATE(68), 1,
      sym_xp_type,
    ACTIONS(211), 2,
      anon_sym_levels,
      anon_sym_points,
  [1296] = 1,
    ACTIONS(193), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [1302] = 1,
    ACTIONS(215), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [1308] = 2,
    STATE(67), 1,
      sym_xp_type,
    ACTIONS(211), 2,
      anon_sym_levels,
      anon_sym_points,
  [1316] = 3,
    ACTIONS(11), 1,
      sym_text,
    ACTIONS(217), 1,
      anon_sym_LBRACK,
    STATE(99), 1,
      sym_selector_arguments,
  [1326] = 2,
    ACTIONS(219), 1,
      anon_sym_COMMA,
    ACTIONS(221), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [1334] = 1,
    ACTIONS(15), 2,
      anon_sym_LBRACK,
      sym_text,
  [1339] = 2,
    ACTIONS(223), 1,
      anon_sym_set,
    ACTIONS(225), 1,
      anon_sym_query,
  [1346] = 1,
    ACTIONS(188), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [1351] = 1,
    ACTIONS(227), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [1356] = 1,
    ACTIONS(229), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [1361] = 2,
    ACTIONS(231), 1,
      anon_sym_LBRACE,
    STATE(61), 1,
      sym_block,
  [1368] = 1,
    ACTIONS(233), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [1373] = 1,
    ACTIONS(235), 1,
      sym_number,
  [1377] = 1,
    ACTIONS(237), 1,
      anon_sym_SEMI,
  [1381] = 1,
    ACTIONS(239), 1,
      anon_sym_SEMI,
  [1385] = 1,
    ACTIONS(241), 1,
      anon_sym_SEMI,
  [1389] = 1,
    ACTIONS(243), 1,
      anon_sym_SEMI,
  [1393] = 1,
    ACTIONS(245), 1,
      anon_sym_EQ,
  [1397] = 1,
    ACTIONS(247), 1,
      sym_text,
  [1401] = 1,
    ACTIONS(249), 1,
      ts_builtin_sym_end,
  [1405] = 1,
    ACTIONS(251), 1,
      sym_text,
  [1409] = 1,
    ACTIONS(253), 1,
      sym_number,
  [1413] = 1,
    ACTIONS(255), 1,
      sym_number,
  [1417] = 1,
    ACTIONS(257), 1,
      sym_number,
  [1421] = 1,
    ACTIONS(259), 1,
      anon_sym_SEMI,
  [1425] = 1,
    ACTIONS(261), 1,
      sym_identifier,
  [1429] = 1,
    ACTIONS(263), 1,
      sym_number,
  [1433] = 1,
    ACTIONS(265), 1,
      anon_sym_SEMI,
  [1437] = 1,
    ACTIONS(267), 1,
      sym_identifier,
  [1441] = 1,
    ACTIONS(269), 1,
      sym_number,
  [1445] = 1,
    ACTIONS(271), 1,
      sym_number,
  [1449] = 1,
    ACTIONS(273), 1,
      sym_number,
  [1453] = 1,
    ACTIONS(275), 1,
      sym_number,
  [1457] = 1,
    ACTIONS(277), 1,
      sym_number,
  [1461] = 1,
    ACTIONS(279), 1,
      anon_sym_SEMI,
  [1465] = 1,
    ACTIONS(281), 1,
      anon_sym_SEMI,
  [1469] = 1,
    ACTIONS(283), 1,
      anon_sym_SEMI,
  [1473] = 1,
    ACTIONS(285), 1,
      sym_number,
  [1477] = 1,
    ACTIONS(287), 1,
      anon_sym_SEMI,
  [1481] = 1,
    ACTIONS(289), 1,
      anon_sym_SEMI,
  [1485] = 1,
    ACTIONS(291), 1,
      anon_sym_SEMI,
  [1489] = 1,
    ACTIONS(293), 1,
      sym_number,
  [1493] = 1,
    ACTIONS(295), 1,
      anon_sym_SEMI,
  [1497] = 1,
    ACTIONS(297), 1,
      sym_number,
  [1501] = 1,
    ACTIONS(299), 1,
      anon_sym_SEMI,
  [1505] = 1,
    ACTIONS(301), 1,
      anon_sym_SEMI,
  [1509] = 1,
    ACTIONS(21), 1,
      sym_text,
  [1513] = 1,
    ACTIONS(17), 1,
      sym_text,
  [1517] = 1,
    ACTIONS(25), 1,
      sym_text,
  [1521] = 1,
    ACTIONS(303), 1,
      anon_sym_SEMI,
  [1525] = 1,
    ACTIONS(305), 1,
      anon_sym_SEMI,
  [1529] = 1,
    ACTIONS(307), 1,
      anon_sym_SEMI,
  [1533] = 1,
    ACTIONS(309), 1,
      sym_number,
  [1537] = 1,
    ACTIONS(311), 1,
      anon_sym_SEMI,
  [1541] = 1,
    ACTIONS(313), 1,
      anon_sym_SEMI,
  [1545] = 1,
    ACTIONS(315), 1,
      sym_number,
  [1549] = 1,
    ACTIONS(317), 1,
      sym_number,
  [1553] = 1,
    ACTIONS(319), 1,
      anon_sym_SEMI,
  [1557] = 1,
    ACTIONS(321), 1,
      anon_sym_SEMI,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 48,
  [SMALL_STATE(4)] = 91,
  [SMALL_STATE(5)] = 133,
  [SMALL_STATE(6)] = 175,
  [SMALL_STATE(7)] = 217,
  [SMALL_STATE(8)] = 262,
  [SMALL_STATE(9)] = 307,
  [SMALL_STATE(10)] = 349,
  [SMALL_STATE(11)] = 391,
  [SMALL_STATE(12)] = 428,
  [SMALL_STATE(13)] = 464,
  [SMALL_STATE(14)] = 502,
  [SMALL_STATE(15)] = 540,
  [SMALL_STATE(16)] = 576,
  [SMALL_STATE(17)] = 612,
  [SMALL_STATE(18)] = 681,
  [SMALL_STATE(19)] = 750,
  [SMALL_STATE(20)] = 819,
  [SMALL_STATE(21)] = 878,
  [SMALL_STATE(22)] = 899,
  [SMALL_STATE(23)] = 920,
  [SMALL_STATE(24)] = 937,
  [SMALL_STATE(25)] = 951,
  [SMALL_STATE(26)] = 965,
  [SMALL_STATE(27)] = 979,
  [SMALL_STATE(28)] = 993,
  [SMALL_STATE(29)] = 1007,
  [SMALL_STATE(30)] = 1021,
  [SMALL_STATE(31)] = 1035,
  [SMALL_STATE(32)] = 1049,
  [SMALL_STATE(33)] = 1063,
  [SMALL_STATE(34)] = 1077,
  [SMALL_STATE(35)] = 1091,
  [SMALL_STATE(36)] = 1105,
  [SMALL_STATE(37)] = 1122,
  [SMALL_STATE(38)] = 1135,
  [SMALL_STATE(39)] = 1147,
  [SMALL_STATE(40)] = 1159,
  [SMALL_STATE(41)] = 1168,
  [SMALL_STATE(42)] = 1181,
  [SMALL_STATE(43)] = 1190,
  [SMALL_STATE(44)] = 1203,
  [SMALL_STATE(45)] = 1216,
  [SMALL_STATE(46)] = 1229,
  [SMALL_STATE(47)] = 1242,
  [SMALL_STATE(48)] = 1255,
  [SMALL_STATE(49)] = 1268,
  [SMALL_STATE(50)] = 1276,
  [SMALL_STATE(51)] = 1282,
  [SMALL_STATE(52)] = 1288,
  [SMALL_STATE(53)] = 1296,
  [SMALL_STATE(54)] = 1302,
  [SMALL_STATE(55)] = 1308,
  [SMALL_STATE(56)] = 1316,
  [SMALL_STATE(57)] = 1326,
  [SMALL_STATE(58)] = 1334,
  [SMALL_STATE(59)] = 1339,
  [SMALL_STATE(60)] = 1346,
  [SMALL_STATE(61)] = 1351,
  [SMALL_STATE(62)] = 1356,
  [SMALL_STATE(63)] = 1361,
  [SMALL_STATE(64)] = 1368,
  [SMALL_STATE(65)] = 1373,
  [SMALL_STATE(66)] = 1377,
  [SMALL_STATE(67)] = 1381,
  [SMALL_STATE(68)] = 1385,
  [SMALL_STATE(69)] = 1389,
  [SMALL_STATE(70)] = 1393,
  [SMALL_STATE(71)] = 1397,
  [SMALL_STATE(72)] = 1401,
  [SMALL_STATE(73)] = 1405,
  [SMALL_STATE(74)] = 1409,
  [SMALL_STATE(75)] = 1413,
  [SMALL_STATE(76)] = 1417,
  [SMALL_STATE(77)] = 1421,
  [SMALL_STATE(78)] = 1425,
  [SMALL_STATE(79)] = 1429,
  [SMALL_STATE(80)] = 1433,
  [SMALL_STATE(81)] = 1437,
  [SMALL_STATE(82)] = 1441,
  [SMALL_STATE(83)] = 1445,
  [SMALL_STATE(84)] = 1449,
  [SMALL_STATE(85)] = 1453,
  [SMALL_STATE(86)] = 1457,
  [SMALL_STATE(87)] = 1461,
  [SMALL_STATE(88)] = 1465,
  [SMALL_STATE(89)] = 1469,
  [SMALL_STATE(90)] = 1473,
  [SMALL_STATE(91)] = 1477,
  [SMALL_STATE(92)] = 1481,
  [SMALL_STATE(93)] = 1485,
  [SMALL_STATE(94)] = 1489,
  [SMALL_STATE(95)] = 1493,
  [SMALL_STATE(96)] = 1497,
  [SMALL_STATE(97)] = 1501,
  [SMALL_STATE(98)] = 1505,
  [SMALL_STATE(99)] = 1509,
  [SMALL_STATE(100)] = 1513,
  [SMALL_STATE(101)] = 1517,
  [SMALL_STATE(102)] = 1521,
  [SMALL_STATE(103)] = 1525,
  [SMALL_STATE(104)] = 1529,
  [SMALL_STATE(105)] = 1533,
  [SMALL_STATE(106)] = 1537,
  [SMALL_STATE(107)] = 1541,
  [SMALL_STATE(108)] = 1545,
  [SMALL_STATE(109)] = 1549,
  [SMALL_STATE(110)] = 1553,
  [SMALL_STATE(111)] = 1557,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [7] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 1, 0, 2),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [11] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 1, 0, 2),
  [13] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_type, 1, 0, 0),
  [15] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_type, 1, 0, 0),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [19] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 2, 0, 2),
  [23] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 2, 0, 2),
  [25] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [27] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(109),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [43] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [46] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [48] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(105),
  [51] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(108),
  [54] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(34),
  [57] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(73),
  [60] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(26),
  [63] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(27),
  [66] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(29),
  [69] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(23),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(24),
  [75] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(59),
  [78] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(32),
  [81] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(33),
  [84] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(35),
  [87] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(28),
  [90] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [92] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [94] = {.entry = {.count = 1, .reusable = false}}, SHIFT(105),
  [96] = {.entry = {.count = 1, .reusable = false}}, SHIFT(108),
  [98] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [100] = {.entry = {.count = 1, .reusable = false}}, SHIFT(73),
  [102] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [104] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [106] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [108] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [110] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [112] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [114] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [116] = {.entry = {.count = 1, .reusable = false}}, SHIFT(33),
  [118] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [120] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [122] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [124] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [126] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [128] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [130] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [132] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [134] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [136] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [138] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [140] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [142] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [144] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [146] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [148] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [150] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [152] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [154] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [157] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [165] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [167] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [169] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [171] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [173] = {.entry = {.count = 1, .reusable = true}}, SHIFT(98),
  [175] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [177] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [179] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(78),
  [182] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [184] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_argument, 3, 0, 12),
  [186] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [188] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0),
  [190] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0), SHIFT_REPEAT(70),
  [193] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 2, 0, 0),
  [195] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [199] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [203] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [207] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [209] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [211] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [213] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted_string, 1, 0, 0),
  [215] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 3, 0, 0),
  [217] = {.entry = {.count = 1, .reusable = false}}, SHIFT(43),
  [219] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [221] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 1, 0, 0),
  [223] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [227] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 3, 0, 1),
  [229] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [231] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [233] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [235] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [237] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inv_clear_command, 2, 0, 4),
  [239] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_add_command, 4, 0, 9),
  [241] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_set_command, 4, 0, 9),
  [243] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_clear_command, 2, 0, 4),
  [245] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [247] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [249] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [251] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [253] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [255] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [257] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enchant_command, 4, 0, 10),
  [261] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [263] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_enchant, 1, 0, 0),
  [265] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [267] = {.entry = {.count = 1, .reusable = true}}, SHIFT(106),
  [269] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_effect, 1, 0, 0),
  [271] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [273] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [275] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [277] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [279] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_survival_command, 2, 0, 4),
  [281] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_adventure_command, 2, 0, 4),
  [283] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_spectator_command, 2, 0, 4),
  [285] = {.entry = {.count = 1, .reusable = true}}, SHIFT(95),
  [287] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 6, 0, 13),
  [289] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 6, 0, 14),
  [291] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_creative_command, 2, 0, 4),
  [293] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_enchant, 1, 0, 0),
  [295] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 7, 0, 15),
  [297] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [299] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command, 2, 0, 0),
  [301] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_unit, 1, 0, 0),
  [303] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 7),
  [305] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_type, 1, 0, 0),
  [307] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_query_command, 3, 0, 5),
  [309] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [311] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 8),
  [313] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tellraw_command, 3, 0, 6),
  [315] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [317] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_effect, 1, 0, 0),
  [319] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_say_command, 2, 0, 3),
  [321] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 5, 0, 11),
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
