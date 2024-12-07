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
#define STATE_COUNT 62
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 83
#define ALIAS_COUNT 0
#define TOKEN_COUNT 62
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 11
#define MAX_ALIAS_SEQUENCE_LENGTH 5
#define PRODUCTION_ID_COUNT 9

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
  anon_sym_DOT_DOT = 15,
  aux_sym_quoted_string_token1 = 16,
  anon_sym_say = 17,
  anon_sym_tellraw = 18,
  anon_sym_effect = 19,
  anon_sym_minecraft_COLONspeed = 20,
  anon_sym_minecraft_COLONslowness = 21,
  anon_sym_minecraft_COLONhaste = 22,
  anon_sym_minecraft_COLONmining_fatigue = 23,
  anon_sym_minecraft_COLONstrength = 24,
  anon_sym_minecraft_COLONinstant_health = 25,
  anon_sym_minecraft_COLONinstant_damage = 26,
  anon_sym_minecraft_COLONjump_boost = 27,
  anon_sym_minecraft_COLONnausea = 28,
  anon_sym_minecraft_COLONregeneration = 29,
  anon_sym_minecraft_COLONresistance = 30,
  anon_sym_minecraft_COLONfire_resistance = 31,
  anon_sym_minecraft_COLONwater_breathing = 32,
  anon_sym_minecraft_COLONinvisibility = 33,
  anon_sym_minecraft_COLONblindness = 34,
  anon_sym_minecraft_COLONnight_vision = 35,
  anon_sym_minecraft_COLONhunger = 36,
  anon_sym_minecraft_COLONweakness = 37,
  anon_sym_minecraft_COLONpoison = 38,
  anon_sym_minecraft_COLONwither = 39,
  anon_sym_minecraft_COLONhealth_boost = 40,
  anon_sym_minecraft_COLONabsorption = 41,
  anon_sym_minecraft_COLONsaturation = 42,
  anon_sym_minecraft_COLONglowing = 43,
  anon_sym_minecraft_COLONlevitation = 44,
  anon_sym_minecraft_COLONluck = 45,
  anon_sym_minecraft_COLONunluck = 46,
  anon_sym_minecraft_COLONslow_falling = 47,
  anon_sym_minecraft_COLONconduit_power = 48,
  anon_sym_minecraft_COLONdolphins_grace = 49,
  anon_sym_minecraft_COLONbad_omen = 50,
  anon_sym_minecraft_COLONhero_of_the_village = 51,
  sym_text = 52,
  anon_sym_time = 53,
  anon_sym_set = 54,
  anon_sym_query = 55,
  anon_sym_day = 56,
  anon_sym_night = 57,
  anon_sym_noon = 58,
  anon_sym_midnight = 59,
  sym_identifier = 60,
  sym_number = 61,
  sym_source_file = 62,
  sym__definition = 63,
  sym_function_definition = 64,
  sym_block = 65,
  sym__command = 66,
  sym_target_selector = 67,
  sym_selector_type = 68,
  sym_selector_arguments = 69,
  sym_selector_argument = 70,
  sym_range_value = 71,
  sym_quoted_string = 72,
  sym_say_command = 73,
  sym_tellraw_command = 74,
  sym_effect_command = 75,
  sym_vanilla_effect = 76,
  sym_custom_effect = 77,
  sym_time_command = 78,
  sym_time_unit = 79,
  aux_sym_source_file_repeat1 = 80,
  aux_sym_block_repeat1 = 81,
  aux_sym_selector_arguments_repeat1 = 82,
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
  [anon_sym_DOT_DOT] = "..",
  [aux_sym_quoted_string_token1] = "quoted_string_token1",
  [anon_sym_say] = "say",
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
  [sym_range_value] = "range_value",
  [sym_quoted_string] = "quoted_string",
  [sym_say_command] = "say_command",
  [sym_tellraw_command] = "tellraw_command",
  [sym_effect_command] = "effect_command",
  [sym_vanilla_effect] = "vanilla_effect",
  [sym_custom_effect] = "custom_effect",
  [sym_time_command] = "time_command",
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
  [anon_sym_DOT_DOT] = anon_sym_DOT_DOT,
  [aux_sym_quoted_string_token1] = aux_sym_quoted_string_token1,
  [anon_sym_say] = anon_sym_say,
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
  [sym_range_value] = sym_range_value,
  [sym_quoted_string] = sym_quoted_string,
  [sym_say_command] = sym_say_command,
  [sym_tellraw_command] = sym_tellraw_command,
  [sym_effect_command] = sym_effect_command,
  [sym_vanilla_effect] = sym_vanilla_effect,
  [sym_custom_effect] = sym_custom_effect,
  [sym_time_command] = sym_time_command,
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
  [anon_sym_DOT_DOT] = {
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
  [sym_range_value] = {
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
  field_amplifier = 1,
  field_body = 2,
  field_duration = 3,
  field_effect_type = 4,
  field_key = 5,
  field_message = 6,
  field_name = 7,
  field_query_type = 8,
  field_selector = 9,
  field_target = 10,
  field_value = 11,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_amplifier] = "amplifier",
  [field_body] = "body",
  [field_duration] = "duration",
  [field_effect_type] = "effect_type",
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
  [4] = {.index = 4, .length = 2},
  [5] = {.index = 6, .length = 1},
  [6] = {.index = 7, .length = 1},
  [7] = {.index = 8, .length = 4},
  [8] = {.index = 12, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_body, 2},
    {field_name, 1},
  [2] =
    {field_message, 1},
  [3] =
    {field_selector, 0},
  [4] =
    {field_message, 2},
    {field_target, 1},
  [6] =
    {field_value, 2},
  [7] =
    {field_query_type, 2},
  [8] =
    {field_amplifier, 4},
    {field_duration, 3},
    {field_effect_type, 2},
    {field_target, 1},
  [12] =
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
  [25] = 20,
  [26] = 23,
  [27] = 27,
  [28] = 3,
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
  [39] = 4,
  [40] = 40,
  [41] = 41,
  [42] = 6,
  [43] = 43,
  [44] = 5,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 7,
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
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(311);
      ADVANCE_MAP(
        '"', 2,
        ',', 324,
        '.', 3,
        ';', 314,
        '=', 326,
        '@', 19,
        '[', 322,
        ']', 325,
        'd', 20,
        'e', 100,
        'f', 177,
        'm', 130,
        'n', 131,
        'q', 293,
        's', 26,
        't', 67,
        '{', 313,
        '}', 316,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(375);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(2);
      if (lookahead == ',') ADVANCE(324);
      if (lookahead == '.') ADVANCE(3);
      if (lookahead == ']') ADVANCE(325);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(375);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(374);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(328);
      if (lookahead != 0) ADVANCE(2);
      END_STATE();
    case 3:
      if (lookahead == '.') ADVANCE(327);
      END_STATE();
    case 4:
      if (lookahead == ':') ADVANCE(22);
      END_STATE();
    case 5:
      if (lookahead == '_') ADVANCE(64);
      END_STATE();
    case 6:
      if (lookahead == '_') ADVANCE(49);
      END_STATE();
    case 7:
      if (lookahead == '_') ADVANCE(103);
      if (lookahead == 'n') ADVANCE(83);
      END_STATE();
    case 8:
      if (lookahead == '_') ADVANCE(51);
      END_STATE();
    case 9:
      if (lookahead == '_') ADVANCE(228);
      END_STATE();
    case 10:
      if (lookahead == '_') ADVANCE(112);
      END_STATE();
    case 11:
      if (lookahead == '_') ADVANCE(221);
      END_STATE();
    case 12:
      if (lookahead == '_') ADVANCE(210);
      END_STATE();
    case 13:
      if (lookahead == '_') ADVANCE(279);
      END_STATE();
    case 14:
      if (lookahead == '_') ADVANCE(302);
      END_STATE();
    case 15:
      if (lookahead == '_') ADVANCE(105);
      END_STATE();
    case 16:
      if (lookahead == '_') ADVANCE(301);
      END_STATE();
    case 17:
      if (lookahead == '_') ADVANCE(240);
      END_STATE();
    case 18:
      if (lookahead == '_') ADVANCE(52);
      END_STATE();
    case 19:
      if (lookahead == 'a') ADVANCE(318);
      if (lookahead == 'e') ADVANCE(321);
      if (lookahead == 'p') ADVANCE(319);
      if (lookahead == 'r') ADVANCE(320);
      if (lookahead == 's') ADVANCE(317);
      END_STATE();
    case 20:
      if (lookahead == 'a') ADVANCE(307);
      END_STATE();
    case 21:
      if (lookahead == 'a') ADVANCE(303);
      END_STATE();
    case 22:
      ADVANCE_MAP(
        'a', 48,
        'b', 23,
        'c', 212,
        'd', 208,
        'f', 133,
        'g', 164,
        'h', 27,
        'i', 179,
        'j', 294,
        'l', 71,
        'm', 147,
        'n', 30,
        'p', 213,
        'r', 72,
        's', 32,
        'u', 194,
        'w', 34,
      );
      END_STATE();
    case 23:
      if (lookahead == 'a') ADVANCE(60);
      if (lookahead == 'l') ADVANCE(138);
      END_STATE();
    case 24:
      if (lookahead == 'a') ADVANCE(340);
      END_STATE();
    case 25:
      if (lookahead == 'a') ADVANCE(308);
      END_STATE();
    case 26:
      if (lookahead == 'a') ADVANCE(308);
      if (lookahead == 'e') ADVANCE(264);
      END_STATE();
    case 27:
      if (lookahead == 'a') ADVANCE(250);
      if (lookahead == 'e') ADVANCE(33);
      if (lookahead == 'u') ADVANCE(188);
      END_STATE();
    case 28:
      if (lookahead == 'a') ADVANCE(160);
      END_STATE();
    case 29:
      if (lookahead == 'a') ADVANCE(102);
      END_STATE();
    case 30:
      if (lookahead == 'a') ADVANCE(295);
      if (lookahead == 'i') ADVANCE(119);
      END_STATE();
    case 31:
      if (lookahead == 'a') ADVANCE(175);
      END_STATE();
    case 32:
      if (lookahead == 'a') ADVANCE(272);
      if (lookahead == 'l') ADVANCE(207);
      if (lookahead == 'p') ADVANCE(87);
      if (lookahead == 't') ADVANCE(242);
      END_STATE();
    case 33:
      if (lookahead == 'a') ADVANCE(168);
      if (lookahead == 'r') ADVANCE(222);
      END_STATE();
    case 34:
      if (lookahead == 'a') ADVANCE(286);
      if (lookahead == 'e') ADVANCE(28);
      if (lookahead == 'i') ADVANCE(271);
      END_STATE();
    case 35:
      if (lookahead == 'a') ADVANCE(114);
      END_STATE();
    case 36:
      if (lookahead == 'a') ADVANCE(199);
      END_STATE();
    case 37:
      if (lookahead == 'a') ADVANCE(195);
      END_STATE();
    case 38:
      if (lookahead == 'a') ADVANCE(285);
      END_STATE();
    case 39:
      if (lookahead == 'a') ADVANCE(289);
      END_STATE();
    case 40:
      if (lookahead == 'a') ADVANCE(166);
      END_STATE();
    case 41:
      if (lookahead == 'a') ADVANCE(58);
      END_STATE();
    case 42:
      if (lookahead == 'a') ADVANCE(169);
      END_STATE();
    case 43:
      if (lookahead == 'a') ADVANCE(115);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(202);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(290);
      END_STATE();
    case 46:
      if (lookahead == 'a') ADVANCE(291);
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(292);
      END_STATE();
    case 48:
      if (lookahead == 'b') ADVANCE(248);
      END_STATE();
    case 49:
      if (lookahead == 'b') ADVANCE(214);
      END_STATE();
    case 50:
      if (lookahead == 'b') ADVANCE(137);
      END_STATE();
    case 51:
      if (lookahead == 'b') ADVANCE(241);
      END_STATE();
    case 52:
      if (lookahead == 'b') ADVANCE(225);
      END_STATE();
    case 53:
      if (lookahead == 'c') ADVANCE(158);
      END_STATE();
    case 54:
      if (lookahead == 'c') ADVANCE(266);
      END_STATE();
    case 55:
      if (lookahead == 'c') ADVANCE(234);
      END_STATE();
    case 56:
      if (lookahead == 'c') ADVANCE(159);
      END_STATE();
    case 57:
      if (lookahead == 'c') ADVANCE(74);
      END_STATE();
    case 58:
      if (lookahead == 'c') ADVANCE(75);
      END_STATE();
    case 59:
      if (lookahead == 'c') ADVANCE(78);
      END_STATE();
    case 60:
      if (lookahead == 'd') ADVANCE(11);
      END_STATE();
    case 61:
      if (lookahead == 'd') ADVANCE(332);
      END_STATE();
    case 62:
      if (lookahead == 'd') ADVANCE(187);
      if (lookahead == 'n') ADVANCE(80);
      END_STATE();
    case 63:
      if (lookahead == 'd') ADVANCE(299);
      END_STATE();
    case 64:
      if (lookahead == 'd') ADVANCE(31);
      if (lookahead == 'h') ADVANCE(89);
      END_STATE();
    case 65:
      if (lookahead == 'd') ADVANCE(203);
      END_STATE();
    case 66:
      if (lookahead == 'e') ADVANCE(100);
      if (lookahead == 's') ADVANCE(25);
      if (lookahead == 't') ADVANCE(67);
      if (lookahead == '}') ADVANCE(316);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(315);
      END_STATE();
    case 67:
      if (lookahead == 'e') ADVANCE(162);
      if (lookahead == 'i') ADVANCE(174);
      END_STATE();
    case 68:
      if (lookahead == 'e') ADVANCE(235);
      END_STATE();
    case 69:
      if (lookahead == 'e') ADVANCE(54);
      END_STATE();
    case 70:
      if (lookahead == 'e') ADVANCE(367);
      END_STATE();
    case 71:
      if (lookahead == 'e') ADVANCE(300);
      if (lookahead == 'u') ADVANCE(53);
      END_STATE();
    case 72:
      if (lookahead == 'e') ADVANCE(111);
      END_STATE();
    case 73:
      if (lookahead == 'e') ADVANCE(334);
      END_STATE();
    case 74:
      if (lookahead == 'e') ADVANCE(342);
      END_STATE();
    case 75:
      if (lookahead == 'e') ADVANCE(361);
      END_STATE();
    case 76:
      if (lookahead == 'e') ADVANCE(338);
      END_STATE();
    case 77:
      if (lookahead == 'e') ADVANCE(335);
      END_STATE();
    case 78:
      if (lookahead == 'e') ADVANCE(343);
      END_STATE();
    case 79:
      if (lookahead == 'e') ADVANCE(363);
      END_STATE();
    case 80:
      if (lookahead == 'e') ADVANCE(55);
      END_STATE();
    case 81:
      if (lookahead == 'e') ADVANCE(17);
      END_STATE();
    case 82:
      if (lookahead == 'e') ADVANCE(61);
      END_STATE();
    case 83:
      if (lookahead == 'e') ADVANCE(249);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(24);
      END_STATE();
    case 85:
      if (lookahead == 'e') ADVANCE(237);
      END_STATE();
    case 86:
      if (lookahead == 'e') ADVANCE(230);
      END_STATE();
    case 87:
      if (lookahead == 'e') ADVANCE(82);
      END_STATE();
    case 88:
      if (lookahead == 'e') ADVANCE(231);
      END_STATE();
    case 89:
      if (lookahead == 'e') ADVANCE(42);
      END_STATE();
    case 90:
      if (lookahead == 'e') ADVANCE(204);
      END_STATE();
    case 91:
      if (lookahead == 'e') ADVANCE(192);
      END_STATE();
    case 92:
      if (lookahead == 'e') ADVANCE(232);
      END_STATE();
    case 93:
      if (lookahead == 'e') ADVANCE(181);
      END_STATE();
    case 94:
      if (lookahead == 'e') ADVANCE(263);
      END_STATE();
    case 95:
      if (lookahead == 'e') ADVANCE(251);
      END_STATE();
    case 96:
      if (lookahead == 'e') ADVANCE(39);
      END_STATE();
    case 97:
      if (lookahead == 'e') ADVANCE(254);
      END_STATE();
    case 98:
      if (lookahead == 'e') ADVANCE(16);
      END_STATE();
    case 99:
      if (lookahead == 'e') ADVANCE(244);
      END_STATE();
    case 100:
      if (lookahead == 'f') ADVANCE(101);
      END_STATE();
    case 101:
      if (lookahead == 'f') ADVANCE(69);
      END_STATE();
    case 102:
      if (lookahead == 'f') ADVANCE(268);
      END_STATE();
    case 103:
      if (lookahead == 'f') ADVANCE(40);
      END_STATE();
    case 104:
      if (lookahead == 'f') ADVANCE(13);
      END_STATE();
    case 105:
      if (lookahead == 'f') ADVANCE(38);
      END_STATE();
    case 106:
      if (lookahead == 'g') ADVANCE(122);
      END_STATE();
    case 107:
      if (lookahead == 'g') ADVANCE(355);
      END_STATE();
    case 108:
      if (lookahead == 'g') ADVANCE(359);
      END_STATE();
    case 109:
      if (lookahead == 'g') ADVANCE(344);
      END_STATE();
    case 110:
      if (lookahead == 'g') ADVANCE(15);
      END_STATE();
    case 111:
      if (lookahead == 'g') ADVANCE(90);
      if (lookahead == 's') ADVANCE(148);
      END_STATE();
    case 112:
      if (lookahead == 'g') ADVANCE(239);
      END_STATE();
    case 113:
      if (lookahead == 'g') ADVANCE(277);
      END_STATE();
    case 114:
      if (lookahead == 'g') ADVANCE(76);
      END_STATE();
    case 115:
      if (lookahead == 'g') ADVANCE(79);
      END_STATE();
    case 116:
      if (lookahead == 'g') ADVANCE(298);
      END_STATE();
    case 117:
      if (lookahead == 'g') ADVANCE(123);
      END_STATE();
    case 118:
      if (lookahead == 'g') ADVANCE(86);
      END_STATE();
    case 119:
      if (lookahead == 'g') ADVANCE(124);
      END_STATE();
    case 120:
      if (lookahead == 'h') ADVANCE(336);
      END_STATE();
    case 121:
      if (lookahead == 'h') ADVANCE(337);
      END_STATE();
    case 122:
      if (lookahead == 'h') ADVANCE(265);
      END_STATE();
    case 123:
      if (lookahead == 'h') ADVANCE(267);
      END_STATE();
    case 124:
      if (lookahead == 'h') ADVANCE(275);
      END_STATE();
    case 125:
      if (lookahead == 'h') ADVANCE(98);
      END_STATE();
    case 126:
      if (lookahead == 'h') ADVANCE(88);
      END_STATE();
    case 127:
      if (lookahead == 'h') ADVANCE(143);
      END_STATE();
    case 128:
      if (lookahead == 'h') ADVANCE(146);
      END_STATE();
    case 129:
      if (lookahead == 'h') ADVANCE(18);
      END_STATE();
    case 130:
      if (lookahead == 'i') ADVANCE(62);
      END_STATE();
    case 131:
      if (lookahead == 'i') ADVANCE(106);
      if (lookahead == 'o') ADVANCE(206);
      END_STATE();
    case 132:
      if (lookahead == 'i') ADVANCE(50);
      END_STATE();
    case 133:
      if (lookahead == 'i') ADVANCE(238);
      END_STATE();
    case 134:
      if (lookahead == 'i') ADVANCE(260);
      END_STATE();
    case 135:
      if (lookahead == 'i') ADVANCE(252);
      END_STATE();
    case 136:
      if (lookahead == 'i') ADVANCE(116);
      END_STATE();
    case 137:
      if (lookahead == 'i') ADVANCE(171);
      END_STATE();
    case 138:
      if (lookahead == 'i') ADVANCE(191);
      END_STATE();
    case 139:
      if (lookahead == 'i') ADVANCE(190);
      END_STATE();
    case 140:
      if (lookahead == 'i') ADVANCE(278);
      END_STATE();
    case 141:
      if (lookahead == 'i') ADVANCE(193);
      END_STATE();
    case 142:
      if (lookahead == 'i') ADVANCE(281);
      END_STATE();
    case 143:
      if (lookahead == 'i') ADVANCE(196);
      END_STATE();
    case 144:
      if (lookahead == 'i') ADVANCE(273);
      END_STATE();
    case 145:
      if (lookahead == 'i') ADVANCE(197);
      END_STATE();
    case 146:
      if (lookahead == 'i') ADVANCE(198);
      END_STATE();
    case 147:
      if (lookahead == 'i') ADVANCE(200);
      END_STATE();
    case 148:
      if (lookahead == 'i') ADVANCE(259);
      END_STATE();
    case 149:
      if (lookahead == 'i') ADVANCE(167);
      END_STATE();
    case 150:
      if (lookahead == 'i') ADVANCE(262);
      END_STATE();
    case 151:
      if (lookahead == 'i') ADVANCE(216);
      END_STATE();
    case 152:
      if (lookahead == 'i') ADVANCE(217);
      END_STATE();
    case 153:
      if (lookahead == 'i') ADVANCE(218);
      END_STATE();
    case 154:
      if (lookahead == 'i') ADVANCE(219);
      END_STATE();
    case 155:
      if (lookahead == 'i') ADVANCE(220);
      END_STATE();
    case 156:
      if (lookahead == 'i') ADVANCE(117);
      END_STATE();
    case 157:
      if (lookahead == 'i') ADVANCE(261);
      END_STATE();
    case 158:
      if (lookahead == 'k') ADVANCE(357);
      END_STATE();
    case 159:
      if (lookahead == 'k') ADVANCE(358);
      END_STATE();
    case 160:
      if (lookahead == 'k') ADVANCE(201);
      END_STATE();
    case 161:
      if (lookahead == 'l') ADVANCE(226);
      END_STATE();
    case 162:
      if (lookahead == 'l') ADVANCE(163);
      END_STATE();
    case 163:
      if (lookahead == 'l') ADVANCE(233);
      END_STATE();
    case 164:
      if (lookahead == 'l') ADVANCE(205);
      END_STATE();
    case 165:
      if (lookahead == 'l') ADVANCE(296);
      END_STATE();
    case 166:
      if (lookahead == 'l') ADVANCE(172);
      END_STATE();
    case 167:
      if (lookahead == 'l') ADVANCE(170);
      END_STATE();
    case 168:
      if (lookahead == 'l') ADVANCE(274);
      END_STATE();
    case 169:
      if (lookahead == 'l') ADVANCE(280);
      END_STATE();
    case 170:
      if (lookahead == 'l') ADVANCE(43);
      END_STATE();
    case 171:
      if (lookahead == 'l') ADVANCE(144);
      END_STATE();
    case 172:
      if (lookahead == 'l') ADVANCE(145);
      END_STATE();
    case 173:
      if (lookahead == 'm') ADVANCE(227);
      END_STATE();
    case 174:
      if (lookahead == 'm') ADVANCE(70);
      END_STATE();
    case 175:
      if (lookahead == 'm') ADVANCE(35);
      END_STATE();
    case 176:
      if (lookahead == 'm') ADVANCE(93);
      END_STATE();
    case 177:
      if (lookahead == 'n') ADVANCE(312);
      END_STATE();
    case 178:
      if (lookahead == 'n') ADVANCE(372);
      END_STATE();
    case 179:
      if (lookahead == 'n') ADVANCE(253);
      END_STATE();
    case 180:
      if (lookahead == 'n') ADVANCE(350);
      END_STATE();
    case 181:
      if (lookahead == 'n') ADVANCE(362);
      END_STATE();
    case 182:
      if (lookahead == 'n') ADVANCE(353);
      END_STATE();
    case 183:
      if (lookahead == 'n') ADVANCE(356);
      END_STATE();
    case 184:
      if (lookahead == 'n') ADVANCE(354);
      END_STATE();
    case 185:
      if (lookahead == 'n') ADVANCE(347);
      END_STATE();
    case 186:
      if (lookahead == 'n') ADVANCE(341);
      END_STATE();
    case 187:
      if (lookahead == 'n') ADVANCE(156);
      END_STATE();
    case 188:
      if (lookahead == 'n') ADVANCE(118);
      END_STATE();
    case 189:
      if (lookahead == 'n') ADVANCE(63);
      END_STATE();
    case 190:
      if (lookahead == 'n') ADVANCE(110);
      END_STATE();
    case 191:
      if (lookahead == 'n') ADVANCE(65);
      END_STATE();
    case 192:
      if (lookahead == 'n') ADVANCE(113);
      END_STATE();
    case 193:
      if (lookahead == 'n') ADVANCE(107);
      END_STATE();
    case 194:
      if (lookahead == 'n') ADVANCE(165);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(57);
      END_STATE();
    case 196:
      if (lookahead == 'n') ADVANCE(255);
      END_STATE();
    case 197:
      if (lookahead == 'n') ADVANCE(108);
      END_STATE();
    case 198:
      if (lookahead == 'n') ADVANCE(109);
      END_STATE();
    case 199:
      if (lookahead == 'n') ADVANCE(282);
      END_STATE();
    case 200:
      if (lookahead == 'n') ADVANCE(139);
      END_STATE();
    case 201:
      if (lookahead == 'n') ADVANCE(95);
      END_STATE();
    case 202:
      if (lookahead == 'n') ADVANCE(59);
      END_STATE();
    case 203:
      if (lookahead == 'n') ADVANCE(97);
      END_STATE();
    case 204:
      if (lookahead == 'n') ADVANCE(99);
      END_STATE();
    case 205:
      if (lookahead == 'o') ADVANCE(305);
      END_STATE();
    case 206:
      if (lookahead == 'o') ADVANCE(178);
      END_STATE();
    case 207:
      if (lookahead == 'o') ADVANCE(304);
      END_STATE();
    case 208:
      if (lookahead == 'o') ADVANCE(161);
      END_STATE();
    case 209:
      if (lookahead == 'o') ADVANCE(306);
      END_STATE();
    case 210:
      if (lookahead == 'o') ADVANCE(104);
      END_STATE();
    case 211:
      if (lookahead == 'o') ADVANCE(236);
      END_STATE();
    case 212:
      if (lookahead == 'o') ADVANCE(189);
      END_STATE();
    case 213:
      if (lookahead == 'o') ADVANCE(134);
      END_STATE();
    case 214:
      if (lookahead == 'o') ADVANCE(223);
      END_STATE();
    case 215:
      if (lookahead == 'o') ADVANCE(180);
      END_STATE();
    case 216:
      if (lookahead == 'o') ADVANCE(182);
      END_STATE();
    case 217:
      if (lookahead == 'o') ADVANCE(183);
      END_STATE();
    case 218:
      if (lookahead == 'o') ADVANCE(184);
      END_STATE();
    case 219:
      if (lookahead == 'o') ADVANCE(185);
      END_STATE();
    case 220:
      if (lookahead == 'o') ADVANCE(186);
      END_STATE();
    case 221:
      if (lookahead == 'o') ADVANCE(176);
      END_STATE();
    case 222:
      if (lookahead == 'o') ADVANCE(12);
      END_STATE();
    case 223:
      if (lookahead == 'o') ADVANCE(257);
      END_STATE();
    case 224:
      if (lookahead == 'o') ADVANCE(258);
      END_STATE();
    case 225:
      if (lookahead == 'o') ADVANCE(224);
      END_STATE();
    case 226:
      if (lookahead == 'p') ADVANCE(127);
      END_STATE();
    case 227:
      if (lookahead == 'p') ADVANCE(6);
      END_STATE();
    case 228:
      if (lookahead == 'p') ADVANCE(209);
      END_STATE();
    case 229:
      if (lookahead == 'p') ADVANCE(284);
      END_STATE();
    case 230:
      if (lookahead == 'r') ADVANCE(348);
      END_STATE();
    case 231:
      if (lookahead == 'r') ADVANCE(351);
      END_STATE();
    case 232:
      if (lookahead == 'r') ADVANCE(360);
      END_STATE();
    case 233:
      if (lookahead == 'r') ADVANCE(21);
      END_STATE();
    case 234:
      if (lookahead == 'r') ADVANCE(29);
      END_STATE();
    case 235:
      if (lookahead == 'r') ADVANCE(309);
      END_STATE();
    case 236:
      if (lookahead == 'r') ADVANCE(229);
      END_STATE();
    case 237:
      if (lookahead == 'r') ADVANCE(8);
      END_STATE();
    case 238:
      if (lookahead == 'r') ADVANCE(81);
      END_STATE();
    case 239:
      if (lookahead == 'r') ADVANCE(41);
      END_STATE();
    case 240:
      if (lookahead == 'r') ADVANCE(94);
      END_STATE();
    case 241:
      if (lookahead == 'r') ADVANCE(96);
      END_STATE();
    case 242:
      if (lookahead == 'r') ADVANCE(91);
      END_STATE();
    case 243:
      if (lookahead == 'r') ADVANCE(46);
      END_STATE();
    case 244:
      if (lookahead == 'r') ADVANCE(47);
      END_STATE();
    case 245:
      if (lookahead == 's') ADVANCE(333);
      END_STATE();
    case 246:
      if (lookahead == 's') ADVANCE(349);
      END_STATE();
    case 247:
      if (lookahead == 's') ADVANCE(346);
      END_STATE();
    case 248:
      if (lookahead == 's') ADVANCE(211);
      END_STATE();
    case 249:
      if (lookahead == 's') ADVANCE(245);
      END_STATE();
    case 250:
      if (lookahead == 's') ADVANCE(283);
      END_STATE();
    case 251:
      if (lookahead == 's') ADVANCE(246);
      END_STATE();
    case 252:
      if (lookahead == 's') ADVANCE(132);
      END_STATE();
    case 253:
      if (lookahead == 's') ADVANCE(276);
      if (lookahead == 'v') ADVANCE(135);
      END_STATE();
    case 254:
      if (lookahead == 's') ADVANCE(247);
      END_STATE();
    case 255:
      if (lookahead == 's') ADVANCE(10);
      END_STATE();
    case 256:
      if (lookahead == 's') ADVANCE(84);
      END_STATE();
    case 257:
      if (lookahead == 's') ADVANCE(269);
      END_STATE();
    case 258:
      if (lookahead == 's') ADVANCE(270);
      END_STATE();
    case 259:
      if (lookahead == 's') ADVANCE(287);
      END_STATE();
    case 260:
      if (lookahead == 's') ADVANCE(215);
      END_STATE();
    case 261:
      if (lookahead == 's') ADVANCE(288);
      END_STATE();
    case 262:
      if (lookahead == 's') ADVANCE(154);
      END_STATE();
    case 263:
      if (lookahead == 's') ADVANCE(157);
      END_STATE();
    case 264:
      if (lookahead == 't') ADVANCE(368);
      END_STATE();
    case 265:
      if (lookahead == 't') ADVANCE(371);
      END_STATE();
    case 266:
      if (lookahead == 't') ADVANCE(331);
      END_STATE();
    case 267:
      if (lookahead == 't') ADVANCE(373);
      END_STATE();
    case 268:
      if (lookahead == 't') ADVANCE(4);
      END_STATE();
    case 269:
      if (lookahead == 't') ADVANCE(339);
      END_STATE();
    case 270:
      if (lookahead == 't') ADVANCE(352);
      END_STATE();
    case 271:
      if (lookahead == 't') ADVANCE(126);
      END_STATE();
    case 272:
      if (lookahead == 't') ADVANCE(297);
      END_STATE();
    case 273:
      if (lookahead == 't') ADVANCE(310);
      END_STATE();
    case 274:
      if (lookahead == 't') ADVANCE(129);
      END_STATE();
    case 275:
      if (lookahead == 't') ADVANCE(14);
      END_STATE();
    case 276:
      if (lookahead == 't') ADVANCE(36);
      END_STATE();
    case 277:
      if (lookahead == 't') ADVANCE(120);
      END_STATE();
    case 278:
      if (lookahead == 't') ADVANCE(45);
      END_STATE();
    case 279:
      if (lookahead == 't') ADVANCE(125);
      END_STATE();
    case 280:
      if (lookahead == 't') ADVANCE(121);
      END_STATE();
    case 281:
      if (lookahead == 't') ADVANCE(9);
      END_STATE();
    case 282:
      if (lookahead == 't') ADVANCE(5);
      END_STATE();
    case 283:
      if (lookahead == 't') ADVANCE(73);
      END_STATE();
    case 284:
      if (lookahead == 't') ADVANCE(151);
      END_STATE();
    case 285:
      if (lookahead == 't') ADVANCE(136);
      END_STATE();
    case 286:
      if (lookahead == 't') ADVANCE(85);
      END_STATE();
    case 287:
      if (lookahead == 't') ADVANCE(37);
      END_STATE();
    case 288:
      if (lookahead == 't') ADVANCE(44);
      END_STATE();
    case 289:
      if (lookahead == 't') ADVANCE(128);
      END_STATE();
    case 290:
      if (lookahead == 't') ADVANCE(152);
      END_STATE();
    case 291:
      if (lookahead == 't') ADVANCE(153);
      END_STATE();
    case 292:
      if (lookahead == 't') ADVANCE(155);
      END_STATE();
    case 293:
      if (lookahead == 'u') ADVANCE(68);
      END_STATE();
    case 294:
      if (lookahead == 'u') ADVANCE(173);
      END_STATE();
    case 295:
      if (lookahead == 'u') ADVANCE(256);
      END_STATE();
    case 296:
      if (lookahead == 'u') ADVANCE(56);
      END_STATE();
    case 297:
      if (lookahead == 'u') ADVANCE(243);
      END_STATE();
    case 298:
      if (lookahead == 'u') ADVANCE(77);
      END_STATE();
    case 299:
      if (lookahead == 'u') ADVANCE(142);
      END_STATE();
    case 300:
      if (lookahead == 'v') ADVANCE(140);
      END_STATE();
    case 301:
      if (lookahead == 'v') ADVANCE(149);
      END_STATE();
    case 302:
      if (lookahead == 'v') ADVANCE(150);
      END_STATE();
    case 303:
      if (lookahead == 'w') ADVANCE(330);
      END_STATE();
    case 304:
      if (lookahead == 'w') ADVANCE(7);
      END_STATE();
    case 305:
      if (lookahead == 'w') ADVANCE(141);
      END_STATE();
    case 306:
      if (lookahead == 'w') ADVANCE(92);
      END_STATE();
    case 307:
      if (lookahead == 'y') ADVANCE(370);
      END_STATE();
    case 308:
      if (lookahead == 'y') ADVANCE(329);
      END_STATE();
    case 309:
      if (lookahead == 'y') ADVANCE(369);
      END_STATE();
    case 310:
      if (lookahead == 'y') ADVANCE(345);
      END_STATE();
    case 311:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 312:
      ACCEPT_TOKEN(anon_sym_fn);
      END_STATE();
    case 313:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 314:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(aux_sym_block_token1);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(315);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(anon_sym_ATs);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(anon_sym_ATa);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(anon_sym_ATp);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(anon_sym_ATr);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(anon_sym_ATe);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(366);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(aux_sym_quoted_string_token1);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(anon_sym_say);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(anon_sym_tellraw);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(anon_sym_effect);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONspeed);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslowness);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhaste);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmining_fatigue);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONstrength);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_health);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_damage);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONjump_boost);
      END_STATE();
    case 340:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnausea);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONregeneration);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONresistance);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_resistance);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwater_breathing);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinvisibility);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblindness);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnight_vision);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhunger);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONweakness);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpoison);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwither);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhealth_boost);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONabsorption);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsaturation);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONglowing);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlevitation);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunluck);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslow_falling);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONconduit_power);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdolphins_grace);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbad_omen);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhero_of_the_village);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(sym_text);
      if (lookahead == '[') ADVANCE(323);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(364);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(366);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(sym_text);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(365);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(366);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(sym_text);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(366);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(anon_sym_query);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(anon_sym_day);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(anon_sym_night);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(anon_sym_noon);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(anon_sym_midnight);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(374);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(375);
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
  [8] = {.lex_state = 66},
  [9] = {.lex_state = 66},
  [10] = {.lex_state = 66},
  [11] = {.lex_state = 0},
  [12] = {.lex_state = 0},
  [13] = {.lex_state = 0},
  [14] = {.lex_state = 0},
  [15] = {.lex_state = 66},
  [16] = {.lex_state = 66},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 1},
  [21] = {.lex_state = 1},
  [22] = {.lex_state = 1},
  [23] = {.lex_state = 1},
  [24] = {.lex_state = 1},
  [25] = {.lex_state = 1},
  [26] = {.lex_state = 1},
  [27] = {.lex_state = 1},
  [28] = {.lex_state = 364},
  [29] = {.lex_state = 1},
  [30] = {.lex_state = 1},
  [31] = {.lex_state = 1},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 0},
  [35] = {.lex_state = 0},
  [36] = {.lex_state = 0},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 364},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 1},
  [42] = {.lex_state = 365},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 365},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 0},
  [49] = {.lex_state = 365},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 0},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 365},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 0},
  [58] = {.lex_state = 1},
  [59] = {.lex_state = 365},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
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
    [anon_sym_DOT_DOT] = ACTIONS(1),
    [aux_sym_quoted_string_token1] = ACTIONS(1),
    [anon_sym_say] = ACTIONS(1),
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
    [anon_sym_day] = ACTIONS(1),
    [anon_sym_night] = ACTIONS(1),
    [anon_sym_noon] = ACTIONS(1),
    [anon_sym_midnight] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(61),
    [sym__definition] = STATE(19),
    [sym_function_definition] = STATE(19),
    [aux_sym_source_file_repeat1] = STATE(19),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_fn] = ACTIONS(5),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 3,
    ACTIONS(7), 1,
      aux_sym_quoted_string_token1,
    STATE(56), 2,
      sym_vanilla_effect,
      sym_custom_effect,
    ACTIONS(9), 32,
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
  [42] = 3,
    ACTIONS(11), 1,
      anon_sym_LBRACK,
    STATE(5), 1,
      sym_selector_arguments,
    ACTIONS(13), 33,
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
  [84] = 1,
    ACTIONS(15), 34,
      anon_sym_LBRACK,
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
  [121] = 1,
    ACTIONS(17), 33,
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
  [157] = 1,
    ACTIONS(19), 33,
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
  [193] = 1,
    ACTIONS(21), 33,
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
  [229] = 8,
    ACTIONS(23), 1,
      aux_sym_block_token1,
    ACTIONS(25), 1,
      anon_sym_RBRACE,
    ACTIONS(27), 1,
      anon_sym_say,
    ACTIONS(29), 1,
      anon_sym_tellraw,
    ACTIONS(31), 1,
      anon_sym_effect,
    ACTIONS(33), 1,
      anon_sym_time,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(45), 5,
      sym__command,
      sym_say_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [258] = 8,
    ACTIONS(23), 1,
      aux_sym_block_token1,
    ACTIONS(27), 1,
      anon_sym_say,
    ACTIONS(29), 1,
      anon_sym_tellraw,
    ACTIONS(31), 1,
      anon_sym_effect,
    ACTIONS(33), 1,
      anon_sym_time,
    ACTIONS(35), 1,
      anon_sym_RBRACE,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(45), 5,
      sym__command,
      sym_say_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [287] = 8,
    ACTIONS(37), 1,
      aux_sym_block_token1,
    ACTIONS(40), 1,
      anon_sym_RBRACE,
    ACTIONS(42), 1,
      anon_sym_say,
    ACTIONS(45), 1,
      anon_sym_tellraw,
    ACTIONS(48), 1,
      anon_sym_effect,
    ACTIONS(51), 1,
      anon_sym_time,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(45), 5,
      sym__command,
      sym_say_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [316] = 5,
    ACTIONS(54), 1,
      anon_sym_say,
    ACTIONS(56), 1,
      anon_sym_tellraw,
    ACTIONS(58), 1,
      anon_sym_effect,
    ACTIONS(60), 1,
      anon_sym_time,
    STATE(50), 4,
      sym_say_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [335] = 3,
    STATE(2), 1,
      sym_target_selector,
    STATE(3), 1,
      sym_selector_type,
    ACTIONS(62), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [349] = 3,
    STATE(28), 1,
      sym_selector_type,
    STATE(53), 1,
      sym_target_selector,
    ACTIONS(64), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [363] = 3,
    ACTIONS(68), 1,
      sym_number,
    STATE(40), 1,
      sym_time_unit,
    ACTIONS(66), 4,
      anon_sym_day,
      anon_sym_night,
      anon_sym_noon,
      anon_sym_midnight,
  [376] = 2,
    ACTIONS(70), 1,
      aux_sym_block_token1,
    ACTIONS(40), 5,
      anon_sym_RBRACE,
      anon_sym_say,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_time,
  [387] = 2,
    ACTIONS(73), 1,
      aux_sym_block_token1,
    ACTIONS(75), 5,
      anon_sym_RBRACE,
      anon_sym_say,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_time,
  [398] = 5,
    ACTIONS(77), 1,
      anon_sym_DOT_DOT,
    ACTIONS(79), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(81), 1,
      sym_identifier,
    ACTIONS(83), 1,
      sym_number,
    STATE(30), 2,
      sym_range_value,
      sym_quoted_string,
  [415] = 3,
    ACTIONS(85), 1,
      ts_builtin_sym_end,
    ACTIONS(87), 1,
      anon_sym_fn,
    STATE(18), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [427] = 3,
    ACTIONS(5), 1,
      anon_sym_fn,
    ACTIONS(90), 1,
      ts_builtin_sym_end,
    STATE(18), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [439] = 4,
    ACTIONS(92), 1,
      anon_sym_RBRACK,
    ACTIONS(94), 1,
      sym_identifier,
    STATE(23), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(29), 1,
      sym_selector_argument,
  [452] = 2,
    ACTIONS(98), 1,
      sym_number,
    ACTIONS(96), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [461] = 4,
    ACTIONS(100), 1,
      anon_sym_RBRACK,
    ACTIONS(102), 1,
      sym_identifier,
    STATE(22), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(29), 1,
      sym_selector_argument,
  [474] = 4,
    ACTIONS(94), 1,
      sym_identifier,
    ACTIONS(105), 1,
      anon_sym_RBRACK,
    STATE(22), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(29), 1,
      sym_selector_argument,
  [487] = 2,
    ACTIONS(109), 1,
      anon_sym_DOT_DOT,
    ACTIONS(107), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [496] = 4,
    ACTIONS(94), 1,
      sym_identifier,
    ACTIONS(111), 1,
      anon_sym_RBRACK,
    STATE(26), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(29), 1,
      sym_selector_argument,
  [509] = 4,
    ACTIONS(94), 1,
      sym_identifier,
    ACTIONS(113), 1,
      anon_sym_RBRACK,
    STATE(22), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(29), 1,
      sym_selector_argument,
  [522] = 1,
    ACTIONS(115), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [528] = 3,
    ACTIONS(117), 1,
      anon_sym_LBRACK,
    ACTIONS(119), 1,
      sym_text,
    STATE(44), 1,
      sym_selector_arguments,
  [538] = 2,
    ACTIONS(121), 1,
      anon_sym_COMMA,
    ACTIONS(123), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [546] = 1,
    ACTIONS(125), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [552] = 1,
    ACTIONS(96), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [558] = 1,
    ACTIONS(127), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [564] = 1,
    ACTIONS(100), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [569] = 1,
    ACTIONS(129), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [574] = 2,
    ACTIONS(131), 1,
      anon_sym_set,
    ACTIONS(133), 1,
      anon_sym_query,
  [581] = 1,
    ACTIONS(135), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [586] = 2,
    ACTIONS(137), 1,
      anon_sym_LBRACE,
    STATE(34), 1,
      sym_block,
  [593] = 1,
    ACTIONS(139), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [598] = 1,
    ACTIONS(141), 2,
      anon_sym_LBRACK,
      sym_text,
  [603] = 1,
    ACTIONS(143), 1,
      anon_sym_SEMI,
  [607] = 1,
    ACTIONS(145), 1,
      sym_identifier,
  [611] = 1,
    ACTIONS(19), 1,
      sym_text,
  [615] = 1,
    ACTIONS(147), 1,
      anon_sym_EQ,
  [619] = 1,
    ACTIONS(17), 1,
      sym_text,
  [623] = 1,
    ACTIONS(149), 1,
      anon_sym_SEMI,
  [627] = 1,
    ACTIONS(151), 1,
      sym_number,
  [631] = 1,
    ACTIONS(153), 1,
      anon_sym_SEMI,
  [635] = 1,
    ACTIONS(155), 1,
      anon_sym_SEMI,
  [639] = 1,
    ACTIONS(21), 1,
      sym_text,
  [643] = 1,
    ACTIONS(157), 1,
      anon_sym_SEMI,
  [647] = 1,
    ACTIONS(159), 1,
      anon_sym_SEMI,
  [651] = 1,
    ACTIONS(161), 1,
      sym_number,
  [655] = 1,
    ACTIONS(163), 1,
      sym_text,
  [659] = 1,
    ACTIONS(165), 1,
      anon_sym_SEMI,
  [663] = 1,
    ACTIONS(167), 1,
      anon_sym_SEMI,
  [667] = 1,
    ACTIONS(169), 1,
      sym_number,
  [671] = 1,
    ACTIONS(171), 1,
      sym_number,
  [675] = 1,
    ACTIONS(173), 1,
      sym_identifier,
  [679] = 1,
    ACTIONS(175), 1,
      sym_text,
  [683] = 1,
    ACTIONS(177), 1,
      sym_number,
  [687] = 1,
    ACTIONS(179), 1,
      ts_builtin_sym_end,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 42,
  [SMALL_STATE(4)] = 84,
  [SMALL_STATE(5)] = 121,
  [SMALL_STATE(6)] = 157,
  [SMALL_STATE(7)] = 193,
  [SMALL_STATE(8)] = 229,
  [SMALL_STATE(9)] = 258,
  [SMALL_STATE(10)] = 287,
  [SMALL_STATE(11)] = 316,
  [SMALL_STATE(12)] = 335,
  [SMALL_STATE(13)] = 349,
  [SMALL_STATE(14)] = 363,
  [SMALL_STATE(15)] = 376,
  [SMALL_STATE(16)] = 387,
  [SMALL_STATE(17)] = 398,
  [SMALL_STATE(18)] = 415,
  [SMALL_STATE(19)] = 427,
  [SMALL_STATE(20)] = 439,
  [SMALL_STATE(21)] = 452,
  [SMALL_STATE(22)] = 461,
  [SMALL_STATE(23)] = 474,
  [SMALL_STATE(24)] = 487,
  [SMALL_STATE(25)] = 496,
  [SMALL_STATE(26)] = 509,
  [SMALL_STATE(27)] = 522,
  [SMALL_STATE(28)] = 528,
  [SMALL_STATE(29)] = 538,
  [SMALL_STATE(30)] = 546,
  [SMALL_STATE(31)] = 552,
  [SMALL_STATE(32)] = 558,
  [SMALL_STATE(33)] = 564,
  [SMALL_STATE(34)] = 569,
  [SMALL_STATE(35)] = 574,
  [SMALL_STATE(36)] = 581,
  [SMALL_STATE(37)] = 586,
  [SMALL_STATE(38)] = 593,
  [SMALL_STATE(39)] = 598,
  [SMALL_STATE(40)] = 603,
  [SMALL_STATE(41)] = 607,
  [SMALL_STATE(42)] = 611,
  [SMALL_STATE(43)] = 615,
  [SMALL_STATE(44)] = 619,
  [SMALL_STATE(45)] = 623,
  [SMALL_STATE(46)] = 627,
  [SMALL_STATE(47)] = 631,
  [SMALL_STATE(48)] = 635,
  [SMALL_STATE(49)] = 639,
  [SMALL_STATE(50)] = 643,
  [SMALL_STATE(51)] = 647,
  [SMALL_STATE(52)] = 651,
  [SMALL_STATE(53)] = 655,
  [SMALL_STATE(54)] = 659,
  [SMALL_STATE(55)] = 663,
  [SMALL_STATE(56)] = 667,
  [SMALL_STATE(57)] = 671,
  [SMALL_STATE(58)] = 675,
  [SMALL_STATE(59)] = 679,
  [SMALL_STATE(60)] = 683,
  [SMALL_STATE(61)] = 687,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [13] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 1, 0, 3),
  [15] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_type, 1, 0, 0),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 2, 0, 3),
  [19] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [37] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [40] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [42] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(59),
  [45] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [48] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [51] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(35),
  [54] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [56] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [58] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [60] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [62] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [64] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [66] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [68] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [70] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [73] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [75] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [77] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [79] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [83] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [85] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [87] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(41),
  [90] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [92] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [94] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [96] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 2, 0, 0),
  [98] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [100] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0),
  [102] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0), SHIFT_REPEAT(43),
  [105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [107] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 1, 0, 0),
  [109] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [111] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [115] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted_string, 1, 0, 0),
  [117] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [119] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 1, 0, 3),
  [121] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [123] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 1, 0, 0),
  [125] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_argument, 3, 0, 8),
  [127] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 3, 0, 0),
  [129] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 3, 0, 1),
  [131] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [133] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [135] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [137] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [139] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [141] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_type, 1, 0, 0),
  [143] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 5),
  [145] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [147] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [149] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [151] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [153] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 6),
  [155] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tellraw_command, 3, 0, 4),
  [157] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command, 2, 0, 0),
  [159] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 5, 0, 7),
  [161] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_effect, 1, 0, 0),
  [163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [165] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_say_command, 2, 0, 2),
  [167] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_unit, 1, 0, 0),
  [169] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [171] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_effect, 1, 0, 0),
  [173] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [175] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [177] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [179] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
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
