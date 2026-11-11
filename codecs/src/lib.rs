//! Codecs de sauvegardes de SaveStudio. Compilé en WebAssembly (`cdylib`) pour l'application ; en `rlib` pour les tests.
//!
//! ABI (version 1) : l'hôte appelle les fonctions `ss_*` exportées ; tout ce qui n'est pas un entier passe par la
//! mémoire linéaire (`ss_alloc` / `ss_free`). Les résultats de type octets ou texte sont lus dans le « tampon résultat »
//! (`ss_result_ptr` / `ss_result_len`), valable jusqu'au prochain appel. Voir `ffi.rs`.
pub mod doc;
mod ffi;
mod gd;
mod hk;
mod nbt;
mod xml;

/// Ouvre un fichier selon le nom du codec déclaré par le jeu (`xml`, `gd`, `hk`, `nbt`).
pub fn open(codec: &str, bytes: &[u8]) -> Result<Box<dyn doc::Doc>, String> {
    match codec {
        "xml" => Ok(Box::new(xml::XmlDoc::parse(bytes.to_vec(), xml::Wrap::Plain)?)),
        "gd" => Ok(Box::new(gd::open(bytes)?)),
        "hk" => Ok(Box::new(hk::HkDoc::open(bytes)?)),
        "nbt" => Ok(Box::new(nbt::NbtDoc::open(bytes)?)),
        c => Err(format!("codec inconnu : {c}")),
    }
}

pub const CODECS: &str = "xml,gd,hk,nbt";
