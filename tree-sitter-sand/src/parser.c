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
#define STATE_COUNT 66
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 87
#define ALIAS_COUNT 0
#define TOKEN_COUNT 64
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 11
#define MAX_ALIAS_SEQUENCE_LENGTH 5
#define PRODUCTION_ID_COUNT 10

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
  anon_sym_clear = 18,
  anon_sym_eclear = 19,
  anon_sym_tellraw = 20,
  anon_sym_effect = 21,
  anon_sym_minecraft_COLONspeed = 22,
  anon_sym_minecraft_COLONslowness = 23,
  anon_sym_minecraft_COLONhaste = 24,
  anon_sym_minecraft_COLONmining_fatigue = 25,
  anon_sym_minecraft_COLONstrength = 26,
  anon_sym_minecraft_COLONinstant_health = 27,
  anon_sym_minecraft_COLONinstant_damage = 28,
  anon_sym_minecraft_COLONjump_boost = 29,
  anon_sym_minecraft_COLONnausea = 30,
  anon_sym_minecraft_COLONregeneration = 31,
  anon_sym_minecraft_COLONresistance = 32,
  anon_sym_minecraft_COLONfire_resistance = 33,
  anon_sym_minecraft_COLONwater_breathing = 34,
  anon_sym_minecraft_COLONinvisibility = 35,
  anon_sym_minecraft_COLONblindness = 36,
  anon_sym_minecraft_COLONnight_vision = 37,
  anon_sym_minecraft_COLONhunger = 38,
  anon_sym_minecraft_COLONweakness = 39,
  anon_sym_minecraft_COLONpoison = 40,
  anon_sym_minecraft_COLONwither = 41,
  anon_sym_minecraft_COLONhealth_boost = 42,
  anon_sym_minecraft_COLONabsorption = 43,
  anon_sym_minecraft_COLONsaturation = 44,
  anon_sym_minecraft_COLONglowing = 45,
  anon_sym_minecraft_COLONlevitation = 46,
  anon_sym_minecraft_COLONluck = 47,
  anon_sym_minecraft_COLONunluck = 48,
  anon_sym_minecraft_COLONslow_falling = 49,
  anon_sym_minecraft_COLONconduit_power = 50,
  anon_sym_minecraft_COLONdolphins_grace = 51,
  anon_sym_minecraft_COLONbad_omen = 52,
  anon_sym_minecraft_COLONhero_of_the_village = 53,
  sym_text = 54,
  anon_sym_time = 55,
  anon_sym_set = 56,
  anon_sym_query = 57,
  anon_sym_day = 58,
  anon_sym_night = 59,
  anon_sym_noon = 60,
  anon_sym_midnight = 61,
  sym_identifier = 62,
  sym_number = 63,
  sym_source_file = 64,
  sym__definition = 65,
  sym_function_definition = 66,
  sym_block = 67,
  sym__command = 68,
  sym_target_selector = 69,
  sym_selector_type = 70,
  sym_selector_arguments = 71,
  sym_selector_argument = 72,
  sym_range_value = 73,
  sym_quoted_string = 74,
  sym_say_command = 75,
  sym_inv_clear_command = 76,
  sym_effect_clear_command = 77,
  sym_tellraw_command = 78,
  sym_effect_command = 79,
  sym_vanilla_effect = 80,
  sym_custom_effect = 81,
  sym_time_command = 82,
  sym_time_unit = 83,
  aux_sym_source_file_repeat1 = 84,
  aux_sym_block_repeat1 = 85,
  aux_sym_selector_arguments_repeat1 = 86,
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
  [sym_inv_clear_command] = "inv_clear_command",
  [sym_effect_clear_command] = "effect_clear_command",
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
  [sym_inv_clear_command] = sym_inv_clear_command,
  [sym_effect_clear_command] = sym_effect_clear_command,
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
  [4] = {.index = 4, .length = 1},
  [5] = {.index = 5, .length = 2},
  [6] = {.index = 7, .length = 1},
  [7] = {.index = 8, .length = 1},
  [8] = {.index = 9, .length = 4},
  [9] = {.index = 13, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_body, 2},
    {field_name, 1},
  [2] =
    {field_message, 1},
  [3] =
    {field_target, 1},
  [4] =
    {field_selector, 0},
  [5] =
    {field_message, 2},
    {field_target, 1},
  [7] =
    {field_value, 2},
  [8] =
    {field_query_type, 2},
  [9] =
    {field_amplifier, 4},
    {field_duration, 3},
    {field_effect_type, 2},
    {field_target, 1},
  [13] =
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
  [27] = 22,
  [28] = 25,
  [29] = 29,
  [30] = 2,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 3,
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
  [62] = 6,
  [63] = 7,
  [64] = 5,
  [65] = 65,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(319);
      ADVANCE_MAP(
        '"', 2,
        ',', 332,
        '.', 3,
        ';', 322,
        '=', 334,
        '@', 19,
        '[', 330,
        ']', 333,
        'c', 165,
        'd', 20,
        'e', 63,
        'f', 183,
        'm', 134,
        'n', 135,
        'q', 301,
        's', 26,
        't', 82,
        '{', 321,
        '}', 324,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(385);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(2);
      if (lookahead == ',') ADVANCE(332);
      if (lookahead == '.') ADVANCE(3);
      if (lookahead == ']') ADVANCE(333);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(385);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(384);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(336);
      if (lookahead != 0) ADVANCE(2);
      END_STATE();
    case 3:
      if (lookahead == '.') ADVANCE(335);
      END_STATE();
    case 4:
      if (lookahead == ':') ADVANCE(22);
      END_STATE();
    case 5:
      if (lookahead == '_') ADVANCE(68);
      END_STATE();
    case 6:
      if (lookahead == '_') ADVANCE(51);
      END_STATE();
    case 7:
      if (lookahead == '_') ADVANCE(108);
      if (lookahead == 'n') ADVANCE(87);
      END_STATE();
    case 8:
      if (lookahead == '_') ADVANCE(53);
      END_STATE();
    case 9:
      if (lookahead == '_') ADVANCE(234);
      END_STATE();
    case 10:
      if (lookahead == '_') ADVANCE(117);
      END_STATE();
    case 11:
      if (lookahead == '_') ADVANCE(227);
      END_STATE();
    case 12:
      if (lookahead == '_') ADVANCE(214);
      END_STATE();
    case 13:
      if (lookahead == '_') ADVANCE(310);
      END_STATE();
    case 14:
      if (lookahead == '_') ADVANCE(109);
      END_STATE();
    case 15:
      if (lookahead == '_') ADVANCE(294);
      END_STATE();
    case 16:
      if (lookahead == '_') ADVANCE(309);
      END_STATE();
    case 17:
      if (lookahead == '_') ADVANCE(248);
      END_STATE();
    case 18:
      if (lookahead == '_') ADVANCE(54);
      END_STATE();
    case 19:
      if (lookahead == 'a') ADVANCE(326);
      if (lookahead == 'e') ADVANCE(329);
      if (lookahead == 'p') ADVANCE(327);
      if (lookahead == 'r') ADVANCE(328);
      if (lookahead == 's') ADVANCE(325);
      END_STATE();
    case 20:
      if (lookahead == 'a') ADVANCE(315);
      END_STATE();
    case 21:
      if (lookahead == 'a') ADVANCE(311);
      END_STATE();
    case 22:
      ADVANCE_MAP(
        'a', 50,
        'b', 23,
        'c', 217,
        'd', 216,
        'f', 139,
        'g', 169,
        'h', 29,
        'i', 185,
        'j', 302,
        'l', 73,
        'm', 151,
        'n', 31,
        'p', 218,
        'r', 74,
        's', 34,
        'u', 201,
        'w', 36,
      );
      END_STATE();
    case 23:
      if (lookahead == 'a') ADVANCE(64);
      if (lookahead == 'l') ADVANCE(142);
      END_STATE();
    case 24:
      if (lookahead == 'a') ADVANCE(350);
      END_STATE();
    case 25:
      if (lookahead == 'a') ADVANCE(316);
      END_STATE();
    case 26:
      if (lookahead == 'a') ADVANCE(316);
      if (lookahead == 'e') ADVANCE(272);
      END_STATE();
    case 27:
      if (lookahead == 'a') ADVANCE(106);
      END_STATE();
    case 28:
      if (lookahead == 'a') ADVANCE(236);
      END_STATE();
    case 29:
      if (lookahead == 'a') ADVANCE(266);
      if (lookahead == 'e') ADVANCE(35);
      if (lookahead == 'u') ADVANCE(194);
      END_STATE();
    case 30:
      if (lookahead == 'a') ADVANCE(164);
      END_STATE();
    case 31:
      if (lookahead == 'a') ADVANCE(303);
      if (lookahead == 'i') ADVANCE(123);
      END_STATE();
    case 32:
      if (lookahead == 'a') ADVANCE(181);
      END_STATE();
    case 33:
      if (lookahead == 'a') ADVANCE(237);
      END_STATE();
    case 34:
      if (lookahead == 'a') ADVANCE(280);
      if (lookahead == 'l') ADVANCE(213);
      if (lookahead == 'p') ADVANCE(91);
      if (lookahead == 't') ADVANCE(249);
      END_STATE();
    case 35:
      if (lookahead == 'a') ADVANCE(173);
      if (lookahead == 'r') ADVANCE(228);
      END_STATE();
    case 36:
      if (lookahead == 'a') ADVANCE(291);
      if (lookahead == 'e') ADVANCE(30);
      if (lookahead == 'i') ADVANCE(279);
      END_STATE();
    case 37:
      if (lookahead == 'a') ADVANCE(206);
      END_STATE();
    case 38:
      if (lookahead == 'a') ADVANCE(200);
      END_STATE();
    case 39:
      if (lookahead == 'a') ADVANCE(293);
      END_STATE();
    case 40:
      if (lookahead == 'a') ADVANCE(297);
      END_STATE();
    case 41:
      if (lookahead == 'a') ADVANCE(171);
      END_STATE();
    case 42:
      if (lookahead == 'a') ADVANCE(119);
      END_STATE();
    case 43:
      if (lookahead == 'a') ADVANCE(61);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(174);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(209);
      END_STATE();
    case 46:
      if (lookahead == 'a') ADVANCE(120);
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(298);
      END_STATE();
    case 48:
      if (lookahead == 'a') ADVANCE(299);
      END_STATE();
    case 49:
      if (lookahead == 'a') ADVANCE(300);
      END_STATE();
    case 50:
      if (lookahead == 'b') ADVANCE(256);
      END_STATE();
    case 51:
      if (lookahead == 'b') ADVANCE(220);
      END_STATE();
    case 52:
      if (lookahead == 'b') ADVANCE(141);
      END_STATE();
    case 53:
      if (lookahead == 'b') ADVANCE(250);
      END_STATE();
    case 54:
      if (lookahead == 'b') ADVANCE(231);
      END_STATE();
    case 55:
      if (lookahead == 'c') ADVANCE(165);
      if (lookahead == 'e') ADVANCE(63);
      if (lookahead == 's') ADVANCE(25);
      if (lookahead == 't') ADVANCE(82);
      if (lookahead == '}') ADVANCE(324);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(323);
      END_STATE();
    case 56:
      if (lookahead == 'c') ADVANCE(162);
      END_STATE();
    case 57:
      if (lookahead == 'c') ADVANCE(274);
      END_STATE();
    case 58:
      if (lookahead == 'c') ADVANCE(163);
      END_STATE();
    case 59:
      if (lookahead == 'c') ADVANCE(244);
      END_STATE();
    case 60:
      if (lookahead == 'c') ADVANCE(76);
      END_STATE();
    case 61:
      if (lookahead == 'c') ADVANCE(77);
      END_STATE();
    case 62:
      if (lookahead == 'c') ADVANCE(80);
      END_STATE();
    case 63:
      if (lookahead == 'c') ADVANCE(177);
      if (lookahead == 'f') ADVANCE(105);
      END_STATE();
    case 64:
      if (lookahead == 'd') ADVANCE(11);
      END_STATE();
    case 65:
      if (lookahead == 'd') ADVANCE(342);
      END_STATE();
    case 66:
      if (lookahead == 'd') ADVANCE(193);
      if (lookahead == 'n') ADVANCE(84);
      END_STATE();
    case 67:
      if (lookahead == 'd') ADVANCE(307);
      END_STATE();
    case 68:
      if (lookahead == 'd') ADVANCE(32);
      if (lookahead == 'h') ADVANCE(93);
      END_STATE();
    case 69:
      if (lookahead == 'd') ADVANCE(210);
      END_STATE();
    case 70:
      if (lookahead == 'e') ADVANCE(242);
      END_STATE();
    case 71:
      if (lookahead == 'e') ADVANCE(57);
      END_STATE();
    case 72:
      if (lookahead == 'e') ADVANCE(377);
      END_STATE();
    case 73:
      if (lookahead == 'e') ADVANCE(308);
      if (lookahead == 'u') ADVANCE(56);
      END_STATE();
    case 74:
      if (lookahead == 'e') ADVANCE(115);
      END_STATE();
    case 75:
      if (lookahead == 'e') ADVANCE(344);
      END_STATE();
    case 76:
      if (lookahead == 'e') ADVANCE(352);
      END_STATE();
    case 77:
      if (lookahead == 'e') ADVANCE(371);
      END_STATE();
    case 78:
      if (lookahead == 'e') ADVANCE(348);
      END_STATE();
    case 79:
      if (lookahead == 'e') ADVANCE(345);
      END_STATE();
    case 80:
      if (lookahead == 'e') ADVANCE(353);
      END_STATE();
    case 81:
      if (lookahead == 'e') ADVANCE(373);
      END_STATE();
    case 82:
      if (lookahead == 'e') ADVANCE(167);
      if (lookahead == 'i') ADVANCE(180);
      END_STATE();
    case 83:
      if (lookahead == 'e') ADVANCE(28);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(59);
      END_STATE();
    case 85:
      if (lookahead == 'e') ADVANCE(17);
      END_STATE();
    case 86:
      if (lookahead == 'e') ADVANCE(65);
      END_STATE();
    case 87:
      if (lookahead == 'e') ADVANCE(257);
      END_STATE();
    case 88:
      if (lookahead == 'e') ADVANCE(24);
      END_STATE();
    case 89:
      if (lookahead == 'e') ADVANCE(245);
      END_STATE();
    case 90:
      if (lookahead == 'e') ADVANCE(205);
      END_STATE();
    case 91:
      if (lookahead == 'e') ADVANCE(86);
      END_STATE();
    case 92:
      if (lookahead == 'e') ADVANCE(238);
      END_STATE();
    case 93:
      if (lookahead == 'e') ADVANCE(44);
      END_STATE();
    case 94:
      if (lookahead == 'e') ADVANCE(198);
      END_STATE();
    case 95:
      if (lookahead == 'e') ADVANCE(239);
      END_STATE();
    case 96:
      if (lookahead == 'e') ADVANCE(240);
      END_STATE();
    case 97:
      if (lookahead == 'e') ADVANCE(187);
      END_STATE();
    case 98:
      if (lookahead == 'e') ADVANCE(33);
      END_STATE();
    case 99:
      if (lookahead == 'e') ADVANCE(271);
      END_STATE();
    case 100:
      if (lookahead == 'e') ADVANCE(259);
      END_STATE();
    case 101:
      if (lookahead == 'e') ADVANCE(261);
      END_STATE();
    case 102:
      if (lookahead == 'e') ADVANCE(40);
      END_STATE();
    case 103:
      if (lookahead == 'e') ADVANCE(16);
      END_STATE();
    case 104:
      if (lookahead == 'e') ADVANCE(252);
      END_STATE();
    case 105:
      if (lookahead == 'f') ADVANCE(71);
      END_STATE();
    case 106:
      if (lookahead == 'f') ADVANCE(276);
      END_STATE();
    case 107:
      if (lookahead == 'f') ADVANCE(15);
      END_STATE();
    case 108:
      if (lookahead == 'f') ADVANCE(41);
      END_STATE();
    case 109:
      if (lookahead == 'f') ADVANCE(39);
      END_STATE();
    case 110:
      if (lookahead == 'g') ADVANCE(126);
      END_STATE();
    case 111:
      if (lookahead == 'g') ADVANCE(365);
      END_STATE();
    case 112:
      if (lookahead == 'g') ADVANCE(369);
      END_STATE();
    case 113:
      if (lookahead == 'g') ADVANCE(354);
      END_STATE();
    case 114:
      if (lookahead == 'g') ADVANCE(14);
      END_STATE();
    case 115:
      if (lookahead == 'g') ADVANCE(90);
      if (lookahead == 's') ADVANCE(152);
      END_STATE();
    case 116:
      if (lookahead == 'g') ADVANCE(92);
      END_STATE();
    case 117:
      if (lookahead == 'g') ADVANCE(247);
      END_STATE();
    case 118:
      if (lookahead == 'g') ADVANCE(284);
      END_STATE();
    case 119:
      if (lookahead == 'g') ADVANCE(78);
      END_STATE();
    case 120:
      if (lookahead == 'g') ADVANCE(81);
      END_STATE();
    case 121:
      if (lookahead == 'g') ADVANCE(306);
      END_STATE();
    case 122:
      if (lookahead == 'g') ADVANCE(127);
      END_STATE();
    case 123:
      if (lookahead == 'g') ADVANCE(128);
      END_STATE();
    case 124:
      if (lookahead == 'h') ADVANCE(346);
      END_STATE();
    case 125:
      if (lookahead == 'h') ADVANCE(347);
      END_STATE();
    case 126:
      if (lookahead == 'h') ADVANCE(273);
      END_STATE();
    case 127:
      if (lookahead == 'h') ADVANCE(275);
      END_STATE();
    case 128:
      if (lookahead == 'h') ADVANCE(283);
      END_STATE();
    case 129:
      if (lookahead == 'h') ADVANCE(95);
      END_STATE();
    case 130:
      if (lookahead == 'h') ADVANCE(103);
      END_STATE();
    case 131:
      if (lookahead == 'h') ADVANCE(147);
      END_STATE();
    case 132:
      if (lookahead == 'h') ADVANCE(150);
      END_STATE();
    case 133:
      if (lookahead == 'h') ADVANCE(18);
      END_STATE();
    case 134:
      if (lookahead == 'i') ADVANCE(66);
      END_STATE();
    case 135:
      if (lookahead == 'i') ADVANCE(110);
      if (lookahead == 'o') ADVANCE(212);
      END_STATE();
    case 136:
      if (lookahead == 'i') ADVANCE(52);
      END_STATE();
    case 137:
      if (lookahead == 'i') ADVANCE(268);
      END_STATE();
    case 138:
      if (lookahead == 'i') ADVANCE(260);
      END_STATE();
    case 139:
      if (lookahead == 'i') ADVANCE(246);
      END_STATE();
    case 140:
      if (lookahead == 'i') ADVANCE(121);
      END_STATE();
    case 141:
      if (lookahead == 'i') ADVANCE(175);
      END_STATE();
    case 142:
      if (lookahead == 'i') ADVANCE(197);
      END_STATE();
    case 143:
      if (lookahead == 'i') ADVANCE(196);
      END_STATE();
    case 144:
      if (lookahead == 'i') ADVANCE(287);
      END_STATE();
    case 145:
      if (lookahead == 'i') ADVANCE(199);
      END_STATE();
    case 146:
      if (lookahead == 'i') ADVANCE(288);
      END_STATE();
    case 147:
      if (lookahead == 'i') ADVANCE(202);
      END_STATE();
    case 148:
      if (lookahead == 'i') ADVANCE(281);
      END_STATE();
    case 149:
      if (lookahead == 'i') ADVANCE(203);
      END_STATE();
    case 150:
      if (lookahead == 'i') ADVANCE(204);
      END_STATE();
    case 151:
      if (lookahead == 'i') ADVANCE(207);
      END_STATE();
    case 152:
      if (lookahead == 'i') ADVANCE(267);
      END_STATE();
    case 153:
      if (lookahead == 'i') ADVANCE(172);
      END_STATE();
    case 154:
      if (lookahead == 'i') ADVANCE(270);
      END_STATE();
    case 155:
      if (lookahead == 'i') ADVANCE(222);
      END_STATE();
    case 156:
      if (lookahead == 'i') ADVANCE(223);
      END_STATE();
    case 157:
      if (lookahead == 'i') ADVANCE(224);
      END_STATE();
    case 158:
      if (lookahead == 'i') ADVANCE(225);
      END_STATE();
    case 159:
      if (lookahead == 'i') ADVANCE(226);
      END_STATE();
    case 160:
      if (lookahead == 'i') ADVANCE(122);
      END_STATE();
    case 161:
      if (lookahead == 'i') ADVANCE(269);
      END_STATE();
    case 162:
      if (lookahead == 'k') ADVANCE(367);
      END_STATE();
    case 163:
      if (lookahead == 'k') ADVANCE(368);
      END_STATE();
    case 164:
      if (lookahead == 'k') ADVANCE(208);
      END_STATE();
    case 165:
      if (lookahead == 'l') ADVANCE(83);
      END_STATE();
    case 166:
      if (lookahead == 'l') ADVANCE(232);
      END_STATE();
    case 167:
      if (lookahead == 'l') ADVANCE(168);
      END_STATE();
    case 168:
      if (lookahead == 'l') ADVANCE(241);
      END_STATE();
    case 169:
      if (lookahead == 'l') ADVANCE(211);
      END_STATE();
    case 170:
      if (lookahead == 'l') ADVANCE(304);
      END_STATE();
    case 171:
      if (lookahead == 'l') ADVANCE(176);
      END_STATE();
    case 172:
      if (lookahead == 'l') ADVANCE(178);
      END_STATE();
    case 173:
      if (lookahead == 'l') ADVANCE(282);
      END_STATE();
    case 174:
      if (lookahead == 'l') ADVANCE(286);
      END_STATE();
    case 175:
      if (lookahead == 'l') ADVANCE(148);
      END_STATE();
    case 176:
      if (lookahead == 'l') ADVANCE(149);
      END_STATE();
    case 177:
      if (lookahead == 'l') ADVANCE(98);
      END_STATE();
    case 178:
      if (lookahead == 'l') ADVANCE(46);
      END_STATE();
    case 179:
      if (lookahead == 'm') ADVANCE(233);
      END_STATE();
    case 180:
      if (lookahead == 'm') ADVANCE(72);
      END_STATE();
    case 181:
      if (lookahead == 'm') ADVANCE(42);
      END_STATE();
    case 182:
      if (lookahead == 'm') ADVANCE(97);
      END_STATE();
    case 183:
      if (lookahead == 'n') ADVANCE(320);
      END_STATE();
    case 184:
      if (lookahead == 'n') ADVANCE(382);
      END_STATE();
    case 185:
      if (lookahead == 'n') ADVANCE(258);
      END_STATE();
    case 186:
      if (lookahead == 'n') ADVANCE(360);
      END_STATE();
    case 187:
      if (lookahead == 'n') ADVANCE(372);
      END_STATE();
    case 188:
      if (lookahead == 'n') ADVANCE(363);
      END_STATE();
    case 189:
      if (lookahead == 'n') ADVANCE(366);
      END_STATE();
    case 190:
      if (lookahead == 'n') ADVANCE(364);
      END_STATE();
    case 191:
      if (lookahead == 'n') ADVANCE(357);
      END_STATE();
    case 192:
      if (lookahead == 'n') ADVANCE(351);
      END_STATE();
    case 193:
      if (lookahead == 'n') ADVANCE(160);
      END_STATE();
    case 194:
      if (lookahead == 'n') ADVANCE(116);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(67);
      END_STATE();
    case 196:
      if (lookahead == 'n') ADVANCE(114);
      END_STATE();
    case 197:
      if (lookahead == 'n') ADVANCE(69);
      END_STATE();
    case 198:
      if (lookahead == 'n') ADVANCE(118);
      END_STATE();
    case 199:
      if (lookahead == 'n') ADVANCE(111);
      END_STATE();
    case 200:
      if (lookahead == 'n') ADVANCE(60);
      END_STATE();
    case 201:
      if (lookahead == 'n') ADVANCE(170);
      END_STATE();
    case 202:
      if (lookahead == 'n') ADVANCE(262);
      END_STATE();
    case 203:
      if (lookahead == 'n') ADVANCE(112);
      END_STATE();
    case 204:
      if (lookahead == 'n') ADVANCE(113);
      END_STATE();
    case 205:
      if (lookahead == 'n') ADVANCE(104);
      END_STATE();
    case 206:
      if (lookahead == 'n') ADVANCE(289);
      END_STATE();
    case 207:
      if (lookahead == 'n') ADVANCE(143);
      END_STATE();
    case 208:
      if (lookahead == 'n') ADVANCE(100);
      END_STATE();
    case 209:
      if (lookahead == 'n') ADVANCE(62);
      END_STATE();
    case 210:
      if (lookahead == 'n') ADVANCE(101);
      END_STATE();
    case 211:
      if (lookahead == 'o') ADVANCE(314);
      END_STATE();
    case 212:
      if (lookahead == 'o') ADVANCE(184);
      END_STATE();
    case 213:
      if (lookahead == 'o') ADVANCE(312);
      END_STATE();
    case 214:
      if (lookahead == 'o') ADVANCE(107);
      END_STATE();
    case 215:
      if (lookahead == 'o') ADVANCE(313);
      END_STATE();
    case 216:
      if (lookahead == 'o') ADVANCE(166);
      END_STATE();
    case 217:
      if (lookahead == 'o') ADVANCE(195);
      END_STATE();
    case 218:
      if (lookahead == 'o') ADVANCE(137);
      END_STATE();
    case 219:
      if (lookahead == 'o') ADVANCE(243);
      END_STATE();
    case 220:
      if (lookahead == 'o') ADVANCE(229);
      END_STATE();
    case 221:
      if (lookahead == 'o') ADVANCE(186);
      END_STATE();
    case 222:
      if (lookahead == 'o') ADVANCE(188);
      END_STATE();
    case 223:
      if (lookahead == 'o') ADVANCE(189);
      END_STATE();
    case 224:
      if (lookahead == 'o') ADVANCE(190);
      END_STATE();
    case 225:
      if (lookahead == 'o') ADVANCE(191);
      END_STATE();
    case 226:
      if (lookahead == 'o') ADVANCE(192);
      END_STATE();
    case 227:
      if (lookahead == 'o') ADVANCE(182);
      END_STATE();
    case 228:
      if (lookahead == 'o') ADVANCE(12);
      END_STATE();
    case 229:
      if (lookahead == 'o') ADVANCE(263);
      END_STATE();
    case 230:
      if (lookahead == 'o') ADVANCE(264);
      END_STATE();
    case 231:
      if (lookahead == 'o') ADVANCE(230);
      END_STATE();
    case 232:
      if (lookahead == 'p') ADVANCE(131);
      END_STATE();
    case 233:
      if (lookahead == 'p') ADVANCE(6);
      END_STATE();
    case 234:
      if (lookahead == 'p') ADVANCE(215);
      END_STATE();
    case 235:
      if (lookahead == 'p') ADVANCE(290);
      END_STATE();
    case 236:
      if (lookahead == 'r') ADVANCE(338);
      END_STATE();
    case 237:
      if (lookahead == 'r') ADVANCE(339);
      END_STATE();
    case 238:
      if (lookahead == 'r') ADVANCE(358);
      END_STATE();
    case 239:
      if (lookahead == 'r') ADVANCE(361);
      END_STATE();
    case 240:
      if (lookahead == 'r') ADVANCE(370);
      END_STATE();
    case 241:
      if (lookahead == 'r') ADVANCE(21);
      END_STATE();
    case 242:
      if (lookahead == 'r') ADVANCE(317);
      END_STATE();
    case 243:
      if (lookahead == 'r') ADVANCE(235);
      END_STATE();
    case 244:
      if (lookahead == 'r') ADVANCE(27);
      END_STATE();
    case 245:
      if (lookahead == 'r') ADVANCE(8);
      END_STATE();
    case 246:
      if (lookahead == 'r') ADVANCE(85);
      END_STATE();
    case 247:
      if (lookahead == 'r') ADVANCE(43);
      END_STATE();
    case 248:
      if (lookahead == 'r') ADVANCE(99);
      END_STATE();
    case 249:
      if (lookahead == 'r') ADVANCE(94);
      END_STATE();
    case 250:
      if (lookahead == 'r') ADVANCE(102);
      END_STATE();
    case 251:
      if (lookahead == 'r') ADVANCE(48);
      END_STATE();
    case 252:
      if (lookahead == 'r') ADVANCE(49);
      END_STATE();
    case 253:
      if (lookahead == 's') ADVANCE(343);
      END_STATE();
    case 254:
      if (lookahead == 's') ADVANCE(359);
      END_STATE();
    case 255:
      if (lookahead == 's') ADVANCE(356);
      END_STATE();
    case 256:
      if (lookahead == 's') ADVANCE(219);
      END_STATE();
    case 257:
      if (lookahead == 's') ADVANCE(253);
      END_STATE();
    case 258:
      if (lookahead == 's') ADVANCE(285);
      if (lookahead == 'v') ADVANCE(138);
      END_STATE();
    case 259:
      if (lookahead == 's') ADVANCE(254);
      END_STATE();
    case 260:
      if (lookahead == 's') ADVANCE(136);
      END_STATE();
    case 261:
      if (lookahead == 's') ADVANCE(255);
      END_STATE();
    case 262:
      if (lookahead == 's') ADVANCE(10);
      END_STATE();
    case 263:
      if (lookahead == 's') ADVANCE(277);
      END_STATE();
    case 264:
      if (lookahead == 's') ADVANCE(278);
      END_STATE();
    case 265:
      if (lookahead == 's') ADVANCE(88);
      END_STATE();
    case 266:
      if (lookahead == 's') ADVANCE(292);
      END_STATE();
    case 267:
      if (lookahead == 's') ADVANCE(295);
      END_STATE();
    case 268:
      if (lookahead == 's') ADVANCE(221);
      END_STATE();
    case 269:
      if (lookahead == 's') ADVANCE(296);
      END_STATE();
    case 270:
      if (lookahead == 's') ADVANCE(158);
      END_STATE();
    case 271:
      if (lookahead == 's') ADVANCE(161);
      END_STATE();
    case 272:
      if (lookahead == 't') ADVANCE(378);
      END_STATE();
    case 273:
      if (lookahead == 't') ADVANCE(381);
      END_STATE();
    case 274:
      if (lookahead == 't') ADVANCE(341);
      END_STATE();
    case 275:
      if (lookahead == 't') ADVANCE(383);
      END_STATE();
    case 276:
      if (lookahead == 't') ADVANCE(4);
      END_STATE();
    case 277:
      if (lookahead == 't') ADVANCE(349);
      END_STATE();
    case 278:
      if (lookahead == 't') ADVANCE(362);
      END_STATE();
    case 279:
      if (lookahead == 't') ADVANCE(129);
      END_STATE();
    case 280:
      if (lookahead == 't') ADVANCE(305);
      END_STATE();
    case 281:
      if (lookahead == 't') ADVANCE(318);
      END_STATE();
    case 282:
      if (lookahead == 't') ADVANCE(133);
      END_STATE();
    case 283:
      if (lookahead == 't') ADVANCE(13);
      END_STATE();
    case 284:
      if (lookahead == 't') ADVANCE(124);
      END_STATE();
    case 285:
      if (lookahead == 't') ADVANCE(37);
      END_STATE();
    case 286:
      if (lookahead == 't') ADVANCE(125);
      END_STATE();
    case 287:
      if (lookahead == 't') ADVANCE(47);
      END_STATE();
    case 288:
      if (lookahead == 't') ADVANCE(9);
      END_STATE();
    case 289:
      if (lookahead == 't') ADVANCE(5);
      END_STATE();
    case 290:
      if (lookahead == 't') ADVANCE(155);
      END_STATE();
    case 291:
      if (lookahead == 't') ADVANCE(89);
      END_STATE();
    case 292:
      if (lookahead == 't') ADVANCE(75);
      END_STATE();
    case 293:
      if (lookahead == 't') ADVANCE(140);
      END_STATE();
    case 294:
      if (lookahead == 't') ADVANCE(130);
      END_STATE();
    case 295:
      if (lookahead == 't') ADVANCE(38);
      END_STATE();
    case 296:
      if (lookahead == 't') ADVANCE(45);
      END_STATE();
    case 297:
      if (lookahead == 't') ADVANCE(132);
      END_STATE();
    case 298:
      if (lookahead == 't') ADVANCE(156);
      END_STATE();
    case 299:
      if (lookahead == 't') ADVANCE(157);
      END_STATE();
    case 300:
      if (lookahead == 't') ADVANCE(159);
      END_STATE();
    case 301:
      if (lookahead == 'u') ADVANCE(70);
      END_STATE();
    case 302:
      if (lookahead == 'u') ADVANCE(179);
      END_STATE();
    case 303:
      if (lookahead == 'u') ADVANCE(265);
      END_STATE();
    case 304:
      if (lookahead == 'u') ADVANCE(58);
      END_STATE();
    case 305:
      if (lookahead == 'u') ADVANCE(251);
      END_STATE();
    case 306:
      if (lookahead == 'u') ADVANCE(79);
      END_STATE();
    case 307:
      if (lookahead == 'u') ADVANCE(146);
      END_STATE();
    case 308:
      if (lookahead == 'v') ADVANCE(144);
      END_STATE();
    case 309:
      if (lookahead == 'v') ADVANCE(153);
      END_STATE();
    case 310:
      if (lookahead == 'v') ADVANCE(154);
      END_STATE();
    case 311:
      if (lookahead == 'w') ADVANCE(340);
      END_STATE();
    case 312:
      if (lookahead == 'w') ADVANCE(7);
      END_STATE();
    case 313:
      if (lookahead == 'w') ADVANCE(96);
      END_STATE();
    case 314:
      if (lookahead == 'w') ADVANCE(145);
      END_STATE();
    case 315:
      if (lookahead == 'y') ADVANCE(380);
      END_STATE();
    case 316:
      if (lookahead == 'y') ADVANCE(337);
      END_STATE();
    case 317:
      if (lookahead == 'y') ADVANCE(379);
      END_STATE();
    case 318:
      if (lookahead == 'y') ADVANCE(355);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(anon_sym_fn);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(aux_sym_block_token1);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(323);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(anon_sym_ATs);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(anon_sym_ATa);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(anon_sym_ATp);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(anon_sym_ATr);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(anon_sym_ATe);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(376);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(aux_sym_quoted_string_token1);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(anon_sym_say);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(anon_sym_clear);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(anon_sym_eclear);
      END_STATE();
    case 340:
      ACCEPT_TOKEN(anon_sym_tellraw);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(anon_sym_effect);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONspeed);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslowness);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhaste);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONmining_fatigue);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONstrength);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_health);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinstant_damage);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONjump_boost);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnausea);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONregeneration);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONresistance);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONfire_resistance);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwater_breathing);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONinvisibility);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONblindness);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONnight_vision);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhunger);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONweakness);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONpoison);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONwither);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhealth_boost);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONabsorption);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONsaturation);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONglowing);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONlevitation);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONluck);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONunluck);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONslow_falling);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONconduit_power);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONdolphins_grace);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONbad_omen);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(anon_sym_minecraft_COLONhero_of_the_village);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(sym_text);
      if (lookahead == '[') ADVANCE(331);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(374);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(376);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(sym_text);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(375);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(376);
      END_STATE();
    case 376:
      ACCEPT_TOKEN(sym_text);
      if (lookahead != 0 &&
          lookahead != ';') ADVANCE(376);
      END_STATE();
    case 377:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 378:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 379:
      ACCEPT_TOKEN(anon_sym_query);
      END_STATE();
    case 380:
      ACCEPT_TOKEN(anon_sym_day);
      END_STATE();
    case 381:
      ACCEPT_TOKEN(anon_sym_night);
      END_STATE();
    case 382:
      ACCEPT_TOKEN(anon_sym_noon);
      END_STATE();
    case 383:
      ACCEPT_TOKEN(anon_sym_midnight);
      END_STATE();
    case 384:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(384);
      END_STATE();
    case 385:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(385);
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
  [8] = {.lex_state = 55},
  [9] = {.lex_state = 55},
  [10] = {.lex_state = 55},
  [11] = {.lex_state = 0},
  [12] = {.lex_state = 55},
  [13] = {.lex_state = 55},
  [14] = {.lex_state = 0},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 1},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 0},
  [22] = {.lex_state = 1},
  [23] = {.lex_state = 1},
  [24] = {.lex_state = 1},
  [25] = {.lex_state = 1},
  [26] = {.lex_state = 1},
  [27] = {.lex_state = 1},
  [28] = {.lex_state = 1},
  [29] = {.lex_state = 1},
  [30] = {.lex_state = 374},
  [31] = {.lex_state = 1},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 1},
  [35] = {.lex_state = 0},
  [36] = {.lex_state = 0},
  [37] = {.lex_state = 1},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 374},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 0},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 0},
  [49] = {.lex_state = 375},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 0},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 0},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 1},
  [58] = {.lex_state = 0},
  [59] = {.lex_state = 375},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 375},
  [63] = {.lex_state = 375},
  [64] = {.lex_state = 375},
  [65] = {.lex_state = 1},
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
    [anon_sym_day] = ACTIONS(1),
    [anon_sym_night] = ACTIONS(1),
    [anon_sym_noon] = ACTIONS(1),
    [anon_sym_midnight] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(53),
    [sym__definition] = STATE(21),
    [sym_function_definition] = STATE(21),
    [aux_sym_source_file_repeat1] = STATE(21),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_fn] = ACTIONS(5),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 3,
    ACTIONS(9), 1,
      anon_sym_LBRACK,
    STATE(6), 1,
      sym_selector_arguments,
    ACTIONS(7), 34,
      anon_sym_SEMI,
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
  [43] = 1,
    ACTIONS(11), 35,
      anon_sym_SEMI,
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
  [81] = 3,
    ACTIONS(13), 1,
      aux_sym_quoted_string_token1,
    STATE(43), 2,
      sym_vanilla_effect,
      sym_custom_effect,
    ACTIONS(15), 32,
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
  [123] = 1,
    ACTIONS(17), 34,
      anon_sym_SEMI,
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
  [160] = 1,
    ACTIONS(19), 34,
      anon_sym_SEMI,
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
  [197] = 1,
    ACTIONS(21), 34,
      anon_sym_SEMI,
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
  [234] = 10,
    ACTIONS(23), 1,
      aux_sym_block_token1,
    ACTIONS(25), 1,
      anon_sym_RBRACE,
    ACTIONS(27), 1,
      anon_sym_say,
    ACTIONS(29), 1,
      anon_sym_clear,
    ACTIONS(31), 1,
      anon_sym_eclear,
    ACTIONS(33), 1,
      anon_sym_tellraw,
    ACTIONS(35), 1,
      anon_sym_effect,
    ACTIONS(37), 1,
      anon_sym_time,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(51), 7,
      sym__command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [271] = 10,
    ACTIONS(23), 1,
      aux_sym_block_token1,
    ACTIONS(27), 1,
      anon_sym_say,
    ACTIONS(29), 1,
      anon_sym_clear,
    ACTIONS(31), 1,
      anon_sym_eclear,
    ACTIONS(33), 1,
      anon_sym_tellraw,
    ACTIONS(35), 1,
      anon_sym_effect,
    ACTIONS(37), 1,
      anon_sym_time,
    ACTIONS(39), 1,
      anon_sym_RBRACE,
    STATE(8), 1,
      aux_sym_block_repeat1,
    STATE(51), 7,
      sym__command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [308] = 10,
    ACTIONS(41), 1,
      aux_sym_block_token1,
    ACTIONS(44), 1,
      anon_sym_RBRACE,
    ACTIONS(46), 1,
      anon_sym_say,
    ACTIONS(49), 1,
      anon_sym_clear,
    ACTIONS(52), 1,
      anon_sym_eclear,
    ACTIONS(55), 1,
      anon_sym_tellraw,
    ACTIONS(58), 1,
      anon_sym_effect,
    ACTIONS(61), 1,
      anon_sym_time,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(51), 7,
      sym__command,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [345] = 7,
    ACTIONS(64), 1,
      anon_sym_say,
    ACTIONS(66), 1,
      anon_sym_clear,
    ACTIONS(68), 1,
      anon_sym_eclear,
    ACTIONS(70), 1,
      anon_sym_tellraw,
    ACTIONS(72), 1,
      anon_sym_effect,
    ACTIONS(74), 1,
      anon_sym_time,
    STATE(45), 6,
      sym_say_command,
      sym_inv_clear_command,
      sym_effect_clear_command,
      sym_tellraw_command,
      sym_effect_command,
      sym_time_command,
  [372] = 2,
    ACTIONS(76), 1,
      aux_sym_block_token1,
    ACTIONS(44), 7,
      anon_sym_RBRACE,
      anon_sym_say,
      anon_sym_clear,
      anon_sym_eclear,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_time,
  [385] = 2,
    ACTIONS(79), 1,
      aux_sym_block_token1,
    ACTIONS(81), 7,
      anon_sym_RBRACE,
      anon_sym_say,
      anon_sym_clear,
      anon_sym_eclear,
      anon_sym_tellraw,
      anon_sym_effect,
      anon_sym_time,
  [398] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(54), 1,
      sym_target_selector,
    ACTIONS(83), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [412] = 3,
    STATE(30), 1,
      sym_selector_type,
    STATE(59), 1,
      sym_target_selector,
    ACTIONS(85), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [426] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(52), 1,
      sym_target_selector,
    ACTIONS(83), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [440] = 3,
    STATE(2), 1,
      sym_selector_type,
    STATE(4), 1,
      sym_target_selector,
    ACTIONS(83), 5,
      anon_sym_ATs,
      anon_sym_ATa,
      anon_sym_ATp,
      anon_sym_ATr,
      anon_sym_ATe,
  [454] = 3,
    ACTIONS(89), 1,
      sym_number,
    STATE(48), 1,
      sym_time_unit,
    ACTIONS(87), 4,
      anon_sym_day,
      anon_sym_night,
      anon_sym_noon,
      anon_sym_midnight,
  [467] = 5,
    ACTIONS(91), 1,
      anon_sym_DOT_DOT,
    ACTIONS(93), 1,
      aux_sym_quoted_string_token1,
    ACTIONS(95), 1,
      sym_identifier,
    ACTIONS(97), 1,
      sym_number,
    STATE(33), 2,
      sym_range_value,
      sym_quoted_string,
  [484] = 3,
    ACTIONS(99), 1,
      ts_builtin_sym_end,
    ACTIONS(101), 1,
      anon_sym_fn,
    STATE(20), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [496] = 3,
    ACTIONS(5), 1,
      anon_sym_fn,
    ACTIONS(104), 1,
      ts_builtin_sym_end,
    STATE(20), 3,
      sym__definition,
      sym_function_definition,
      aux_sym_source_file_repeat1,
  [508] = 4,
    ACTIONS(106), 1,
      anon_sym_RBRACK,
    ACTIONS(108), 1,
      sym_identifier,
    STATE(25), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(31), 1,
      sym_selector_argument,
  [521] = 4,
    ACTIONS(110), 1,
      anon_sym_RBRACK,
    ACTIONS(112), 1,
      sym_identifier,
    STATE(23), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(31), 1,
      sym_selector_argument,
  [534] = 2,
    ACTIONS(117), 1,
      anon_sym_DOT_DOT,
    ACTIONS(115), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [543] = 4,
    ACTIONS(108), 1,
      sym_identifier,
    ACTIONS(119), 1,
      anon_sym_RBRACK,
    STATE(23), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(31), 1,
      sym_selector_argument,
  [556] = 2,
    ACTIONS(123), 1,
      sym_number,
    ACTIONS(121), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [565] = 4,
    ACTIONS(108), 1,
      sym_identifier,
    ACTIONS(125), 1,
      anon_sym_RBRACK,
    STATE(28), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(31), 1,
      sym_selector_argument,
  [578] = 4,
    ACTIONS(108), 1,
      sym_identifier,
    ACTIONS(127), 1,
      anon_sym_RBRACK,
    STATE(23), 1,
      aux_sym_selector_arguments_repeat1,
    STATE(31), 1,
      sym_selector_argument,
  [591] = 1,
    ACTIONS(121), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [597] = 3,
    ACTIONS(129), 1,
      anon_sym_LBRACK,
    ACTIONS(131), 1,
      sym_text,
    STATE(62), 1,
      sym_selector_arguments,
  [607] = 2,
    ACTIONS(133), 1,
      anon_sym_COMMA,
    ACTIONS(135), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [615] = 1,
    ACTIONS(137), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [621] = 1,
    ACTIONS(139), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [627] = 1,
    ACTIONS(141), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      sym_identifier,
  [633] = 1,
    ACTIONS(143), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [638] = 2,
    ACTIONS(145), 1,
      anon_sym_set,
    ACTIONS(147), 1,
      anon_sym_query,
  [645] = 1,
    ACTIONS(110), 2,
      anon_sym_RBRACK,
      sym_identifier,
  [650] = 1,
    ACTIONS(149), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [655] = 1,
    ACTIONS(151), 2,
      ts_builtin_sym_end,
      anon_sym_fn,
  [660] = 1,
    ACTIONS(153), 2,
      anon_sym_LBRACK,
      sym_text,
  [665] = 2,
    ACTIONS(155), 1,
      anon_sym_LBRACE,
    STATE(38), 1,
      sym_block,
  [672] = 1,
    ACTIONS(157), 1,
      anon_sym_SEMI,
  [676] = 1,
    ACTIONS(159), 1,
      sym_number,
  [680] = 1,
    ACTIONS(161), 1,
      anon_sym_SEMI,
  [684] = 1,
    ACTIONS(163), 1,
      anon_sym_SEMI,
  [688] = 1,
    ACTIONS(165), 1,
      anon_sym_SEMI,
  [692] = 1,
    ACTIONS(167), 1,
      anon_sym_EQ,
  [696] = 1,
    ACTIONS(169), 1,
      anon_sym_SEMI,
  [700] = 1,
    ACTIONS(171), 1,
      sym_text,
  [704] = 1,
    ACTIONS(173), 1,
      sym_number,
  [708] = 1,
    ACTIONS(175), 1,
      anon_sym_SEMI,
  [712] = 1,
    ACTIONS(177), 1,
      anon_sym_SEMI,
  [716] = 1,
    ACTIONS(179), 1,
      ts_builtin_sym_end,
  [720] = 1,
    ACTIONS(181), 1,
      anon_sym_SEMI,
  [724] = 1,
    ACTIONS(183), 1,
      anon_sym_SEMI,
  [728] = 1,
    ACTIONS(185), 1,
      anon_sym_SEMI,
  [732] = 1,
    ACTIONS(187), 1,
      sym_identifier,
  [736] = 1,
    ACTIONS(189), 1,
      sym_number,
  [740] = 1,
    ACTIONS(191), 1,
      sym_text,
  [744] = 1,
    ACTIONS(193), 1,
      sym_number,
  [748] = 1,
    ACTIONS(195), 1,
      sym_number,
  [752] = 1,
    ACTIONS(19), 1,
      sym_text,
  [756] = 1,
    ACTIONS(21), 1,
      sym_text,
  [760] = 1,
    ACTIONS(17), 1,
      sym_text,
  [764] = 1,
    ACTIONS(197), 1,
      sym_identifier,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 43,
  [SMALL_STATE(4)] = 81,
  [SMALL_STATE(5)] = 123,
  [SMALL_STATE(6)] = 160,
  [SMALL_STATE(7)] = 197,
  [SMALL_STATE(8)] = 234,
  [SMALL_STATE(9)] = 271,
  [SMALL_STATE(10)] = 308,
  [SMALL_STATE(11)] = 345,
  [SMALL_STATE(12)] = 372,
  [SMALL_STATE(13)] = 385,
  [SMALL_STATE(14)] = 398,
  [SMALL_STATE(15)] = 412,
  [SMALL_STATE(16)] = 426,
  [SMALL_STATE(17)] = 440,
  [SMALL_STATE(18)] = 454,
  [SMALL_STATE(19)] = 467,
  [SMALL_STATE(20)] = 484,
  [SMALL_STATE(21)] = 496,
  [SMALL_STATE(22)] = 508,
  [SMALL_STATE(23)] = 521,
  [SMALL_STATE(24)] = 534,
  [SMALL_STATE(25)] = 543,
  [SMALL_STATE(26)] = 556,
  [SMALL_STATE(27)] = 565,
  [SMALL_STATE(28)] = 578,
  [SMALL_STATE(29)] = 591,
  [SMALL_STATE(30)] = 597,
  [SMALL_STATE(31)] = 607,
  [SMALL_STATE(32)] = 615,
  [SMALL_STATE(33)] = 621,
  [SMALL_STATE(34)] = 627,
  [SMALL_STATE(35)] = 633,
  [SMALL_STATE(36)] = 638,
  [SMALL_STATE(37)] = 645,
  [SMALL_STATE(38)] = 650,
  [SMALL_STATE(39)] = 655,
  [SMALL_STATE(40)] = 660,
  [SMALL_STATE(41)] = 665,
  [SMALL_STATE(42)] = 672,
  [SMALL_STATE(43)] = 676,
  [SMALL_STATE(44)] = 680,
  [SMALL_STATE(45)] = 684,
  [SMALL_STATE(46)] = 688,
  [SMALL_STATE(47)] = 692,
  [SMALL_STATE(48)] = 696,
  [SMALL_STATE(49)] = 700,
  [SMALL_STATE(50)] = 704,
  [SMALL_STATE(51)] = 708,
  [SMALL_STATE(52)] = 712,
  [SMALL_STATE(53)] = 716,
  [SMALL_STATE(54)] = 720,
  [SMALL_STATE(55)] = 724,
  [SMALL_STATE(56)] = 728,
  [SMALL_STATE(57)] = 732,
  [SMALL_STATE(58)] = 736,
  [SMALL_STATE(59)] = 740,
  [SMALL_STATE(60)] = 744,
  [SMALL_STATE(61)] = 748,
  [SMALL_STATE(62)] = 752,
  [SMALL_STATE(63)] = 756,
  [SMALL_STATE(64)] = 760,
  [SMALL_STATE(65)] = 764,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [7] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 1, 0, 4),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [11] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_type, 1, 0, 0),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 3, 0, 0),
  [19] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_target_selector, 2, 0, 4),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_arguments, 2, 0, 0),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(49),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [41] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(11),
  [44] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [46] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(49),
  [49] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [52] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(14),
  [55] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(15),
  [58] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [61] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(36),
  [64] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [66] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [68] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [70] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [72] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [74] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [76] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [79] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [81] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 3, 0, 0),
  [83] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [85] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [87] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [89] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [91] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [93] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [95] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [97] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [99] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [101] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(57),
  [104] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [106] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [108] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [110] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0),
  [112] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 2, 0, 0), SHIFT_REPEAT(47),
  [115] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 1, 0, 0),
  [117] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [121] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 2, 0, 0),
  [123] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [125] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [127] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [129] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [131] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_target_selector, 1, 0, 4),
  [133] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [135] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_selector_arguments_repeat1, 1, 0, 0),
  [137] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted_string, 1, 0, 0),
  [139] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_selector_argument, 3, 0, 9),
  [141] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_range_value, 3, 0, 0),
  [143] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [145] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [147] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [149] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_function_definition, 3, 0, 1),
  [151] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [153] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_selector_type, 1, 0, 0),
  [155] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [157] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_unit, 1, 0, 0),
  [159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [161] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 7),
  [163] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command, 2, 0, 0),
  [165] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_say_command, 2, 0, 2),
  [167] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [169] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_time_command, 3, 0, 6),
  [171] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [173] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [175] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [177] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inv_clear_command, 2, 0, 3),
  [179] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [181] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_clear_command, 2, 0, 3),
  [183] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_command, 5, 0, 8),
  [185] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tellraw_command, 3, 0, 5),
  [187] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [193] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_custom_effect, 1, 0, 0),
  [195] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_vanilla_effect, 1, 0, 0),
  [197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
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
