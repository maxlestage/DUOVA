# Duova · Qui doit quoi ?

Le partage de dépenses au centime près : on crée un groupe, on ajoute les membres et
les dépenses, et Duova dit qui rembourse qui, en un minimum de virements.

| Partie | Où | Technologies |
|---|---|---|
| Site de présentation | [`site/`](site/) | Rust 1.100, [Paquets](https://github.com/maxlestage/Paquets), WebAssembly, Canvas 2D |
| App iPhone | [`ios/`](ios/) | Swift 6.4, SwiftUI, SQLite 3.54.1, iOS 27, Xcode 27 |

Le même algorithme est écrit deux fois et testé des deux côtés, avec les mêmes cas :
[`site/src/split.rs`](site/src/split.rs) (Rust, utilisé par le serveur et par le simulateur
du site) et [`ios/DuovaKit`](ios/DuovaKit) (Swift, utilisé par l'app).

## L'algorithme

1. Les montants sont des **centimes entiers** : jamais `0,1 + 0,2 = 0,30000000000000004`.
2. Une dépense se partage à parts égales **sans perdre un centime** : 10 € pour 3, c'est
   3,34 €, 3,33 € et 3,33 €.
3. Le **solde** de chacun est ce qu'il a payé moins sa part ; la somme des soldes vaut 0.
4. À chaque tour, le plus gros débiteur rembourse le plus gros créancier : **au plus
   n − 1 virements** pour n personnes.

Exemple : Alice paie 90 € de courses pour Alice, Bob et Chloé ; Bob paie 30 € d'essence
pour Bob et Chloé. Soldes : Alice +60 €, Bob −15 €, Chloé −45 €. Deux virements :
Chloé → Alice 45 €, Bob → Alice 15 €.

## Le site (`site/`)

Un site Paquets mobile first, en français, tout en 2D, sans aucun défilement horizontal :

- **Scène 2D du hero** (Canvas, écrite en Rust) : des pièces, des billets et des tickets
  tombent, rebondissent, s'entrechoquent et s'empilent, avec un petit moteur physique
  (gravité, collisions, frottements, rotation) ; un appui les relance en l'air, la souris
  les pousse.
- **Scène 2D de l'algorithme** : Alice, Bob et Chloé en bulles, des chemins en pointillés
  qui avancent, et les pièces des deux virements qui volent de l'un à l'autre.
- **Simulateur** : un îlot Rust compilé en WebAssembly, avec le vrai calcul. On ajoute des
  membres et des dépenses, les soldes et les remboursements se recalculent et s'animent,
  et le récap se copie pour WhatsApp.
- **Animations 2D** : titre qui se lève mot à mot, aurores, grille de lumière, cartes
  aimantées par le pointeur avec un halo, sections qui apparaissent au défilement,
  iPhone et pastilles en parallaxe, anneaux qui tournent, pièce qui se retourne, bandeau
  défilant, compteurs. `prefers-reduced-motion` coupe tout. Aucune transformation 3D
  (un test le vérifie).
- **SEO** : page rendue par le serveur, Open Graph, JSON-LD `MobileApplication`, sitemap,
  PWA installable avec ses icônes, Content-Security-Policy stricte.

```bash
cd site
# 1. Le bundle du navigateur (îlots, scènes 2D, animations) → pkg/
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir pkg target/wasm32-unknown-unknown/wasm-release/duova_site.wasm
# 2. Le serveur, sur http://localhost:3000 (PORT sur Heroku)
cargo run --release
```

`wasm-bindgen-cli` doit avoir la version de `wasm-bindgen` de `Cargo.lock` (0.2.130) :
`cargo install wasm-bindgen-cli --version 0.2.130 --locked`. Sans `pkg/`, le site
s'affiche quand même, sans les scènes 2D ni le simulateur.

Vérifications : `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings`, `cargo test`.

Paquets et ses paquets sont des dépôts privés : en CI et sur Heroku, un jeton GitHub qui
peut les lire est attendu dans `REPOS_TOKEN` (secret GitHub Actions, config var Heroku).

### Branches et merge automatique

On travaille sur une branche (par exemple `dev`) : la CI vérifie le site et l'app iOS, et
si tout est vert, la branche est **mergée automatiquement dans `master`** (sauf les
branches `wip/…`), puis `master` est déployé.

### Déployer

- **GitHub Pages** (workflow *Deploy to GitHub Pages*) : le site est exporté en fichiers
  statiques par l'app elle-même (`cargo run --release -- export dist`, avec `BASE_PATH`
  et `SITE_URL`), puis publié. À faire une fois : *Settings → Pages → Build and
  deployment → Source : GitHub Actions*. Il se lance après chaque merge automatique, à
  chaque push sur `master`, ou à la main depuis l'onglet *Actions* (aussi depuis l'app
  GitHub).
- **Heroku** (buildpack `emk/rust`, comme Paquets) : `RustConfig`, `Procfile` et
  `heroku-build.sh`. Le workflow *Deploy to Heroku* déploie `master` par l'API Heroku si
  le secret `HEROKU_API_KEY`, la variable `HEROKU_APP_NAME` et la config var
  `REPOS_TOKEN` sont définis ; sinon il ne fait rien.
- **Docker** : `REPOS_TOKEN=… docker build --secret id=repos_token,env=REPOS_TOKEN -t duova .`

## L'app iPhone (`ios/`)

Ouvrir `ios/Duova.xcodeproj` avec Xcode 27, choisir son équipe dans *Signing &
Capabilities*, puis lancer. Le projet utilise des dossiers synchronisés : un fichier
ajouté dans `ios/Duova/` fait partie de l'app.

- `DuovaKit` (paquet Swift) : modèles, centimes, parts, soldes, virements, récap, et la
  base SQLite ; testé avec Swift Testing (`swift test --package-path ios/DuovaKit`, sur
  macOS comme sur Linux).
- `CSQLite` : SQLite 3.54.1, l'amalgamation officielle de sqlite.org compilée dans l'app
  (empreinte SHA3-256 vérifiée, voir [`Sources/CSQLite`](ios/DuovaKit/Sources/CSQLite)) :
  la même version récente sur chaque iPhone, quelle que soit la version d'iOS.
  `./update.sh` y passe à la dernière version publiée.
- `Duova` (app SwiftUI) : liste des groupes, détail d'un groupe (total, barres de soldes,
  remboursements, dépenses, membres), formulaire de dépense avec la part de chacun en
  direct, partage du récap avec la feuille de partage d'iOS.

Les données sont dans une base SQLite sur l'appareil (`Application Support/Duova/duova.sqlite`,
journal WAL), protégée par le chiffrement d'iOS. Une table par sorte de chose, liées par des
clés étrangères :

| Table | Colonnes |
|---|---|
| `groups` | `id`, `name`, `emoji`, `created_at` |
| `members` | `id`, `group_id` → `groups`, `name` |
| `expenses` | `id`, `group_id` → `groups`, `title`, `amount` (centimes, `> 0`), `paid_by` → `members`, `date` |
| `expense_participants` | `expense_id` → `expenses`, `member_id` → `members`, `position` |

Supprimer un groupe supprime ses membres et ses dépenses ; un membre qui a payé ou partage
une dépense ne peut pas être supprimé (la base le refuse aussi). Le schéma est versionné
(`PRAGMA user_version`). Les groupes du fichier JSON de la première version y sont
importés au premier lancement.

La CI construit l'app et lance les tests sur le runner `xcode-27` de GitHub Actions.

## Versions

Relevées sur le site officiel de chacun, le 25 novembre 2026 :

| Outil | Version | Source |
|---|---|---|
| Rust | 1.100.0 | [rust-lang.org](https://www.rust-lang.org/) (canal stable) |
| wasm-bindgen | 0.2.130 | [crates.io](https://crates.io/crates/wasm-bindgen) |
| Paquets | 0.3.11 | [github.com/maxlestage/Paquets](https://github.com/maxlestage/Paquets) |
| Swift | 6.4.3 | [swift.org](https://www.swift.org/install/) |
| Xcode | 27.2 | [developer.apple.com](https://developer.apple.com/news/releases/) |
| iOS | 27.2.1 | [developer.apple.com](https://developer.apple.com/news/releases/) |
| SQLite | 3.54.1 | [sqlite.org](https://www.sqlite.org/download.html) |
