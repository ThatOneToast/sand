//! Typed projections and operands for component-owned storage values.
use super::{BlockPosition, FieldReference, ScoreOperand};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A checked NBT path in the block entity at a position.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::BlockDataAddress", module = "sand::advanced::compiler",
 summary = "An addressed block entity value.",
 context = "Defines a portable block position and checked NBT path shared by reads and writes.",
 minecraft = "Lowers to the canonical block data target and does not load chunks.",
 use_when = ["Reading or editing a known block entity field"],
 avoid_when = ["Storing pack-owned data independently of a block entity"],
 example = "use sand::advanced::compiler::BlockDataAddress;",
 fields(at = "Block position.", path = "Checked NBT path within the block entity."),
)]
pub struct BlockDataAddress {
    /// Absolute or relative block position.
    pub at: BlockPosition,
    /// The NBT path, validated before emission.
    pub path: String,
}

/// A State storage field and its checked nested projections.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::DataReference", module = "sand::advanced::compiler",
 summary = "A State storage field and its checked nested projections.",
 context = "Typed portable storage operands share the canonical State owner lifecycle and NBT command model.",
 minecraft = "Values are copied through command storage; dynamic indices are integer macro arguments and authored text remains data.",
 use_when = ["Lowering runtime strings, lists or record values from a typed frontend"],
 avoid_when = ["An integer score operand is sufficient"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 fields(field = "Owning State storage field.", path = "Nested members or list indices, in order."),
)]
pub struct DataReference {
    /// Owning State storage field.
    pub field: FieldReference,
    /// Nested members or list indices, in order. An empty path selects the root.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<DataProjection>,
}

/// One typed step into a stored value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::DataProjection", module = "sand::advanced::compiler",
 summary = "One typed step into a stored value.",
 context = "Typed portable storage operands share the canonical State owner lifecycle and NBT command model.",
 minecraft = "Values are copied through command storage; dynamic indices are integer macro arguments and authored text remains data.",
 use_when = ["Lowering runtime strings, lists or record values from a typed frontend"],
 avoid_when = ["An integer score operand is sufficient"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Member = "Select a named compound member.", Index = "Select a nonnegative list index.", Each = "Select every list element as a modification source."),
 variant_fields(Member(name = "Compound member identity."), Index(index = "Nonnegative index, evaluated before the modification.")),
)]
pub enum DataProjection {
    /// Select a named compound member.
    Member { name: String },
    /// Select a nonnegative list index. Missing or invalid selections skip writes.
    Index { index: ScoreOperand },
    /// Select every list element; only valid as the final modification source.
    Each,
}

/// An NBT literal, stored value, or scoreboard integer used in a data write.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::advanced::compiler::DataOperand", module = "sand::advanced::compiler",
 summary = "An NBT literal, stored value, or scoreboard integer used in a data write.",
 context = "Typed portable storage operands share the canonical State owner lifecycle and NBT command model.",
 minecraft = "Values are copied through command storage; dynamic indices are integer macro arguments and authored text remains data.",
 use_when = ["Lowering runtime strings, lists or record values from a typed frontend"],
 avoid_when = ["An integer score operand is sufficient"],
 example = "let schema = sand::advanced::compiler::Compiler::schema();",
 variants(Value = "A typed NBT literal.", Reference = "A stored value selection.", Score = "A scoreboard integer encoded as NBT int."),
 variant_fields(Value(value = "Typed NBT literal; opaque raw SNBT is unsupported."), Reference(value = "Stored value selection."), Score(value = "Integer score operand.")),
)]
pub enum DataOperand {
    /// A typed literal, staged outside any macro template.
    Value { value: sand_commands::NbtValue },
    /// A selection from another owned storage field.
    Reference { value: DataReference },
    /// A scoreboard integer encoded as NBT int.
    Score { value: ScoreOperand },
}
