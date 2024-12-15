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

                $.enchant_command,

                $.execute_command,
            )
        )),

        execute_command: $ => prec.left(seq(
            'execute',
            repeat1($.execute_subcommand)
        )),

        execute_subcommand: $ => choice(
            $.execute_as,
            $.execute_at,
            $.execute_align,
            $.execute_anchored,
            $.execute_facing,
            $.execute_facing_entity,
            $.execute_in,
            $.execute_positioned,
            $.execute_positioned_as,
            $.execute_rotated,
            $.execute_rotated_as,
            $.execute_if,
            $.execute_unless,
            $.execute_store,
            $.execute_run
        ),

        execute_as: $ => seq(
            'as',
            field('target', $.target_selector)
        ),

        execute_at: $ => seq(
            'at',
            field('target', $.target_selector)
        ),

        execute_align: $ => seq(
            'align',
            field('axes', $.align_axes)
        ),

        execute_anchored: $ => seq(
            'anchored',
            field('anchor', choice('eyes', 'feet'))
        ),

        execute_facing: $ => seq(
            'facing',
            field('pos', $.position)
        ),

        execute_facing_entity: $ => seq(
            'facing',
            'entity',
            field('target', $.target_selector),
            optional(field('anchor', choice('eyes', 'feet')))
        ),

        execute_in: $ => seq(
            'in',
            field('dimension', $.dimension)
        ),

        execute_positioned: $ => seq(
            'positioned',
            field('pos', $.position)
        ),

        execute_positioned_as: $ => seq(
            'positioned',
            'as',
            field('target', $.target_selector)
        ),

        execute_rotated: $ => seq(
            'rotated',
            field('rot', $.rotation)
        ),

        execute_rotated_as: $ => seq(
            'rotated',
            'as',
            field('target', $.target_selector)
        ),

        execute_if: $ => seq(
            'if',
            field('condition', $.execute_condition)
        ),

        execute_unless: $ => seq(
            'unless',
            field('condition', $.execute_condition)
        ),

        execute_store: $ => seq(
            'store',
            field('mode', choice('result', 'success')),
            field('target', $.store_target),
            optional(seq(
                field('type', choice('byte', 'short', 'int', 'long', 'float', 'double')),
                field('scale', $.number)
            ))
        ),

        execute_run: $ => seq(
            'run',
            field('command', $._command)
        ),


        _coordinate: $ => choice(
            $.number,  // Changed from $.absolute_coordinate
            $.relative_coordinate,
            $.local_coordinate
        ),

        relative_coordinate: $ => prec.left(1, choice(
            $.relative_coordinate_plain,
            $.relative_coordinate_offset
        )),

        relative_coordinate_plain: $ => '~',

        relative_coordinate_offset: $ => prec.right(2, seq(
            '~',
            optional('-'),
            $.number
        )),

        local_coordinate: $ => prec.left(1, choice(
            $.local_coordinate_plain,
            $.local_coordinate_offset
        )),

        local_coordinate_plain: $ => '^',

        local_coordinate_offset: $ => prec.right(2, seq(
            '^',
            optional('-'),
            $.number
        )),

        position: $ => seq(
            field('x', $._coordinate),
            field('y', $._coordinate),
            field('z', $._coordinate)
        ),


        rotation: $ => seq(
            field('yaw', $._coordinate),
            field('pitch', $._coordinate)
        ),

        align_axes: $ => /[xyz]+/,

        dimension: $ => choice(
            'minecraft:overworld',
            'minecraft:the_nether',
            'minecraft:the_end',
            $.identifier
        ),

        execute_condition: $ => choice(
            $.condition_block,
            $.condition_blocks,
            $.condition_data,
            $.condition_entity,
            $.condition_predicate,
            $.condition_score
        ),

        condition_block: $ => seq(
            'block',
            field('pos', $.position),
            field('block', seq(
                optional('minecraft:'),
                $.identifier,
                optional(seq(
                    '[',
                    repeat(seq($.block_state, optional(','))),
                    ']'
                ))
            ))
        ),

        condition_blocks: $ => seq(
            'blocks',
            field('start', $.position),
            field('end', $.position),
            field('destination', $.position),
            optional('masked')
        ),

        condition_data: $ => seq(
            'data',
            choice(
                seq('block', field('pos', $.position)),
                seq('entity', field('target', $.target_selector)),
                seq('storage', field('source', $.identifier))
            ),
            field('path', $.nbt_path)
        ),

        condition_entity: $ => seq(
            'entity',
            field('target', $.target_selector)
        ),

        condition_predicate: $ => seq(
            'predicate',
            field('id', $.identifier)
        ),

        condition_score: $ => seq(
            'score',
            field('target', $.target_selector),
            field('objective', $.identifier),
            choice(
                seq(
                    field('operator', choice('matches', '<', '<=', '=', '>=', '>')),
                    choice(
                        field('range', $.range_value),
                        seq(
                            field('source', $.target_selector),
                            field('source_objective', $.identifier)
                        )
                    )
                )
            )
        ),

        store_target: $ => choice(
            seq('score', field('target', $.target_selector), field('objective', $.identifier)),
            seq('block', field('pos', $.position), field('path', $.nbt_path)),
            seq('entity', field('target', $.target_selector), field('path', $.nbt_path)),
            seq('storage', field('source', $.identifier), field('path', $.nbt_path)),
            seq('bossbar', field('id', $.identifier), choice('value', 'max'))
        ),

        block_predicate: $ => seq(
            optional('minecraft:'),
            $.identifier,
            optional(seq(
                '[',
                repeat(seq($.block_state, optional(','))),
                ']'
            )),
            optional(seq(
                '{',
                repeat(seq($.nbt_tag, optional(','))),
                '}'
            ))
        ),

        nbt_path: $ => /[a-zA-Z0-9._{}\\[\\]]+/,

        block_state: $ => seq(
            field('key', $.identifier),
            '=',
            field('value', choice($.number, $.identifier, $.quoted_string))
        ),

        nbt_tag: $ => seq(
            field('key', $.identifier),
            ':',
            field('value', choice($.number, $.identifier, $.quoted_string, $.compound_tag))
        ),

        compound_tag: $ => seq(
            '{',
            repeat($.nbt_tag),
            '}'
        ),

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

        range_value: $ => choice(
            seq($.number, '..', $.number),  // 1..10
            seq($.number, '..'),            // 1..
            seq('..', $.number),            // ..10
            $.number                        // exact value
        ),

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
            optional('give'), // Make 'give' optional to support both forms
            field('target', $.target_selector),
            field('effect_type', choice(
                seq(optional('minecraft:'), $.vanilla_effect),
                $.custom_effect
            )),
            field('duration', $.number),
            field('amplifier', $.number)
        ),

        enchant_command: $ => seq(
            'enchant',
            field('target', $.target_selector),
            field('enchantment', choice(
                $.vanilla_enchant,
                $.custom_enchant
            )),
            field('level', $.number)
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

        vanilla_enchant: $ => choice(
            'minecraft:protection',
            'minecraft:fire_protection',
            'minecraft:feather_falling',
            'minecraft:blast_protection',
            'minecraft:projectile_protection',
            'minecraft:respiration',
            'minecraft:aqua_affinity',
            'minecraft:thorns',
            'minecraft:depth_strider',
            'minecraft:frost_walker',
            'minecraft:binding_curse',
            'minecraft:sharpness',
            'minecraft:smite',
            'minecraft:bane_of_arthropods',
            'minecraft:knockback',
            'minecraft:fire_aspect',
            'minecraft:looting',
            'minecraft:sweeping',
            'minecraft:efficiency',
            'minecraft:silk_touch',
            'minecraft:unbreaking',
            'minecraft:fortune',
            'minecraft:power',
            'minecraft:punch',
            'minecraft:flame',
            'minecraft:infinity',
            'minecraft:luck_of_the_sea',
            'minecraft:lure',
            'minecraft:mending',
            'minecraft:vanishing_curse',
            'minecraft:soul_speed',
            'minecraft:swift_sneak'
        ),

        custom_effect: $ => /"[^"]*"/,
        custom_enchant: $ => /"[^"]*"/,

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
