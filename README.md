# SaveStudio — dépôt des jeux

Ce dépôt contient **tout ce qui décrit les jeux** pour [SaveStudio](https://github.com/martinlepro/The-ultimate-savefile-editor) :
où se trouvent les sauvegardes, **le code qui les comprend** (les codecs, compilés en WebAssembly), les onglets et valeurs modifiables (la disposition),
les icônes de la colonne de gauche, les polices, les thèmes de couleurs et les versions des boutiques.
SaveStudio, lui, ne fait que le style, la personnalisation et les fonctions « en plus » (partage entre ordinateurs via Tailscale…).
L'application n'en contient aucune copie : au **premier lancement** elle propose de télécharger ce dépôt
(fichiers listés dans `manifest.json`, vérifiés par SHA-256), puis, **à chaque démarrage**, compare `version` à celle installée et demande à l'utilisateur
s'il veut installer la mise à jour (rien n'est téléchargé sans son accord).
Ajouter ou corriger un jeu ne demande donc **aucune mise à jour de l'application**.

```
manifest.json            version du pack, version minimale de l'appli, empreintes des fichiers, codecs, polices, jeux (+ versions boutiques)
codecs/                  code Rust qui lit/écrit les formats xml, gd, hk, nbt (tests inclus)
wasm/savecodecs.wasm     ce code compilé en WebAssembly — généré par le workflow, c'est ce que l'application télécharge
themes.json              thèmes de couleurs (un par jeu, choisis dans l'en-tête de sa définition)
icons.txt                icônes pixel-art 12x12 de la colonne de gauche
games/<id>.sve           une définition par jeu (une seule ligne)
windows|linux|android/locations.svl    où chercher les sauvegardes, par plateforme (macOS n'est pas géré)
fonts/                   polices
tools/make_manifest.py   recalcule les empreintes, la date et la version
tools/build_fonts.py, build_icons.py   (re)génèrent fonts/ et icons.txt
.github/workflows/manifest.yml   teste et compile les codecs, puis fait tourner make_manifest.py à chaque envoi sur main
```

## Publier une modification (version du pack)

1. Modifiez un fichier (sur github.com ou en local) et envoyez sur `main`.
2. Le workflow **Pack** teste les codecs, recompile `wasm/savecodecs.wasm`, valide le pack, recalcule les empreintes et **incrémente `version`** (0.0.+1) dans `manifest.json`
   (commit automatique `[skip ci]`, qui contient aussi le `.wasm` recompilé). Les applications installées proposent alors la mise à jour au prochain démarrage.
3. En local : `python3 tools/make_manifest.py` (même effet), `--bump minor|major`, `--version 2.0.0`, `--no-bump`,
   `--min-app 0.3.0` (version minimale de SaveStudio exigée par ce pack, à relever quand un `.sve` utilise une option récente du moteur),
   `--check` (échoue si `manifest.json` n'est pas à jour — utile avant un envoi).

⚠ Ne modifiez pas les empreintes à la main : un fichier dont l'empreinte ne correspond pas est refusé par l'application.

### Versions des boutiques (affichées en lecture seule)
Aucune boutique ne fournit d'API publique pour la version « marketing » d'un jeu : ces valeurs sont donc saisies ici, à la main, et peuvent
être en retard. Elles apparaissent sur la carte du jeu et sous le titre de l'éditeur.
```
python3 tools/make_manifest.py --store stardewvalley Steam 1.6.16 https://store.steampowered.com/app/413150
python3 tools/make_manifest.py --store stardewvalley "Google Play" 1.6.16 "https://play.google.com/store/apps/details?id=com.chucklefish.stardewvalley"
```
Dans l'éditeur, SaveStudio affiche aussi la version du jeu qui a écrit la sauvegarde ouverte (option `ver=` de la définition).

## Ajouter un jeu

1. `games/<id>.sve` (voir le format plus bas), avec `<id>` identique à la clé dans `manifest.json` → `games`.
2. Une ligne `<id>|<motif>` dans chaque `windows|linux|android/locations.svl`.
3. Si besoin : un thème dans `themes.json`, une police dans `fonts/` + `manifest.json` → `fonts`, des icônes dans `icons.txt`.
4. Entrée dans `manifest.json` → `games` : `"<id>": {"file": "games/<id>.sve", "stores": []}`, puis `python3 tools/make_manifest.py`.
5. Essai sans rien publier : `savestudio --pack /chemin/de/ce/depot --open <sauvegarde>`.

Le moteur sait lire quatre formats : `xml` (Stardew, Celeste, Geometry Dash…), `gd` (Geometry Dash : XOR + base64 + gzip), `hk` (Hollow Knight : AES + base64) et `nbt` (Minecraft Java).
Un autre format de fichier demande une mise à jour de l'application (nouveau codec), puis `--min-app` ici.

## `manifest.json`

| Clé | Rôle |
|---|---|
| `version` | `1.2.3` ; l'application propose la mise à jour si elle est plus grande que la version installée |
| `min_app` | version minimale de SaveStudio ; si l'application est plus ancienne, elle le dit et n'installe pas ce pack |
| `updated` | date (remplie par l'outil) |
| `themes`, `icons`, `locations` | fichiers des thèmes, des icônes, et des emplacements par plateforme |
| `fonts` | `"nom": {"regular": "fonts/x.ttf", "bold": "fonts/x-bold.ttf", "snap": 10}` ; `snap` : les polices pixel restent nettes à des multiples de cette taille (px), 0 = libre |
| `games` | `"<id>": {"file": "games/<id>.sve", "stores": [{"name": "Steam", "version": "1.6.15", "url": "…"}]}` (ordre = ordre à l'écran) |
| `files` | **généré** : chemin → SHA-256 de chaque fichier utilisé |

## Emplacements : `*/locations.svl`
Une ligne par motif : `<id du jeu>|<motif>` ; `#` commence un commentaire. Plusieurs lignes pour un même jeu = plusieurs emplacements essayés.
Motifs : `~` (dossier personnel), `%VARIABLE%`, `*` (joker), `{d}` (nom du dossier parent : Stardew `Saves/Ferme_123/Ferme_123`).
L'application ne lit que le dossier de sa plateforme (Android : `android/`, stockage partagé `/storage/emulated/0/SaveStudio/<Jeu>/…`).

## Thèmes : `themes.json`
Un objet par thème ; les clés absentes reprennent celles du thème `wood`. Couleurs `"#rrggbb"` : `bg bg_edge title title_shadow panel panel_edge card card_alt ink ink_soft input input_ink tab tab_sel tab_ink outline hi lo accent knob btn_ink preset` ;
`btn` : 4 couleurs de boutons ; nombres : `fs` (échelle des textes), `bw` (épaisseur des bordures), `round` (arrondi) ; booléens : `caps` (titres en capitales), `bevel`.
L'utilisateur peut ensuite surcharger fond / panneaux / accent dans l'application.

## Icônes : `icons.txt`
Blocs séparés par une ligne vide : une ligne de nom, puis 12 lignes de 12 caractères (palette dans `tools/build_icons.py`, qui régénère le fichier).
Une définition s'y réfère par nom (`#Onglet|nom-d-icône`, `+Carte|nom-d-icône`).

## Définitions : `games/<id>.sve`
Un jeu = **un seul fichier d'une seule ligne**. Enregistrements séparés par `;`, champs par `|`, `\` échappe `; | \`.
Les espaces et retours à la ligne entre enregistrements sont ignorés.

```
@id|Nom|Court|police|codec|thème|l1=..|l2=..|ver=chemin|mirror=fichier:src->dst     jeu
#Catégorie|icône                                                                     onglet de gauche
+Sous-catégorie|icône|c=N                                                            carte (N colonnes)
type|Libellé|chemin|options                                                          champ
```

* **police** : clé de `manifest.json` → `fonts` · **codec** : `xml`, `gd`, `hk`, `nbt` · **thème** : clé de `themes.json`.
* `l1`/`l2` : titre et sous-titre de la page, avec `{chemin}` pour insérer une valeur du fichier (`{chemin:c}` : première lettre en capitale).
* `ver=chemin` : où lire, dans la sauvegarde, la version du jeu qui l'a écrite (affichée en lecture seule).
* `mirror=fichier:préfixe->préfixe` : **fichier compagnon** dans le même dossier, recopié à chaque modification.
  Stardew : `mirror=SaveGameInfo:SaveGame/player->Farmer` — le menu *Charger* du jeu lit `SaveGameInfo`, pas la sauvegarde ; toute valeur modifiée sous
  `SaveGame/player/…` est aussi écrite sous `Farmer/…` (champs absents ignorés), les deux fichiers étant sauvegardés ensemble avec une copie de sécurité chacun.
* Motifs de détection (ancien format, toujours accepté) : `win:` / `lin:` / `and:` dans l'en-tête ; préférez `locations.svl`.
* **types** : `n` nombre · `t` texte · `b` booléen · `e` liste de choix · `s` curseur · `l` liste · `g` grille de cases · `i` information (lecture seule)
* **options** : `min` `max` `st`(pas) `dp`(décimales) `u`(unité) `sc`(échelle) `off`(décalage) `btn=Libellé:valeur`
  `bh`(en-tête du bouton) `o=Libellé:valeur,…` `r=a-b` `names=` `n=`/`v=` (listes) `xp=`/`xt=` `ro=1`
  `also=chemin:valeur` (autre valeur écrite par les boutons ; `=` recopie la valeur) et
  `sync=chemin` (à **chaque** modification, la même valeur brute est aussi écrite à `chemin` ; `sync=chemin:#` : l'indice de l'option choisie d'une liste `e`).
  `min`/`max`/`btn` sont en unités **affichées**, `also`/`sync` en valeur brute.
  Exemple Stardew : le jour/saison/année du menu *Charger* viennent de `player/dayOfMonthForSaveGame` etc. :
  `s|Day of month|SaveGame/dayOfMonth|min=1|max=28|sync=SaveGame/player/dayOfMonthForSaveGame`,
  `e|Season|SaveGame/currentSeason|o=Spring:spring,Summer:summer,Fall:fall,Winter:winter|sync=SaveGame/player/seasonForSaveGame:#`.
* **chemins** : `a/b/c`, `a/b[2]`, `a/b[*]` (tous), `a?b` (alternatives), `~clé` (plist : valeur qui suit la clé), `@scale` / `@backup` (réglages de l'application).

## Notes sur ce pack
* Les chemins `ver=` de Celeste (`SaveData/Version`), Hollow Knight (`playerData/version`) et Minecraft (`Data/Version/Name`) n'ont pas pu être
  vérifiés sur de vraies sauvegardes : si le champ n'existe pas, la version de la sauvegarde n'est simplement pas affichée.
* Geometry Dash et Minecraft n'ont pas de version de boutique renseignée (`"stores": []`) : ajoutez-la avec `--store` après vérification.
* Les versions Steam renseignées (Stardew 1.6.15, Celeste 1.4.0.0, Hollow Knight 1.5.78.11833) sont à confirmer.

## Licences des polices — à vérifier
`fonts/` contient des sous-ensembles Latin-1 de polices fournies. Pixelify Sans et VT323 sont sous licence libre (SIL OFL).
**Trajan Pro** (Hollow Knight) est une police commerciale d'Adobe et **CELESTE.otf** a une licence inconnue : les publier dans un dépôt **public**
peut ne pas être permis. Gardez le dépôt privé (l'application ne pourra alors pas le télécharger sans authentification) ou remplacez ces deux fichiers
(par exemple par *Cinzel*, libre et proche de Trajan, en gardant les mêmes noms de fichiers), puis `python3 tools/build_fonts.py <dossier_polices> <feuille_de_glyphes.png>`.

## Les codecs (le code qui comprend la sauvegarde)

`codecs/` est un petit crate Rust (`cargo test --release --manifest-path codecs/Cargo.toml`) qui sait ouvrir un fichier selon le codec
déclaré par le jeu (5ᵉ champ de l'en-tête du `.sve` : `xml`, `gd`, `hk`, `nbt`), lire/écrire des valeurs par chemin (`SaveGame/player/money`) et
ré-encoder le fichier à l'identique. Corriger un bug de lecture, ou ajouter un format, se fait ici — sans publier de nouvelle application
tant que l'**ABI** (ci-dessous) ne change pas.

Pour compiler à la main : `rustup target add wasm32-unknown-unknown`, puis
`cargo build --release --manifest-path codecs/Cargo.toml --target wasm32-unknown-unknown` et copier
`codecs/target/wasm32-unknown-unknown/release/savecodecs.wasm` vers `wasm/savecodecs.wasm` (le workflow le fait tout seul).
Ajouter un format : un module dans `codecs/src/`, une ligne dans `open()` et dans `CODECS` (`lib.rs`), puis le nom dans `codecs.formats` de `manifest.json`.

**Sécurité** : le module n'a *aucune* importation (pas de fichiers, pas de réseau, pas d'horloge) ; l'application refuse un module qui en demande.
Il tourne dans l'interpréteur WebAssembly de l'application et ne voit que les octets de la sauvegarde ouverte.

**ABI 1** (`codecs/src/ffi.rs`, `codecs.abi` dans le manifeste) : `ss_abi`, `ss_alloc/ss_free`, `ss_result_ptr/ss_result_len` (tampon résultat),
`ss_codecs`, `ss_open(codec, données) → handle`, `ss_close`, `ss_get(handle, chemin) → 0 absent | 1 bool | 2 entier | 3 flottant | 4 texte`,
`ss_set(handle, chemin, type, valeur)`, `ss_expand(handle, chemin)` (jokers), `ss_encode(handle)`. Si l'ABI change, incrémentez-la et relevez `--min-app`.
