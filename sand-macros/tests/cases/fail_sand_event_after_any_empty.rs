use sand_core::events::SandEventDispatch;

fn main() {
    let _ = SandEventDispatch::after_any::<()>();
}
