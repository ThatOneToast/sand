module.exports = grammar({
    name: 'sand',

    rules: {
        source_file: $ => repeat($._definition),

        _definition: $ => $.function_definition,

        function_definition: $ => seq(
            'fn',
            field('name', $.identifier),
            field('body', $.block)
        ),

        block: $ => seq(
            '{',
            repeat(seq(
                $._command,
                ';',
                optional(/\s+/)
            )),
            '}'
        ),

        _command: $ => prec.left(seq(
            optional(/\s+/),
            choice(
                $.time_command,

                $.say_command,
                $.tellraw_command,

                $.effect_command,

                $.inv_clear_command,
                $.effect_clear_command,

                $.gm_creative_command,
                $.gm_spectator_command,
                $.gm_survival_command,
                $.gm_adventure_command,

                $.xp_add_command,
                $.xp_set_command,
                $.xp_query_command,


            )
        )),

        target_selector: $ => seq(
            field('selector', $.selector_type),
            optional($.selector_arguments)
        ),

        selector_type: $ => choice(
            '@s',
            '@a',
            '@p',
            '@r',
            '@e'
        ),

        selector_arguments: $ => seq(
            '[',
            repeat(seq(
                $.selector_argument,
                optional(',')
            )),
            ']'
        ),

        selector_argument: $ => prec(1, seq(
            field('key', $.identifier),
            '=',
            field('value', choice(
                $.range_value,
                $.number,
                $.identifier,
                $.quoted_string
            ))
        )),
        
        xp_type: $ => choice(
            'levels',
            'points'
        ),

        range_value: $ => prec.left(2, choice(
            seq($.number, '..', $.number),  // 1..10
            seq($.number, '..'),            // 1..
            seq('..', $.number),            // ..10
            $.number                        // exact value
        )),

        xp_add_command: $ => seq(
            'xpadd',
            field('amount', $.number),
            field('target', $.target_selector),
            field('etype', $.xp_type)
        ),

        xp_set_command: $ => seq(
            'xpset',
            field('amount', $.number),
            field('target', $.target_selector),
            field('etype', $.xp_type)
        ),

        xp_query_command: $ => seq(
            'xpquery',
            field('target', $.target_selector),
            field('etype', $.xp_type)
        ),

        quoted_string: $ => /"[^"]*"/,

        say_command: $ => seq(
            'say',
            field('message', $.text)
        ),

        inv_clear_command: $ => seq(
            'clear',
            field('target', $.target_selector)
        ),

        effect_clear_command: $ => seq(
            'eclear',
            field('target', $.target_selector),
        ),

        tellraw_command: $ => seq(
            'tellraw',
            field('target', $.target_selector),
            field('message', $.text)
        ),

        effect_command: $ => seq(
            'effect',
            field('target', $.target_selector),
            field('effect_type', choice(
                $.vanilla_effect,
                $.custom_effect
            )),
            field('duration', $.number),
            field('amplifier', $.number)
        ),

        vanilla_effect: $ => choice(
            'minecraft:speed',
            'minecraft:slowness',
            'minecraft:haste',
            'minecraft:mining_fatigue',
            'minecraft:strength',
            'minecraft:instant_health',
            'minecraft:instant_damage',
            'minecraft:jump_boost',
            'minecraft:nausea',
            'minecraft:regeneration',
            'minecraft:resistance',
            'minecraft:fire_resistance',
            'minecraft:water_breathing',
            'minecraft:invisibility',
            'minecraft:blindness',
            'minecraft:night_vision',
            'minecraft:hunger',
            'minecraft:weakness',
            'minecraft:poison',
            'minecraft:wither',
            'minecraft:health_boost',
            'minecraft:absorption',
            'minecraft:saturation',
            'minecraft:glowing',
            'minecraft:levitation',
            'minecraft:luck',
            'minecraft:unluck',
            'minecraft:slow_falling',
            'minecraft:conduit_power',
            'minecraft:dolphins_grace',
            'minecraft:bad_omen',
            'minecraft:hero_of_the_village'
        ),

        custom_effect: $ => /"[^"]*"/,

        text: $ => /[^;]*/,

        time_command: $ => seq(
            'time',
            choice(
                seq('set', field('value', choice($.number, $.time_unit))),
                seq('query', field('query_type', $.identifier))
            )
        ),

        gm_survival_command: $ => seq(
            'gms',
            field('target', $.target_selector)
        ),

        gm_adventure_command: $ => seq(
            'gma',
            field('target', $.target_selector)
        ),

        gm_spectator_command: $ => seq(
            'gmsp',
            field('target', $.target_selector)
        ),

        gm_creative_command: $ => seq(
            'gmc',
            field('target', $.target_selector)
        ),

        time_unit: $ => choice('day', 'night', 'noon', 'midnight'),

        identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
        number: $ => /\d+/,
    }
});
