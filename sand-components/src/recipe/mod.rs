//! Recipe builders for `data/<namespace>/recipe/` JSON files (Minecraft 1.21+).
//!
//! Each recipe type lives in its own submodule:
//!
//! | Module          | Type(s)                                         |
//! |----------------|-------------------------------------------------|
//! | `shaped`       | [`ShapedRecipe`]                                |
//! | `shapeless`    | [`ShapelessRecipe`]                             |
//! | `cooking`      | [`CookingRecipe`]                               |
//! | `stonecutting` | [`StonecuttingRecipe`]                          |
//! | `smithing`     | [`SmithingTransformRecipe`], [`SmithingTrimRecipe`] |
//! | `types`        | [`Ingredient`], [`RecipeResult`], [`CookingType`] (shared) |

pub mod cooking;
pub mod shaped;
pub mod shapeless;
pub mod smithing;
pub mod stonecutting;
pub mod types;

pub use cooking::CookingRecipe;
pub use shaped::ShapedRecipe;
pub use shapeless::ShapelessRecipe;
pub use smithing::{SmithingTransformRecipe, SmithingTrimRecipe};
pub use stonecutting::StonecuttingRecipe;
pub use types::{CookingType, Ingredient, RecipeResult};
