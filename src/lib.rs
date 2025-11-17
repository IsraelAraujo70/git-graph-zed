use zed_extension_api as zed;

struct GitGraphExtension;

impl zed::Extension for GitGraphExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitGraphExtension
    }
}

zed::register_extension!(GitGraphExtension);
