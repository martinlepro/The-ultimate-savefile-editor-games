#!/usr/bin/env python3
"""Régénère `manifest.json` : empreintes SHA-256 des fichiers du pack, date, et numéro de version.

Usage (à la racine du dépôt) :
  python3 tools/make_manifest.py                     recalcule les empreintes ; si un fichier a changé, passe la version en +0.0.1
  python3 tools/make_manifest.py --bump minor        … en +0.1.0   (major : +1.0.0)
  python3 tools/make_manifest.py --version 2.0.0     … impose la version
  python3 tools/make_manifest.py --min-app 0.3.0     version minimale de SaveStudio exigée par ce pack
  python3 tools/make_manifest.py --store stardewvalley Steam 1.6.16 https://store.steampowered.com/app/413150
                                                     ajoute/met à jour la version d'un jeu sur une boutique (affichée en lecture seule)
  python3 tools/make_manifest.py --check             ne modifie rien ; code 1 si manifest.json n'est pas à jour ou si le pack est invalide

Le workflow GitHub `.github/workflows/manifest.yml` fait cela automatiquement à chaque envoi sur `main`.
"""
import argparse, datetime, hashlib, json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MANIFEST = os.path.join(ROOT, "manifest.json")


def safe(p):
    return bool(p) and not p.startswith("/") and "\\" not in p and ":" not in p and ".." not in p.split("/")


def referenced(m):
    out = set()
    for k in ("themes", "icons"):
        if m.get(k):
            out.add(m[k])
    out.update(m.get("locations", {}).values())
    for f in m.get("fonts", {}).values():
        out.add(f["regular"])
        if f.get("bold"):
            out.add(f["bold"])
    for g in m.get("games", {}).values():
        out.add(g["file"])
    return sorted(out)


def validate(m):
    errs = []
    if not re.fullmatch(r"\d+(\.\d+){0,3}", m.get("version", "")):
        errs.append("« version » doit ressembler à 1.2.3")
    for p in referenced(m):
        if not safe(p):
            errs.append(f"chemin refusé : {p}")
        elif not os.path.isfile(os.path.join(ROOT, p)):
            errs.append(f"fichier manquant : {p}")
    for gid, g in m.get("games", {}).items():
        path = os.path.join(ROOT, g["file"])
        if not os.path.isfile(path):
            continue
        src = open(path, encoding="utf-8").read().strip()
        if "\n" in src.replace("\r", ""):
            # un retour à la ligne est toléré par l'application, mais le format officiel tient sur une ligne
            errs.append(f"{g['file']} : le format compact tient sur une seule ligne")
        if not src.startswith(f"@{gid}|"):
            errs.append(f"{g['file']} : doit commencer par @{gid}|")
        for r in src.split(";"):
            r = r.strip()
            if r and r[0] not in "@#+" and r.split("|")[0] not in ("n", "t", "b", "e", "s", "l", "g", "i"):
                errs.append(f"{g['file']} : type de champ inconnu dans « {r[:40]} »")
                break
        for s in g.get("stores", []):
            if not s.get("name") or not s.get("version"):
                errs.append(f"{gid} : chaque boutique a besoin de « name » et « version »")
    for name, f in m.get("fonts", {}).items():
        if "regular" not in f:
            errs.append(f"police {name} : « regular » manquant")
    return errs


def hashes(m):
    h = {}
    for p in referenced(m):
        fp = os.path.join(ROOT, p)
        if os.path.isfile(fp):
            h[p] = hashlib.sha256(open(fp, "rb").read()).hexdigest()
    return h


def bump(v, kind):
    n = [int(x) for x in v.split(".")] + [0] * (3 - len(v.split(".")))
    if kind == "major":
        return f"{n[0] + 1}.0.0"
    if kind == "minor":
        return f"{n[0]}.{n[1] + 1}.0"
    return f"{n[0]}.{n[1]}.{n[2] + 1}"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--bump", choices=["patch", "minor", "major"], default="patch")
    ap.add_argument("--version")
    ap.add_argument("--min-app")
    ap.add_argument("--no-bump", action="store_true")
    ap.add_argument("--store", nargs="+", metavar=("JEU", "BOUTIQUE VERSION [URL]"))
    a = ap.parse_args()

    m = json.load(open(MANIFEST, encoding="utf-8"))
    errs = validate(m)
    if errs:
        print("Pack invalide :\n  " + "\n  ".join(errs))
        sys.exit(1)
    changed = False
    if a.store:
        if len(a.store) < 3 or a.store[0] not in m["games"]:
            sys.exit("usage : --store <id du jeu> <boutique> <version> [url]  (jeux : " + ", ".join(m["games"]) + ")")
        gid, name, ver = a.store[:3]
        url = a.store[3] if len(a.store) > 3 else ""
        lst = m["games"][gid].setdefault("stores", [])
        cur = next((s for s in lst if s["name"].lower() == name.lower()), None)
        new = {"name": name, "version": ver, "url": url or (cur or {}).get("url", "")}
        if cur is None:
            lst.append(new)
        else:
            cur.update(new)
        changed = True
    if a.min_app and a.min_app != m.get("min_app"):
        m["min_app"] = a.min_app
        changed = True
    new_files = hashes(m)
    if new_files != m.get("files"):
        changed = True
    if a.check:
        if changed:
            print("manifest.json n'est pas à jour : lancez  python3 tools/make_manifest.py")
            sys.exit(1)
        print("OK : manifest.json est à jour, version", m["version"])
        return
    m["files"] = new_files
    if changed:
        m["version"] = a.version or (m["version"] if a.no_bump else bump(m["version"], a.bump))
        m["updated"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    elif a.version:
        m["version"] = a.version
    json.dump(m, open(MANIFEST, "w", encoding="utf-8"), indent=2, ensure_ascii=False)
    open(MANIFEST, "a").write("\n")
    print(("Mis à jour" if changed else "Inchangé") + f" : version {m['version']}, {len(new_files)} fichiers")


if __name__ == "__main__":
    main()
