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
#define STATE_COUNT 86
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 104
#define ALIAS_COUNT 0
#define TOKEN_COUNT 73
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 13
#define MAX_ALIAS_SEQUENCE_LENGTH 5
#define PRODUCTION_ID_COUNT 12

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
  anon_sym_minecraft_COLONspeed = 27,
  anon_sym_minecraft_COLONslowness = 28,
  anon_sym_minecraft_COLONhaste = 29,
  anon_sym_minecraft_COLONmining_fatigue = 30,
  anon_sym_minecraft_COLONstrength = 31,
  anon_sym_minecraft_COLONinstant_health = 32,
  anon_sym_minecraft_COLONinstant_damage = 33,
  anon_sym_minecraft_COLONjump_boost = 34,
  anon_sym_minecraft_COLONnausea = 35,
  anon_sym_minecraft_COLONregeneration = 36,
  anon_sym_minecraft_COLONresistance = 37,
  anon_sym_minecraft_COLONfire_resistance = 38,
  anon_sym_minecraft_COLONwater_breathing = 39,
  anon_sym_minecraft_COLONinvisibility = 40,
  anon_sym_minecraft_COLONblindness = 41,
  anon_sym_minecraft_COLONnight_vision = 42,
  anon_sym_minecraft_COLONhunger = 43,
  anon_sym_minecraft_COLONweakness = 44,
  anon_sym_minecraft_COLONpoison = 45,
  anon_sym_minecraft_COLONwither = 46,
  anon_sym_minecraft_COLONhealth_boost = 47,
  anon_sym_minecraft_COLONabsorption = 48,
  anon_sym_minecraft_COLONsaturation = 49,
  anon_sym_minecraft_COLONglowing = 50,
  anon_sym_minecraft_COLONlevitation = 51,
  anon_sym_minecraft_COLONluck = 52,
  anon_sym_minecraft_COLONunluck = 53,
  anon_sym_minecraft_COLONslow_falling = 54,
  anon_sym_minecraft_COLONconduit_power = 55,
  anon_sym_minecraft_COLONdolphins_grace = 56,
  anon_sym_minecraft_COLONbad_omen = 57,
  anon_sym_minecraft_COLONhero_of_the_village = 58,
  sym_text = 59,
  anon_sym_time = 60,
  anon_sym_set = 61,
  anon_sym_query = 62,
  anon_sym_gms = 63,
  anon_sym_gma = 64,
  anon_sym_gmsp = 65,
  anon_sym_gmc = 66,
  anon_sym_day = 67,
  anon_sym_night = 68,
  anon_sym_noon = 69,
  anon_sym_midnight = 70,
  sym_identifier = 71,
  sym_number = 72,
  sym_source_file = 73,
  sym__definition = 74,
  sym_function_definition = 75,
  sym_block = 76,
  sym__command = 77,
  sym_target_selector = 78,
  sym_selector_type = 79,
  sym_selector_arguments = 80,
  sym_selector_argument = 81,
  sym_xp_type = 82,
  sym_range_value = 83,
  sym_xp_add_command = 84,
  sym_xp_set_command = 85,
  sym_xp_query_command = 86,
  sym_quoted_string = 87,
  sym_say_command = 88,
  sym_inv_clear_command = 89,
  sym_effect_clear_command = 90,
  sym_tellraw_command = 91,
  sym_effect_command = 92,
  sym_vanilla_effect = 93,
  sym_custom_effect = 94,
  sym_time_command = 95,
  sym_gm_survival_command = 96,
  sym_gm_adventure_command = 97,
  sym_gm_spectator_command = 98,
  sym_gm_creative_command = 99,
  sym_time_unit = 100,
  aux_sym_source_file_repeat1 = 101,
  aux_sym_block_repeat1 = 102,
  aux_sym_selector_arguments_repeat1 = 103,
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
  [sym_vanilla_effect] = "vanilla_effect",
  [sym_custom_effect] = "custom_effect",
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
  [sym_vanilla_effect] = sym_vanilla_effect,
  [sym_custom_effect] = sym_custom_effect,
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
  [sym_vanilla_effect] = {
    .visible = true,
    .named = true,
  },
  [sym_custom_effect] = {
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
  field_etype = 6,
  field_key = 7,
  field_message = 8,
  field_name = 9,
  field_query_type = 10,
  field_selector = 11,
  field_target = 12,
  field_value = 13,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_amount] = "amount",
  [field_amplifier] = "amplifier",
  [field_body] = "body",
  [field_duration] = "duration",
  [field_effect_type] = "effect_type",
  [field_etype] = "etype",
  [field_key] = "key",
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
  [10] = {.index = 14, .length = 4},
  [11] = {.index = 18, .length = 2},
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
    {field_amplifier, 4},
    {field_duration, 3},
    {field_effect_type, 2},
    {field_target, 1},
  [18] =
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
  [34] = 30,
  [35] = 32,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 2,
  [45] = 3,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 5,
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
  [81] = 4,
  [82] = 6,
  [83] = 83,
  [84] = 84,
  [85] = 85,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(341);
      ADVANCE_MAP(
        '"', 2,
        ',', 354,
        '.', 3,
        ';', 344,
        '=', 356,
        '@', 19,
        '[', 352,
        ']', 355,
        'c', 175,
        'd', 20,
        'e', 65,
        'f', 194,
        'g', 189,
        'l', 74,
        'm', 142,
        'n', 143,
        'p', 223,
        'q', 321,
        's', 27,
        't', 86,
        'x', 245,
        '{', 343,
        '}', 346,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(416);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(2);
      if (lookahead == ',') ADVANCE(354);
      if (lookahead == '.') ADVANCE(3);
      if (lookahead == ']') ADVANCE(355);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(416);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(415);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(363);
      if (lookahead != 0) ADVANCE(2);
      END_STATE();
    case 3:
      if (lookahead == '.') ADVANCE(359);
      END_STATE();
    case 4:
      if (lookahead == ':') ADVANCE(24);
      END_STATE();
    case 5:
      if (lookahead == '_') ADVANCE(72);
      END_STATE();
    case 6:
      if (lookahead == '_') ADVANCE(53);
      END_STATE();
    case 7:
      if (lookahead == '_') ADVANCE(116);
      if (lookahead == 'n') ADVANCE(95);
      END_STATE();
    case 8:
      if (lookahead == '_') ADVANCE(55);
      END_STATE();
    case 9:
      if (lookahead == '_') ADVANCE(248);
      END_STATE();
    case 10:
      if (lookahead == '_') ADVANCE(124);
      END_STATE();
    case 11:
      if (lookahead == '_') ADVANCE(241);
      END_STATE();
    case 12:
      if (lookahead == '_') ADVANCE(227);
      END_STATE();
    case 13:
      if (lookahead == '_') ADVANCE(331);
      END_STATE();
    case 14:
      if (lookahead == '_') ADVANCE(117);
      END_STATE();
    case 15:
      if (lookahead == '_') ADVANCE(313);
      END_STATE();
    case 16:
      if (lookahead == '_') ADVANCE(330);
      END_STATE();
    case 17:
      if (lookahead == '_') ADVANCE(264);
      END_STATE();
    case 18:
      if (lookahead == '_') ADVANCE(56);
      END_STATE();
    case 19:
      if (lookahead == 'a') ADVANCE(348);
      if (lookahead == 'e') ADVANCE(351);
      if (lookahead == 'p') ADVANCE(349);
      if (lookahead == 'r') ADVANCE(350);
      if (lookahead == 's') ADVANCE(347);
      END_STATE();
    case 20:
      if (lookahead == 'a') ADVANCE(336);
      END_STATE();
    case 21:
      if (lookahead == 'a') ADVANCE(408);
      if (lookahead == 'c') ADVANCE(410);
      if (lookahead == 's') ADVANCE(407);
      END_STATE();
    case 22:
      if (lookahead == 'a') ADVANCE(70);
      if (lookahead == 'q') ADVANCE(326);
      if (lookahead == 's') ADVANCE(92);
      END_STATE();
    case 23:
      if (lookahead == 'a') ADVANCE(332);
      END_STATE();
    case 24:
      ADVANCE_MAP(
        'a', 52,
        'b', 32,
        'c', 231,
        'd', 229,
        'f', 146,
        'g', 179,
        'h', 35,
        'i', 196,
        'j', 320,
        'l', 88,
        'm', 160,
        'n', 31,
        'p', 230,
        'r', 78,
        's', 37,
        'u', 213,
        'w', 38,
      );
      END_STATE();
    case 25:
      if (lookahead == 'a') ADVANCE(377);
      END_STATE();
    case 26:
      if (lookahead == 'a') ADVANCE(337);
      END_STATE();
    case 27:
      if (lookahead == 'a') ADVANCE(337);
      if (lookahead == 'e') ADVANCE(289);
      END_STATE();
    case 28:
      if (lookahead == 'a') ADVANCE(114);
      END_STATE();
    case 29:
      if (lookahead == 'a') ADVANCE(250);
      END_STATE();
    case 30:
      if (lookahead == 'a') ADVANCE(173);
      END_STATE();
    case 31:
      if (lookahead == 'a') ADVANCE(323);
      if (lookahead == 'i') ADVANCE(131);
      END_STATE();
    case 32:
      if (lookahead == 'a') ADVANCE(67);
      if (lookahead == 'l') ADVANCE(151);
      END_STATE();
    case 33:
      if (lookahead == 'a') ADVANCE(192);
      END_STATE();
    case 34:
      if (lookahead == 'a') ADVANCE(251);
      END_STATE();
    case 35:
      if (lookahead == 'a') ADVANCE(283);
      if (lookahead == 'e') ADVANCE(36);
      if (lookahead == 'u') ADVANCE(204);
      END_STATE();
    case 36:
      if (lookahead == 'a') ADVANCE(183);
      if (lookahead == 'r') ADVANCE(240);
      END_STATE();
    case 37:
      if (lookahead == 'a') ADVANCE(299);
      if (lookahead == 'l') ADVANCE(226);
      if (lookahead == 'p') ADVANCE(102);
      if (lookahead == 't') ADVANCE(263);
      END_STATE();
    case 38:
      if (lookahead == 'a') ADVANCE(311);
      if (lookahead == 'e') ADVANCE(30);
      if (lookahead == 'i') ADVANCE(297);
      END_STATE();
    case 39:
      if (lookahead == 'a') ADVANCE(217);
      END_STATE();
    case 40:
      if (lookahead == 'a') ADVANCE(212);
      END_STATE();
    case 41:
      if (lookahead == 'a') ADVANCE(310);
      END_STATE();
    case 42:
      if (lookahead == 'a') ADVANCE(316);
      END_STATE();
    case 43:
      if (lookahead == 'a') ADVANCE(181);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(127);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(63);
      END_STATE();
    case 46:
      if (lookahead == 'a') ADVANCE(184);
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(221);
      END_STATE();
    case 48:
      if (lookahead == 'a') ADVANCE(128);
      END_STATE();
    case 49:
      if (lookahead == 'a') ADVANCE(317);
      END_STATE();
    case 50:
      if (lookahead == 'a') ADVANCE(318);
      END_STATE();
    case 51:
      if (lookahead == 'a') ADVANCE(319);
      END_STATE();
    case 52:
      if (lookahead == 'b') ADVANCE(273);
      END_STATE();
    case 53:
      if (lookahead == 'b') ADVANCE(233);
      END_STATE();
    case 54:
      if (lookahead == 'b') ADVANCE(150);
      END_STATE();
    case 55:
      if (lookahead == 'b') ADVANCE(265);
      END_STATE();
    case 56:
      if (lookahead == 'b') ADVANCE(244);
      END_STATE();
    case 57:
      if (lookahead == 'c') ADVANCE(175);
      if (lookahead == 'e') ADVANCE(65);
      if (lookahead == 'g') ADVANCE(189);
      if (lookahead == 's') ADVANCE(26);
      if (lookahead == 't') ADVANCE(86);
      if (lookahead == 'x') ADVANCE(245);
      if (lookahead == '}') ADVANCE(346);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(345);
      END_STATE();
    case 58:
      if (lookahead == 'c') ADVANCE(171);
      END_STATE();
    case 59:
      if (lookahead == 'c') ADVANCE(172);
      END_STATE();
    case 60:
      if (lookahead == 'c') ADVANCE(292);
      END_STATE();
    case 61:
      if (lookahead == 'c') ADVANCE(257);
      END_STATE();
    case 62:
      if (lookahead == 'c') ADVANCE(80);
      END_STATE();
    case 63:
      if (lookahead == 'c') ADVANCE(81);
      END_STATE();
    case 64:
      if (lookahead == 'c') ADVANCE(84);
      END_STATE();
    case 65:
      if (lookahead == 'c') ADVANCE(185);
      if (lookahead == 'f') ADVANCE(113);
      END_STATE();
    case 66:
      if (lookahead == 'd') ADVANCE(360);
      END_STATE();
    case 67:
      if (lookahead == 'd') ADVANCE(11);
      END_STATE();
    case 68:
      if (lookahead == 'd') ADVANCE(369);
      END_STATE();
    case 69:
      if (lookahead == 'd') ADVANCE(205);
      if (lookahead == 'n') ADVANCE(89);
      END_STATE();
    case 70:
      if (lookahead == 'd') ADVANCE(66);
      END_STATE();
    case 71:
      if (lookahead == 'd') ADVANCE(327);
      END_STATE();
    case 72:
      if (lookahead == 'd') ADVANCE(33);
      if (lookahead == 'h') ADVANCE(98);
      END_STATE();
    case 73:
      if (lookahead == 'd') ADVANCE(222);
      END_STATE();
    case 74:
      if (lookahead == 'e') ADVANCE(328);
      END_STATE();
    case 75:
      if (lookahead == 'e') ADVANCE(256);
      END_STATE();
    case 76:
      if (lookahead == 'e') ADVANCE(60);
      END_STATE();
    case 77:
      if (lookahead == 'e') ADVANCE(404);
      END_STATE();
    case 78:
      if (lookahead == 'e') ADVANCE(123);
      END_STATE();
    case 79:
      if (lookahead == 'e') ADVANCE(371);
      END_STATE();
    case 80:
      if (lookahead == 'e') ADVANCE(379);
      END_STATE();
    case 81:
      if (lookahead == 'e') ADVANCE(398);
      END_STATE();
    case 82:
      if (lookahead == 'e') ADVANCE(375);
      END_STATE();
    case 83:
      if (lookahead == 'e') ADVANCE(372);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(380);
      END_STATE();
    case 85:
      if (lookahead == 'e') ADVANCE(400);
      END_STATE();
    case 86:
      if (lookahead == 'e') ADVANCE(177);
      if (lookahead == 'i') ADVANCE(191);
      END_STATE();
    case 87:
      if (lookahead == 'e') ADVANCE(29);
      END_STATE();
    case 88:
      if (lookahead == 'e') ADVANCE(329);
      if (lookahead == 'u') ADVANCE(58);
      END_STATE();
    case 89:
      if (lookahead == 'e') ADVANCE(61);
      END_STATE();
    case 90:
      if (lookahead == 'e') ADVANCE(17);
      END_STATE();
    case 91:
      if (lookahead == 'e') ADVANCE(174);
      END_STATE();
    case 92:
      if (lookahead == 'e') ADVANCE(291);
      END_STATE();
    case 93:
      if (lookahead == 'e') ADVANCE(68);
      END_STATE();
    case 94:
      if (lookahead == 'e') ADVANCE(25);
      END_STATE();
    case 95:
      if (lookahead == 'e') ADVANCE(275);
      END_STATE();
    case 96:
      if (lookahead == 'e') ADVANCE(260);
      END_STATE();
    case 97:
      if (lookahead == 'e') ADVANCE(252);
      END_STATE();
    case 98:
      if (lookahead == 'e') ADVANCE(46);
      END_STATE();
    case 99:
      if (lookahead == 'e') ADVANCE(218);
      END_STATE();
    case 100:
      if (lookahead == 'e') ADVANCE(253);
      END_STATE();
    case 101:
      if (lookahead == 'e') ADVANCE(208);
      END_STATE();
    case 102:
      if (lookahead == 'e') ADVANCE(93);
      END_STATE();
    case 103:
      if (lookahead == 'e') ADVANCE(254);
      END_STATE();
    case 104:
      if (lookahead == 'e') ADVANCE(198);
      END_STATE();
    case 105:
      if (lookahead == 'e') ADVANCE(259);
      END_STATE();
    case 106:
      if (lookahead == 'e') ADVANCE(34);
      END_STATE();
    case 107:
      if (lookahead == 'e') ADVANCE(288);
      END_STATE();
    case 108:
      if (lookahead == 'e') ADVANCE(278);
      END_STATE();
    case 109:
      if (lookahead == 'e') ADVANCE(42);
      END_STATE();
    case 110:
      if (lookahead == 'e') ADVANCE(279);
      END_STATE();
    case 111:
      if (lookahead == 'e') ADVANCE(16);
      END_STATE();
    case 112:
      if (lookahead == 'e') ADVANCE(267);
      END_STATE();
    case 113:
      if (lookahead == 'f') ADVANCE(76);
      END_STATE();
    case 114:
      if (lookahead == 'f') ADVANCE(294);
      END_STATE();
    case 115:
      if (lookahead == 'f') ADVANCE(15);
      END_STATE();
    case 116:
      if (lookahead == 'f') ADVANCE(43);
      END_STATE();
    case 117:
      if (lookahead == 'f') ADVANCE(41);
      END_STATE();
    case 118:
      if (lookahead == 'g') ADVANCE(134);
      END_STATE();
    case 119:
      if (lookahead == 'g') ADVANCE(392);
      END_STATE();
    case 120:
      if (lookahead == 'g') ADVANCE(396);
      END_STATE();
    case 121:
      if (lookahead == 'g') ADVANCE(381);
      END_STATE();
    case 122:
      if (lookahead == 'g') ADVANCE(14);
      END_STATE();
    case 123:
      if (lookahead == 'g') ADVANCE(99);
      if (lookahead == 's') ADVANCE(161);
      END_STATE();
    case 124:
      if (lookahead == 'g') ADVANCE(262);
      END_STATE();
    case 125:
      if (lookahead == 'g') ADVANCE(97);
      END_STATE();
    case 126:
      if (lookahead == 'g') ADVANCE(303);
      END_STATE();
    case 127:
      if (lookahead == 'g') ADVANCE(82);
      END_STATE();
    case 128:
      if (lookahead == 'g') ADVANCE(85);
      END_STATE();
    case 129:
      if (lookahead == 'g') ADVANCE(325);
      END_STATE();
    case 130:
      if (lookahead == 'g') ADVANCE(135);
      END_STATE();
    case 131:
      if (lookahead == 'g') ADVANCE(136);
      END_STATE();
    case 132:
      if (lookahead == 'h') ADVANCE(373);
      END_STATE();
    case 133:
      if (lookahead == 'h') ADVANCE(374);
      END_STATE();
    case 134:
      if (lookahead == 'h') ADVANCE(290);
      END_STATE();
    case 135:
      if (lookahead == 'h') ADVANCE(293);
      END_STATE();
    case 136:
      if (lookahead == 'h') ADVANCE(301);
      END_STATE();
    case 137:
      if (lookahead == 'h') ADVANCE(100);
      END_STATE();
    case 138:
      if (lookahead == 'h') ADVANCE(111);
      END_STATE();
    case 139:
      if (lookahead == 'h') ADVANCE(155);
      END_STATE();
    case 140:
      if (lookahead == 'h') ADVANCE(159);
      END_STATE();
    case 141:
      if (lookahead == 'h') ADVANCE(18);
      END_STATE();
    case 142:
      if (lookahead == 'i') ADVANCE(69);
      END_STATE();
    case 143:
      if (lookahead == 'i') ADVANCE(118);
      if (lookahead == 'o') ADVANCE(225);
      END_STATE();
    case 144:
      if (lookahead == 'i') ADVANCE(54);
      END_STATE();
    case 145:
      if (lookahead == 'i') ADVANCE(207);
      END_STATE();
    case 146:
      if (lookahead == 'i') ADVANCE(261);
      END_STATE();
    case 147:
      if (lookahead == 'i') ADVANCE(285);
      END_STATE();
    case 148:
      if (lookahead == 'i') ADVANCE(129);
      END_STATE();
    case 149:
      if (lookahead == 'i') ADVANCE(274);
      END_STATE();
    case 150:
      if (lookahead == 'i') ADVANCE(186);
      END_STATE();
    case 151:
      if (lookahead == 'i') ADVANCE(211);
      END_STATE();
    case 152:
      if (lookahead == 'i') ADVANCE(206);
      END_STATE();
    case 153:
      if (lookahead == 'i') ADVANCE(210);
      END_STATE();
    case 154:
      if (lookahead == 'i') ADVANCE(306);
      END_STATE();
    case 155:
      if (lookahead == 'i') ADVANCE(216);
      END_STATE();
    case 156:
      if (lookahead == 'i') ADVANCE(307);
      END_STATE();
    case 157:
      if (lookahead == 'i') ADVANCE(302);
      END_STATE();
    case 158:
      if (lookahead == 'i') ADVANCE(214);
      END_STATE();
    case 159:
      if (lookahead == 'i') ADVANCE(215);
      END_STATE();
    case 160:
      if (lookahead == 'i') ADVANCE(220);
      END_STATE();
    case 161:
      if (lookahead == 'i') ADVANCE(284);
      END_STATE();
    case 162:
      if (lookahead == 'i') ADVANCE(182);
      END_STATE();
    case 163:
      if (lookahead == 'i') ADVANCE(287);
      END_STATE();
    case 164:
      if (lookahead == 'i') ADVANCE(235);
      END_STATE();
    case 165:
      if (lookahead == 'i') ADVANCE(236);
      END_STATE();
    case 166:
      if (lookahead == 'i') ADVANCE(237);
      END_STATE();
    case 167:
      if (lookahead == 'i') ADVANCE(238);
      END_STATE();
    case 168:
      if (lookahead == 'i') ADVANCE(239);
      END_STATE();
    case 169:
      if (lookahead == 'i') ADVANCE(130);
      END_STATE();
    case 170:
      if (lookahead == 'i') ADVANCE(286);
      END_STATE();
    case 171:
      if (lookahead == 'k') ADVANCE(394);
      END_STATE();
    case 172:
      if (lookahead == 'k') ADVANCE(395);
      END_STATE();
    case 173:
      if (lookahead == 'k') ADVANCE(219);
      END_STATE();
    case 174:
      if (lookahead == 'l') ADVANCE(268);
      END_STATE();
    case 175:
      if (lookahead == 'l') ADVANCE(87);
      END_STATE();
    case 176:
      if (lookahead == 'l') ADVANCE(246);
      END_STATE();
    case 177:
      if (lookahead == 'l') ADVANCE(178);
      END_STATE();
    case 178:
      if (lookahead == 'l') ADVANCE(255);
      END_STATE();
    case 179:
      if (lookahead == 'l') ADVANCE(224);
      END_STATE();
    case 180:
      if (lookahead == 'l') ADVANCE(322);
      END_STATE();
    case 181:
      if (lookahead == 'l') ADVANCE(187);
      END_STATE();
    case 182:
      if (lookahead == 'l') ADVANCE(188);
      END_STATE();
    case 183:
      if (lookahead == 'l') ADVANCE(300);
      END_STATE();
    case 184:
      if (lookahead == 'l') ADVANCE(305);
      END_STATE();
    case 185:
      if (lookahead == 'l') ADVANCE(106);
      END_STATE();
    case 186:
      if (lookahead == 'l') ADVANCE(157);
      END_STATE();
    case 187:
      if (lookahead == 'l') ADVANCE(158);
      END_STATE();
    case 188:
      if (lookahead == 'l') ADVANCE(48);
      END_STATE();
    case 189:
      if (lookahead == 'm') ADVANCE(21);
      END_STATE();
    case 190:
      if (lookahead == 'm') ADVANCE(247);
      END_STATE();
    case 191:
      if (lookahead == 'm') ADVANCE(77);
      END_STATE();
    case 192:
      if (lookahead == 'm') ADVANCE(44);
      END_STATE();
    case 193:
      if (lookahead == 'm') ADVANCE(104);
      END_STATE();
    case 194:
      if (lookahead == 'n') ADVANCE(342);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(413);
      END_STATE();
    case 196:
      if (lookahead == 'n') ADVANCE(277);
      END_STATE();
    case 197:
      if (lookahead == 'n') ADVANCE(387);
      END_STATE();
    case 198:
      if (lookahead == 'n') ADVANCE(399);
      END_STATE();
    case 199:
      if (lookahead == 'n') ADVANCE(390);
      END_STATE();
    case 200:
      if (lookahead == 'n') ADVANCE(393);
      END_STATE();
    case 201:
      if (lookahead == 'n') ADVANCE(391);
      END_STATE();
    case 202:
      if (lookahead == 'n') ADVANCE(384);
      END_STATE();
    case 203:
      if (lookahead == 'n') ADVANCE(378);
      END_STATE();
    case 204:
      if (lookahead == 'n') ADVANCE(125);
      END_STATE();
    case 205:
      if (lookahead == 'n') ADVANCE(169);
      END_STATE();
    case 206:
      if (lookahead == 'n') ADVANCE(122);
      END_STATE();
    case 207:
      if (lookahead == 'n') ADVANCE(298);
      END_STATE();
    case 208:
      if (lookahead == 'n') ADVANCE(126);
      END_STATE();
    case 209:
      if (lookahead == 'n') ADVANCE(71);
      END_STATE();
    case 210:
      if (lookahead == 'n') ADVANCE(119);
      END_STATE();
    case 211:
      if (lookahead == 'n') ADVANCE(73);
      END_STATE();
    case 212:
      if (lookahead == 'n') ADVANCE(62);
      END_STATE();
    case 213:
      if (lookahead == 'n') ADVANCE(180);
      END_STATE();
    case 214:
      if (lookahead == 'n') ADVANCE(120);
      END_STATE();
    case 215:
      if (lookahead == 'n') ADVANCE(121);
      END_STATE();
    case 216:
      if (lookahead == 'n') ADVANCE(276);
      END_STATE();
    case 217:
      if (lookahead == 'n') ADVANCE(308);
      END_STATE();
    case 218:
      if (lookahead == 'n') ADVANCE(112);
      END_STATE();
    case 219:
      if (lookahead == 'n') ADVANCE(108);
      END_STATE();
    case 220:
      if (lookahead == 'n') ADVANCE(152);
      END_STATE();
    case 221:
      if (lookahead == 'n') ADVANCE(64);
      END_STATE();
    case 222:
      if (lookahead == 'n') ADVANCE(110);
      END_STATE();
    case 223:
      if (lookahead == 'o') ADVANCE(145);
      END_STATE();
    case 224:
      if (lookahead == 'o') ADVANCE(335);
      END_STATE();
    case 225:
      if (lookahead == 'o') ADVANCE(195);
      END_STATE();
    case 226:
      if (lookahead == 'o') ADVANCE(333);
      END_STATE();
    case 227:
      if (lookahead == 'o') ADVANCE(115);
      END_STATE();
    case 228:
      if (lookahead == 'o') ADVANCE(334);
      END_STATE();
    case 229:
      if (lookahead == 'o') ADVANCE(176);
      END_STATE();
    case 230:
      if (lookahead == 'o') ADVANCE(147);
      END_STATE();
    case 231:
      if (lookahead == 'o') ADVANCE(209);
      END_STATE();
    case 232:
      if (lookahead == 'o') ADVANCE(258);
      END_STATE();
    case 233:
      if (lookahead == 'o') ADVANCE(242);
      END_STATE();
    case 234:
      if (lookahead == 'o') ADVANCE(197);
      END_STATE();
    case 235:
      if (lookahead == 'o') ADVANCE(199);
      END_STATE();
    case 236:
      if (lookahead == 'o') ADVANCE(200);
      END_STATE();
    case 237:
      if (lookahead == 'o') ADVANCE(201);
      END_STATE();
    case 238:
      if (lookahead == 'o') ADVANCE(202);
      END_STATE();
    case 239:
      if (lookahead == 'o') ADVANCE(203);
      END_STATE();
    case 240:
      if (lookahead == 'o') ADVANCE(12);
      END_STATE();
    case 241:
      if (lookahead == 'o') ADVANCE(193);
      END_STATE();
    case 242:
      if (lookahead == 'o') ADVANCE(281);
      END_STATE();
    case 243:
      if (lookahead == 'o') ADVANCE(282);
      END_STATE();
    case 244:
      if (lookahead == 'o') ADVANCE(243);
      END_STATE();
    case 245:
      if (lookahead == 'p') ADVANCE(22);
      END_STATE();
    case 246:
      if (lookahead == 'p') ADVANCE(139);
      END_STATE();
    case 247:
      if (lookahead == 'p') ADVANCE(6);
      END_STATE();
    case 248:
      if (lookahead == 'p') ADVANCE(228);
      END_STATE();
    case 249:
      if (lookahead == 'p') ADVANCE(309);
      END_STATE();
    case 250:
      if (lookahead == 'r') ADVANCE(365);
      END_STATE();
    case 251:
      if (lookahead == 'r') ADVANCE(366);
      END_STATE();
    case 252:
      if (lookahead == 'r') ADVANCE(385);
      END_STATE();
    case 253:
      if (lookahead == 'r') ADVANCE(388);
      END_STATE();
    case 254:
      if (lookahead == 'r') ADVANCE(397);
      END_STATE();
    case 255:
      if (lookahead == 'r') ADVANCE(23);
      END_STATE();
    case 256:
      if (lookahead == 'r') ADVANCE(338);
      END_STATE();
    case 257:
      if (lookahead == 'r') ADVANCE(28);
      END_STATE();
    case 258:
      if (lookahead == 'r') ADVANCE(249);
      END_STATE();
    case 259:
      if (lookahead == 'r') ADVANCE(339);
      END_STATE();
    case 260:
      if (lookahead == 'r') ADVANCE(8);
      END_STATE();
    case 261:
      if (lookahead == 'r') ADVANCE(90);
      END_STATE();
    case 262:
      if (lookahead == 'r') ADVANCE(45);
      END_STATE();
    case 263:
      if (lookahead == 'r') ADVANCE(101);
      END_STATE();
    case 264:
      if (lookahead == 'r') ADVANCE(107);
      END_STATE();
    case 265:
      if (lookahead == 'r') ADVANCE(109);
      END_STATE();
    case 266:
      if (lookahead == 'r') ADVANCE(50);
      END_STATE();
    case 267:
      if (lookahead == 'r') ADVANCE(51);
      END_STATE();
    case 268:
      if (lookahead == 's') ADVANCE(357);
      END_STATE();
    case 269:
      if (lookahead == 's') ADVANCE(358);
      END_STATE();
    case 270:
      if (lookahead == 's') ADVANCE(370);
      END_STATE();
    case 271:
      if (lookahead == 's') ADVANCE(386);
      END_STATE();
    case 272:
      if (lookahead == 's') ADVANCE(383);
      END_STATE();
    case 273:
      if (lookahead == 's') ADVANCE(232);
      END_STATE();
    case 274:
      if (lookahead == 's') ADVANCE(144);
      END_STATE();
    case 275:
      if (lookahead == 's') ADVANCE(270);
      END_STATE();
    case 276:
      if (lookahead == 's') ADVANCE(10);
      END_STATE();
    case 277:
      if (lookahead == 's') ADVANCE(304);
      if (lookahead == 'v') ADVANCE(149);
      END_STATE();
    case 278:
      if (lookahead == 's') ADVANCE(271);
      END_STATE();
    case 279:
      if (lookahead == 's') ADVANCE(272);
      END_STATE();
    case 280:
      if (lookahead == 's') ADVANCE(94);
      END_STATE();
    case 281:
      if (lookahead == 's') ADVANCE(295);
      END_STATE();
    case 282:
      if (lookahead == 's') ADVANCE(296);
      END_STATE();
    case 283:
      if (lookahead == 's') ADVANCE(312);
      END_STATE();
    case 284:
      if (lookahead == 's') ADVANCE(314);
      END_STATE();
    case 285:
      if (lookahead == 's') ADVANCE(234);
      END_STATE();
    case 286:
      if (lookahead == 's') ADVANCE(315);
      END_STATE();
    case 287:
      if (lookahead == 's') ADVANCE(167);
      END_STATE();
    case 288:
      if (lookahead == 's') ADVANCE(170);
      END_STATE();
    case 289:
      if (lookahead == 't') ADVANCE(405);
      END_STATE();
    case 290:
      if (lookahead == 't') ADVANCE(412);
      END_STATE();
    case 291:
      if (lookahead == 't') ADVANCE(361);
      END_STATE();
    case 292:
      if (lookahead == 't') ADVANCE(368);
      END_STATE();
    case 293:
      if (lookahead == 't') ADVANCE(414);
      END_STATE();
    case 294:
      if (lookahead == 't') ADVANCE(4);
      END_STATE();
    case 295:
      if (lookahead == 't') ADVANCE(376);
      END_STATE();
    case 296:
      if (lookahead == 't') ADVANCE(389);
      END_STATE();
    case 297:
      if (lookahead == 't') ADVANCE(137);
      END_STATE();
    case 298:
      if (lookahead == 't') ADVANCE(269);
      END_STATE();
    case 299:
      if (lookahead == 't') ADVANCE(324);
      END_STATE();
    case 300:
      if (lookahead == 't') ADVANCE(141);
      END_STATE();
    case 301:
      if (lookahead == 't') ADVANCE(13);
      END_STATE();
    case 302:
      if (lookahead == 't') ADVANCE(340);
      END_STATE();
    case 303:
      if (lookahead == 't') ADVANCE(132);
      END_STATE();
    case 304:
      if (lookahead == 't') ADVANCE(39);
      END_STATE();
    case 305:
      if (lookahead == 't') ADVANCE(133);
      END_STATE();
    case 306:
      if (lookahead == 't') ADVANCE(49);
      END_STATE();
    case 307:
      if (lookahead == 't') ADVANCE(9);
      END_STATE();
    case 308:
      if (lookahead == 't') ADVANCE(5);
      END_STATE();
    case 309:
      if (lookahead == 't') ADVANCE(164);
      END_STATE();
    case 310:
      if (lookahead == 't') ADVANCE(148);
      END_STATE();
    case 311:
      if (lookahead == 't') ADVANCE(96);
      END_STATE();
    case 312:
      if (lookahead == 't') ADVANCE(79);
      END_STATE();
    case 313:
      if (lookahead == 't') ADVANCE(138);
      END_STATE();
    case 314:
      if (lookahead == 't') ADVANCE(40);
      END_STATE();
    case 315:
      if (lookahead == 't') ADVANCE(47);
      END_STATE();
    case 316:
      if (lookahead == 't') ADVANCE(140);
      END_STATE();
    case 317:
      if (lookahead == 't') ADVANCE(165);
      END_STATE();
    case 318:
      if (lookahead == 't') ADVANCE(166);
      END_STATE();
    case 319:
      if (lookahead == 't') ADVANCE(168);
      END_STATE();
    case 320:
      if (lookahead == 'u') ADVANCE(190);
      END_STATE();
    case 321:
      if (lookahead == 'u') ADVANCE(75);
      END_STATE();
    case 322:
      if (lookahead == 'u') ADVANCE(59);
      END_STATE();
    case 323:
      if (lookahead == 'u') ADVANCE(280);
      END_STATE();
    case 324:
      if (lookahead == 'u') ADVANCE(266);
      END_STATE();
    case 325:
      if (lookahead == 'u') ADVANCE(83);
      END_STATE();
    case 326:
      if (lookahead == 'u') ADVANCE(105);
      END_STATE();
    case 327:
      if (lookahead == 'u') ADVANCE(156);
      END_STATE();
    case 328:
      if (lookahead == 'v') ADVANCE(91);
      END_STATE();
    case 329:
      if (lookahead == 'v') ADVANCE(154);
      END_STATE();
    case 330:
      if (lookahead == 'v') ADVANCE(162);
      END_STATE();
    case 331:
      if (lookahead == 'v') ADVANCE(163);
      END_STATE();
    case 332:
      if (lookahead == 'w') ADVANCE(367);
      END_STATE();
    case 333:
      if (lookahead == 'w') ADVANCE(7);
      END_STATE();
    case 334:
      if (lookahead == 'w') ADVANCE(103);
      END_STATE();
    case 335:
      if (lookahead == 'w') ADVANCE(153);
      END_STATE();
    case 336:
      if (lookahead == 'y') ADVANCE(411);
      END_STATE();
    case 337:
      if (lookahead == 'y') ADVANCE(364);
      END_STATE();
    case 338:
      if (lookahead == 'y') ADVANCE(406);
      END_STATE();
    case 339:
      if (lookahead == 'y') ADVANCE(362);
      END_STATE();
    case 340:
      if (lookahead == 'y') ADVANCE(382);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(anon_sym_fn);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(aux_sym_block_token1);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(345);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(anon_sym_ATs);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(anon_sym_ATa);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(anon_sym_ATp);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(anon_sym_ATr);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(anon_sym_ATe);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(403);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(anon_sym_levels);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(anon_sym_points);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(anon_sym_xpadd);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(anon_sym_xpset);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(anon_sym_xpquery);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(aux_sym_quoted_string_token1);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(anon_sym_say);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(anon_sym_clear);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(anon_sym_eclear);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(anon_sym_tellraw);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(anon_sym_effect);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONspeed);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslowness);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhaste);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmining_fatigue);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONstrength);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_health);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_damage);
      END_STATE();
    case 376:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONjump_boost);
      END_STATE();
    case 377:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnausea);
      END_STATE();
    case 378:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONregeneration);
      END_STATE();
    case 379:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONresistance);
      END_STATE();
    case 380:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_resistance);
      END_STATE();
    case 381:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwater_breathing);
      END_STATE();
    case 382:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinvisibility);
      END_STATE();
    case 383:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblindness);
      END_STATE();
    case 384:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnight_vision);
      END_STATE();
    case 385:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhunger);
      END_STATE();
    case 386:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONweakness);
      END_STATE();
    case 387:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpoison);
      END_STATE();
    case 388:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwither);
      END_STATE();
    case 389:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhealth_boost);
      END_STATE();
    case 390:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONabsorption);
      END_STATE();
    case 391:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsaturation);
      END_STATE();
    case 392:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONglowing);
      END_STATE();
    case 393:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlevitation);
      END_STATE();
    case 394:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      END_STATE();
    case 395:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunluck);
      END_STATE();
    case 396:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslow_falling);
      END_STATE();
    case 397:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONconduit_power);
      END_STATE();
    case 398:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdolphins_grace);
      END_STATE();
    case 399:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbad_omen);
      END_STATE();
    case 400:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhero_of_the_village);
      END_STATE();
    case 401:
      ACCEPT_TOKEN(sym_text);
      if (lookahead == '[') ADVANCE(353);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(401);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(403);
      END_STATE();
    case 402:
      ACCEPT_TOKEN(sym_text);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(402);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(403);
      END_STATE();
    case 403:
      ACCEPT_TOKEN(sym_text);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(403);
      END_STATE();
    case 404:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 405:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 406:
      ACCEPT_TOKEN(anon_sym_query);
      END_STATE();
    case 407:
      ACCEPT_TOKEN(anon_sym_gms);
      if (lookahead == 'p') ADVANCE(409);
      END_STATE();
    case 408:
      ACCEPT_TOKEN(anon_sym_gma);
      END_STATE();
    case 409:
      ACCEPT_TOKEN(anon_sym_gmsp);
      END_STATE();
    case 410:
      ACCEPT_TOKEN(anon_sym_gmc);
      END_STATE();
    case 411:
      ACCEPT_TOKEN(anon_sym_day);
      END_STATE();
    case 412:
      ACCEPT_TOKEN(anon_sym_night);
      END_STATE();
    case 413:
      ACCEPT_TOKEN(anon_sym_noon);
      END_STATE();
    case 414:
      ACCEPT_TOKEN(anon_sym_midnight);
      END_STATE();
    case 415:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(415);
      END_STATE();
    case 416:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(416);
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
  [7] = {.lex_state = 0},
  [8] = {.lex_state = 57},
  [9] = {.lex_state = 57},
  [10] = {.lex_state = 57},
  [11] = {.lex_state = 0},
  [12] = {.lex_state = 57},
  [13] = {.lex_state = 57},
  [14] = {.lex_state = 0},
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
  [26] = {.lex_state = 1},
  [27] = {.lex_state = 0},
  [28] = {.lex_state = 0},
  [29] = {.lex_state = 1},
  [30] = {.lex_state = 1},
  [31] = {.lex_state = 1},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 1},
  [35] = {.lex_state = 1},
  [36] = {.lex_state = 1},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 1},
  [39] = {.lex_state = 1},
  [40] = {.lex_state = 1},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 1},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 401},
  [45] = {.lex_state = 401},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 1},
  [49] = {.lex_state = 0},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 0},
  [52] = {.lex_state = 402},
  [53] = {.lex_state = 0},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 0},
  [58] = {.lex_state = 1},
  [59] = {.lex_state = 0},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 0},
  [63] = {.lex_state = 402},
  [64] = {.lex_state = 0},
  [65] = {.lex_state = 0},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 0},
  [69] = {.lex_state = 0},
  [70] = {.lex_state = 0},
  [71] = {.lex_state = 0},
  [72] = {.lex_state = 0},
  [73] = {.lex_state = 1},
  [74] = {.lex_state = 0},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 0},
  [79] = {.lex_state = 0},
  [80] = {.lex_state = 0},
  [81] = {.lex_state = 402},
  [82] = {.lex_state = 402},
  [83] = {.lex_state = 402},
  [84] = {.lex_state = 0},
  [85] = {.lex_state = 0},
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
    [sym_source_file] = STATE(78),
    [sym__definition] = STATE(28),
    [sym_function_definition] = STATE(28),
    [aux_sym_source_file_repeat1] = STATE(28),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_fn] = ACTIONS(5),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 3,
    ACTIONS(9), 1,
      anon_sym_LBRACK,
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
  [45] = 1,
    ACTIONS(11), 37,
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
  [85] = 1,
    ACTIONS(13), 36,
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
  [124] = 1,
    ACTIONS(15), 36,
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
  [163] = 1,
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
  [202] = 3,
    ACTIONS(19), 1,
      aux_sym_quoted_string_token1,
    STATE(54), 2,
      sym_vanilla_effect,
      sym_custom_effect,
    ACTIONS(21), 32,
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
  [244] = 17,
    ACTIONS(23), 1,
      aux_sym_block_token1,
    ACTIONS(25), 1,
      anon_sym_RBRACE,
    ACTIONS(27), 1,
      anon_sym_xpadd,
    ACTIONS(29), 1,
      anon_sym_xpset,
    ACTIONS(31), 1,
      anon_sym_xpquery,
    ACTIONS(33), 1,
      anon_sym_say,
    ACTIONS(35), 1,
      anon_sym_clear,
    ACTIONS(37), 1,
      anon_sym_eclear,
    ACTIONS(39), 1,
      anon_sym_tellraw,
    ACTIONS(41), 1,
      anon_sym_effect,
    ACTIONS(43), 1,
      anon_sym_time,
    ACTIONS(45), 1,
      anon_sym_gms,
    ACTIONS(47), 1,
      anon_sym_gma,
    ACTIONS(49), 1,
      anon_sym_gmsp,
    ACTIONS(51), 1,
      anon_sym_gmc,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(69), 14,
      sym__command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [309] = 17,
    ACTIONS(23), 1,
      aux_sym_block_token1,
    ACTIONS(27), 1,
      anon_sym_xpadd,
    ACTIONS(29), 1,
      anon_sym_xpset,
    ACTIONS(31), 1,
      anon_sym_xpquery,
    ACTIONS(33), 1,
      anon_sym_say,
    ACTIONS(35), 1,
      anon_sym_clear,
    ACTIONS(37), 1,
      anon_sym_eclear,
    ACTIONS(39), 1,
      anon_sym_tellraw,
    ACTIONS(41), 1,
      anon_sym_effect,
    ACTIONS(43), 1,
      anon_sym_time,
    ACTIONS(45), 1,
      anon_sym_gms,
    ACTIONS(47), 1,
      anon_sym_gma,
    ACTIONS(49), 1,
      anon_sym_gmsp,
    ACTIONS(51), 1,
      anon_sym_gmc,
    ACTIONS(53), 1,
      anon_sym_RBRACE,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(69), 14,
      sym__command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [374] = 17,
    ACTIONS(55), 1,
      aux_sym_block_token1,
    ACTIONS(58), 1,
      anon_sym_RBRACE,
    ACTIONS(60), 1,
      anon_sym_xpadd,
    ACTIONS(63), 1,
      anon_sym_xpset,
    ACTIONS(66), 1,
      anon_sym_xpquery,
    ACTIONS(69), 1,
      anon_sym_say,
    ACTIONS(72), 1,
      anon_sym_clear,
    ACTIONS(75), 1,
      anon_sym_eclear,
    ACTIONS(78), 1,
      anon_sym_tellraw,
    ACTIONS(81), 1,
      anon_sym_effect,
    ACTIONS(84), 1,
      anon_sym_time,
    ACTIONS(87), 1,
      anon_sym_gms,
    ACTIONS(90), 1,
      anon_sym_gma,
    ACTIONS(93), 1,
      anon_sym_gmsp,
    ACTIONS(96), 1,
      anon_sym_gmc,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(69), 14,
      sym__command,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [439] = 14,
    ACTIONS(45), 1,
      anon_sym_gms,
    ACTIONS(99), 1,
      anon_sym_xpadd,
    ACTIONS(101), 1,
      anon_sym_xpset,
    ACTIONS(103), 1,
      anon_sym_xpquery,
    ACTIONS(105), 1,
      anon_sym_say,
    ACTIONS(107), 1,
      anon_sym_clear,
    ACTIONS(109), 1,
      anon_sym_eclear,
    ACTIONS(111), 1,
      anon_sym_tellraw,
    ACTIONS(113), 1,
      anon_sym_effect,
    ACTIONS(115), 1,
      anon_sym_time,
    ACTIONS(117), 1,
      anon_sym_gma,
    ACTIONS(119), 1,
      anon_sym_gmsp,
    ACTIONS(121), 1,
      anon_sym_gmc,
    STATE(61), 13,
      sym_xp_add_command,
      sym_xp_set_command,
      sym_xp_query_command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
      sym_gm_survival_command,
      sym_gm_adventure_command,
      sym_gm_spectator_command,
      sym_gm_creative_command,
  [494] = 2,
    ACTIONS(123), 1,
      aux_sym_block_token1,
    ACTIONS(125), 14,
      anon_sym_RBRACE,
      anon_sym_xpadd,
      anon_sym_xpset,
      anon_sym_xpquery,
      anon_sym_say,
      anon_sym_clear,
      anon_sym_eclear,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_time,
      anon_sym_gms,
      anon_sym_gma,
      anon_sym_gmsp,
      anon_sym_gmc,
  [514] = 2,
    ACTIONS(127), 1,
      aux_sym_block_token1,
    ACTIONS(58), 14,
      anon_sym_RBRACE,
      anon_sym_xpadd,
      anon_sym_xpset,
      anon_sym_xpquery,
      anon_sym_say,
      anon_sym_clear,
      anon_sym_eclear,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_time,
      anon_sym_gms,
      anon_sym_gma,
      anon_sym_gmsp,
      anon_sym_gmc,
  [534] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(7), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [548] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(74), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [562] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(43), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [576] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(68), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [590] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(85), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [604] = 3,
    STATE(44), 1,
      sym_selector_type,
    STATE(63), 1,
      sym_target_selector,
    ACTIONS(132), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [618] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(76), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [632] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(75), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [646] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(77), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [660] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(37), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [674] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(41), 1,
      sym_target_selector,
    ACTIONS(130), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [688] = 3,
    ACTIONS(136), 1,
      sym_number,
    STATE(56), 1,
      sym_time_unit,
    ACTIONS(134), 4,
      anon_sym_day,
      anon_sym_night,
      anon_sym_noon,
      anon_sym_midnight,
  [701] = 5,
    ACTIONS(138), 1,
      anon_sym_DOT_DOT,
    ACTIONS(140), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(142), 1,
      sym_identifier,
    ACTIONS(144), 1,
      sym_number,
    STATE(38), 2,
      sym_range_value,
      sym_quoted_string,
  [718] = 3,
    ACTIONS(146), 1,
      ts_builtin_sym_end,
    ACTIONS(148), 1,
      anon_sym_fn,
    STATE(27), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [730] = 3,
    ACTIONS(5), 1,
      anon_sym_fn,
    ACTIONS(151), 1,
      ts_builtin_sym_end,
    STATE(27), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [742] = 4,
    ACTIONS(153), 1,
      anon_sym_RBRACK,
    ACTIONS(155), 1,
      sym_identifier,
    STATE(29), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(36), 1,
      sym_selector_argument,
  [755] = 4,
    ACTIONS(158), 1,
      anon_sym_RBRACK,
    ACTIONS(160), 1,
      sym_identifier,
    STATE(29), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(36), 1,
      sym_selector_argument,
  [768] = 2,
    ACTIONS(164), 1,
      sym_number,
    ACTIONS(162), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [777] = 4,
    ACTIONS(160), 1,
      sym_identifier,
    ACTIONS(166), 1,
      anon_sym_RBRACK,
    STATE(34), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(36), 1,
      sym_selector_argument,
  [790] = 2,
    ACTIONS(170), 1,
      anon_sym_DOT_DOT,
    ACTIONS(168), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [799] = 4,
    ACTIONS(160), 1,
      sym_identifier,
    ACTIONS(172), 1,
      anon_sym_RBRACK,
    STATE(29), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(36), 1,
      sym_selector_argument,
  [812] = 4,
    ACTIONS(160), 1,
      sym_identifier,
    ACTIONS(174), 1,
      anon_sym_RBRACK,
    STATE(30), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(36), 1,
      sym_selector_argument,
  [825] = 2,
    ACTIONS(176), 1,
      anon_sym_COMMA,
    ACTIONS(178), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [833] = 2,
    STATE(60), 1,
      sym_xp_type,
    ACTIONS(180), 2,
      anon_sym_levels,
      anon_sym_points,
  [841] = 1,
    ACTIONS(182), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [847] = 1,
    ACTIONS(184), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [853] = 1,
    ACTIONS(162), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [859] = 2,
    STATE(59), 1,
      sym_xp_type,
    ACTIONS(180), 2,
      anon_sym_levels,
      anon_sym_points,
  [867] = 1,
    ACTIONS(186), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [873] = 2,
    STATE(67), 1,
      sym_xp_type,
    ACTIONS(180), 2,
      anon_sym_levels,
      anon_sym_points,
  [881] = 3,
    ACTIONS(188), 1,
      anon_sym_LBRACK,
    ACTIONS(190), 1,
      sym_text,
    STATE(52), 1,
      sym_selector_arguments,
  [891] = 1,
    ACTIONS(192), 2,
      anon_sym_LBRACK,
      sym_text,
  [896] = 2,
    ACTIONS(194), 1,
      anon_sym_set,
    ACTIONS(196), 1,
      anon_sym_query,
  [903] = 1,
    ACTIONS(198), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [908] = 1,
    ACTIONS(153), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [913] = 2,
    ACTIONS(200), 1,
      anon_sym_LBRACE,
    STATE(47), 1,
      sym_block,
  [920] = 1,
    ACTIONS(202), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [925] = 1,
    ACTIONS(204), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [930] = 1,
    ACTIONS(15), 1,
      sym_text,
  [934] = 1,
    ACTIONS(206), 1,
      sym_number,
  [938] = 1,
    ACTIONS(208), 1,
      sym_number,
  [942] = 1,
    ACTIONS(210), 1,
      anon_sym_SEMI,
  [946] = 1,
    ACTIONS(212), 1,
      anon_sym_SEMI,
  [950] = 1,
    ACTIONS(214), 1,
      anon_sym_SEMI,
  [954] = 1,
    ACTIONS(216), 1,
      sym_identifier,
  [958] = 1,
    ACTIONS(218), 1,
      anon_sym_SEMI,
  [962] = 1,
    ACTIONS(220), 1,
      anon_sym_SEMI,
  [966] = 1,
    ACTIONS(222), 1,
      anon_sym_SEMI,
  [970] = 1,
    ACTIONS(224), 1,
      anon_sym_EQ,
  [974] = 1,
    ACTIONS(226), 1,
      sym_text,
  [978] = 1,
    ACTIONS(228), 1,
      sym_number,
  [982] = 1,
    ACTIONS(230), 1,
      sym_number,
  [986] = 1,
    ACTIONS(232), 1,
      anon_sym_SEMI,
  [990] = 1,
    ACTIONS(234), 1,
      anon_sym_SEMI,
  [994] = 1,
    ACTIONS(236), 1,
      anon_sym_SEMI,
  [998] = 1,
    ACTIONS(238), 1,
      anon_sym_SEMI,
  [1002] = 1,
    ACTIONS(240), 1,
      anon_sym_SEMI,
  [1006] = 1,
    ACTIONS(242), 1,
      sym_number,
  [1010] = 1,
    ACTIONS(244), 1,
      sym_number,
  [1014] = 1,
    ACTIONS(246), 1,
      sym_identifier,
  [1018] = 1,
    ACTIONS(248), 1,
      anon_sym_SEMI,
  [1022] = 1,
    ACTIONS(250), 1,
      anon_sym_SEMI,
  [1026] = 1,
    ACTIONS(252), 1,
      anon_sym_SEMI,
  [1030] = 1,
    ACTIONS(254), 1,
      anon_sym_SEMI,
  [1034] = 1,
    ACTIONS(256), 1,
      ts_builtin_sym_end,
  [1038] = 1,
    ACTIONS(258), 1,
      anon_sym_SEMI,
  [1042] = 1,
    ACTIONS(260), 1,
      sym_number,
  [1046] = 1,
    ACTIONS(13), 1,
      sym_text,
  [1050] = 1,
    ACTIONS(17), 1,
      sym_text,
  [1054] = 1,
    ACTIONS(262), 1,
      sym_text,
  [1058] = 1,
    ACTIONS(264), 1,
      anon_sym_SEMI,
  [1062] = 1,
    ACTIONS(266), 1,
      anon_sym_SEMI,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 45,
  [SMALL_STATE(4)] = 85,
  [SMALL_STATE(5)] = 124,
  [SMALL_STATE(6)] = 163,
  [SMALL_STATE(7)] = 202,
  [SMALL_STATE(8)] = 244,
  [SMALL_STATE(9)] = 309,
  [SMALL_STATE(10)] = 374,
  [SMALL_STATE(11)] = 439,
  [SMALL_STATE(12)] = 494,
  [SMALL_STATE(13)] = 514,
  [SMALL_STATE(14)] = 534,
  [SMALL_STATE(15)] = 548,
  [SMALL_STATE(16)] = 562,
  [SMALL_STATE(17)] = 576,
  [SMALL_STATE(18)] = 590,
  [SMALL_STATE(19)] = 604,
  [SMALL_STATE(20)] = 618,
  [SMALL_STATE(21)] = 632,
  [SMALL_STATE(22)] = 646,
  [SMALL_STATE(23)] = 660,
  [SMALL_STATE(24)] = 674,
  [SMALL_STATE(25)] = 688,
  [SMALL_STATE(26)] = 701,
  [SMALL_STATE(27)] = 718,
  [SMALL_STATE(28)] = 730,
  [SMALL_STATE(29)] = 742,
  [SMALL_STATE(30)] = 755,
  [SMALL_STATE(31)] = 768,
  [SMALL_STATE(32)] = 777,
  [SMALL_STATE(33)] = 790,
  [SMALL_STATE(34)] = 799,
  [SMALL_STATE(35)] = 812,
  [SMALL_STATE(36)] = 825,
  [SMALL_STATE(37)] = 833,
  [SMALL_STATE(38)] = 841,
  [SMALL_STATE(39)] = 847,
  [SMALL_STATE(40)] = 853,
  [SMALL_STATE(41)] = 859,
  [SMALL_STATE(42)] = 867,
  [SMALL_STATE(43)] = 873,
  [SMALL_STATE(44)] = 881,
  [SMALL_STATE(45)] = 891,
  [SMALL_STATE(46)] = 896,
  [SMALL_STATE(47)] = 903,
  [SMALL_STATE(48)] = 908,
  [SMALL_STATE(49)] = 913,
  [SMALL_STATE(50)] = 920,
  [SMALL_STATE(51)] = 925,
  [SMALL_STATE(52)] = 930,
  [SMALL_STATE(53)] = 934,
  [SMALL_STATE(54)] = 938,
  [SMALL_STATE(55)] = 942,
  [SMALL_STATE(56)] = 946,
  [SMALL_STATE(57)] = 950,
  [SMALL_STATE(58)] = 954,
  [SMALL_STATE(59)] = 958,
  [SMALL_STATE(60)] = 962,
  [SMALL_STATE(61)] = 966,
  [SMALL_STATE(62)] = 970,
  [SMALL_STATE(63)] = 974,
  [SMALL_STATE(64)] = 978,
  [SMALL_STATE(65)] = 982,
  [SMALL_STATE(66)] = 986,
  [SMALL_STATE(67)] = 990,
  [SMALL_STATE(68)] = 994,
  [SMALL_STATE(69)] = 998,
  [SMALL_STATE(70)] = 1002,
  [SMALL_STATE(71)] = 1006,
  [SMALL_STATE(72)] = 1010,
  [SMALL_STATE(73)] = 1014,
  [SMALL_STATE(74)] = 1018,
  [SMALL_STATE(75)] = 1022,
  [SMALL_STATE(76)] = 1026,
  [SMALL_STATE(77)] = 1030,
  [SMALL_STATE(78)] = 1034,
  [SMALL_STATE(79)] = 1038,
  [SMALL_STATE(80)] = 1042,
  [SMALL_STATE(81)] = 1046,
  [SMALL_STATE(82)] = 1050,
  [SMALL_STATE(83)] = 1054,
  [SMALL_STATE(84)] = 1058,
  [SMALL_STATE(85)] = 1062,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [7] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 1, 0, 2),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [11] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_type, 1, 0, 0),
  [13] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [15] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 2, 0, 2),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(50),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(72),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(83),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [43] = {.entry = {.count = 1, .reusable = false}}, SHIFT(46),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [49] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [51] = {.entry = {.count = 1, .reusable = false}}, SHIFT(22),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [55] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [58] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [60] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(64),
  [63] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(72),
  [66] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [69] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(83),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [75] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(18),
  [78] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(19),
  [81] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(14),
  [84] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(46),
  [87] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(15),
  [90] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [93] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [96] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(22),
  [99] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [107] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [109] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [111] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [115] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [117] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [121] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [123] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [125] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [127] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [130] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [132] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [134] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [136] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [138] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [140] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [142] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [144] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [146] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [148] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(58),
  [151] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [153] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0),
  [155] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0), SHIFT_REPEAT(62),
  [158] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [160] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [162] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 2, 0, 0),
  [164] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [166] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [168] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 1, 0, 0),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [172] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [174] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [176] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [178] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 1, 0, 0),
  [180] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [182] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_argument, 3, 0, 11),
  [184] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted_string, 1, 0, 0),
  [186] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 3, 0, 0),
  [188] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [190] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 1, 0, 2),
  [192] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_type, 1, 0, 0),
  [194] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [196] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [198] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 3, 0, 1),
  [200] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [202] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [204] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [206] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_effect, 1, 0, 0),
  [208] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [210] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_unit, 1, 0, 0),
  [212] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 7),
  [214] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 8),
  [216] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [218] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_add_command, 4, 0, 9),
  [220] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_set_command, 4, 0, 9),
  [222] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command, 2, 0, 0),
  [224] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [226] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [230] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [232] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_type, 1, 0, 0),
  [234] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_xp_query_command, 3, 0, 5),
  [236] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inv_clear_command, 2, 0, 4),
  [238] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [240] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 5, 0, 10),
  [242] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [244] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [246] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [248] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_survival_command, 2, 0, 4),
  [250] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_adventure_command, 2, 0, 4),
  [252] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_spectator_command, 2, 0, 4),
  [254] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_gm_creative_command, 2, 0, 4),
  [256] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [258] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tellraw_command, 3, 0, 6),
  [260] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_effect, 1, 0, 0),
  [262] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [264] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_say_command, 2, 0, 3),
  [266] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_clear_command, 2, 0, 4),
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
