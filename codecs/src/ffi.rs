//! Points d'entrée WebAssembly (ABI 1). Aucune importation : le module ne peut rien faire d'autre que calculer.
use crate::doc::{Doc, Scalar};
use std::cell::RefCell;

pub const ABI: u32 = 1;

#[derive(Default)]
struct State {
    docs: Vec<Option<Box<dyn Doc>>>,
    res: Vec<u8>,
}

thread_local! {
    static ST: RefCell<State> = RefCell::new(State::default());
}

unsafe fn bytes<'a>(p: *const u8, n: u32) -> &'a [u8] {
    if n == 0 { &[] } else { std::slice::from_raw_parts(p, n as usize) }
}

unsafe fn text<'a>(p: *const u8, n: u32) -> &'a str {
    std::str::from_utf8(bytes(p, n)).unwrap_or("")
}

fn put(v: Vec<u8>) {
    ST.with(|s| s.borrow_mut().res = v);
}

#[no_mangle]
pub extern "C" fn ss_abi() -> u32 {
    ABI
}

#[no_mangle]
pub extern "C" fn ss_alloc(len: u32) -> *mut u8 {
    let b = vec![0u8; len.max(1) as usize].into_boxed_slice();
    Box::into_raw(b) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn ss_free(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        drop(Box::from_raw(std::slice::from_raw_parts_mut(ptr, len.max(1) as usize)));
    }
}

#[no_mangle]
pub extern "C" fn ss_result_ptr() -> *const u8 {
    ST.with(|s| s.borrow().res.as_ptr())
}

#[no_mangle]
pub extern "C" fn ss_result_len() -> u32 {
    ST.with(|s| s.borrow().res.len() as u32)
}

/// Liste des codecs gérés, séparés par des virgules (dans le tampon résultat).
#[no_mangle]
pub extern "C" fn ss_codecs() -> i32 {
    put(crate::CODECS.as_bytes().to_vec());
    0
}

/// Ouvre un fichier. Retourne un identifiant (>= 1) ou 0 (message d'erreur dans le tampon résultat).
#[no_mangle]
pub unsafe extern "C" fn ss_open(codec: *const u8, codec_len: u32, data: *const u8, data_len: u32) -> i32 {
    match crate::open(text(codec, codec_len), bytes(data, data_len)) {
        Ok(d) => ST.with(|s| {
            let mut s = s.borrow_mut();
            match s.docs.iter().position(|x| x.is_none()) {
                Some(i) => {
                    s.docs[i] = Some(d);
                    i as i32 + 1
                }
                None => {
                    s.docs.push(Some(d));
                    s.docs.len() as i32
                }
            }
        }),
        Err(e) => {
            put(e.into_bytes());
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn ss_close(h: i32) {
    ST.with(|s| {
        if let Some(x) = s.borrow_mut().docs.get_mut((h - 1).max(0) as usize) {
            *x = None;
        }
    });
}

fn with_doc<R>(h: i32, f: impl FnOnce(&mut dyn Doc) -> R) -> Option<R> {
    ST.with(|s| s.borrow_mut().docs.get_mut((h - 1).max(0) as usize).and_then(|x| x.as_mut()).map(|d| f(&mut **d)))
}

/// Lit une valeur. 0 = absente ; 1 booléen, 2 entier, 3 flottant, 4 texte (valeur en texte dans le tampon résultat).
#[no_mangle]
pub unsafe extern "C" fn ss_get(h: i32, path: *const u8, path_len: u32) -> i32 {
    let p = text(path, path_len);
    match with_doc(h, |d| d.get(p)).flatten() {
        None => 0,
        Some(v) => {
            let (tag, s) = match v {
                Scalar::Bool(b) => (1, b.to_string()),
                Scalar::Int(i) => (2, i.to_string()),
                Scalar::Float(f) => (3, format!("{f}")),
                Scalar::Text(t) => (4, t),
            };
            put(s.into_bytes());
            tag
        }
    }
}

/// Écrit une valeur (mêmes étiquettes que `ss_get`). 0 = ok, 1 = erreur (message dans le tampon résultat).
#[no_mangle]
pub unsafe extern "C" fn ss_set(h: i32, path: *const u8, path_len: u32, tag: i32, val: *const u8, val_len: u32) -> i32 {
    let p = text(path, path_len);
    let t = text(val, val_len);
    let v = match tag {
        1 => Scalar::Bool(t == "true"),
        2 => match t.parse::<i64>() {
            Ok(i) => Scalar::Int(i),
            Err(_) => {
                put(format!("entier invalide « {t} »").into_bytes());
                return 1;
            }
        },
        3 => match t.parse::<f64>() {
            Ok(f) => Scalar::Float(f),
            Err(_) => {
                put(format!("nombre invalide « {t} »").into_bytes());
                return 1;
            }
        },
        _ => Scalar::Text(t.to_string()),
    };
    match with_doc(h, |d| d.set(p, v)) {
        Some(Ok(())) => 0,
        Some(Err(e)) => {
            put(e.into_bytes());
            1
        }
        None => {
            put(b"document inconnu".to_vec());
            1
        }
    }
}

/// Remplace le premier joker du chemin par chaque index/clé existant : résultat = chemins séparés par « \n ». Retourne leur nombre.
#[no_mangle]
pub unsafe extern "C" fn ss_expand(h: i32, path: *const u8, path_len: u32) -> i32 {
    let p = text(path, path_len);
    let v = with_doc(h, |d| d.expand(p)).unwrap_or_default();
    let n = v.len() as i32;
    put(v.join("\n").into_bytes());
    n
}

/// Réencode le fichier : 0 = ok (octets dans le tampon résultat), 1 = erreur (message dans le tampon résultat).
#[no_mangle]
pub extern "C" fn ss_encode(h: i32) -> i32 {
    match with_doc(h, |d| d.encode()) {
        Some(Ok(b)) => {
            put(b);
            0
        }
        Some(Err(e)) => {
            put(e.into_bytes());
            1
        }
        None => {
            put(b"document inconnu".to_vec());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_get(h: i32, p: &str) -> (i32, String) {
        let t = unsafe { ss_get(h, p.as_ptr(), p.len() as u32) };
        (t, String::from_utf8(ST.with(|s| s.borrow().res.clone())).unwrap())
    }

    #[test]
    fn open_get_set_encode_through_the_abi() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?><A><b>12</b><c>hi</c></A>"#;
        let c = "xml";
        let h = unsafe { ss_open(c.as_ptr(), 3, xml.as_ptr(), xml.len() as u32) };
        assert!(h >= 1);
        assert_eq!(call_get(h, "A/c"), (4, "hi".to_string()));
        assert_eq!(call_get(h, "A/nothing").0, 0);
        let (p, v) = ("A/b", "99");
        assert_eq!(unsafe { ss_set(h, p.as_ptr(), 3, 2, v.as_ptr(), 2) }, 0);
        assert_eq!(unsafe { ss_set(h, p.as_ptr(), 3, 2, "x".as_ptr(), 1) }, 1);
        assert_eq!(ss_encode(h), 0);
        let out = ST.with(|s| s.borrow().res.clone());
        assert!(String::from_utf8(out).unwrap().contains("<b>99</b>"));
        ss_close(h);
        // codec inconnu
        let bad = unsafe { ss_open("zzz".as_ptr(), 3, xml.as_ptr(), 1) };
        assert_eq!(bad, 0);
    }
}
