use crate::compiler_plugin;
use crate::gen::code_writer::CodeWriter;

pub(crate) fn gen_mod_rs(mods: &[String]) -> compiler_plugin::GenResult {
    let v = CodeWriter::with(|w| {
        w.comment(&format!("{}generated", "@"));
        w.write_line("");
        for m in mods {
            w.write_line(&format!("pub mod {};", m));
        }
    });
    compiler_plugin::GenResult {
        name: "mod.rs".to_owned(),
        content: v.into_bytes(),
    }
}
