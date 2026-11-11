# Duova · Qui doit quoi ?

Le partage de dépenses au centime près : on crée un groupe, on ajoute les membres et
les dépenses, et Duova dit qui rembourse qui, en un minimum de virements.

| Partie | Où | Technologies |
|---|---|---|
| Site de présentation | [`site/`](site/) | Rust, [Paquets](https://github.com/maxlestage/Paquets), WebAssembly, Canvas 2D |
| App iPhone | [`ios/`](ios/) | Swift 6.4, SwiftUI, iOS 27, Xcode 27 |

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

- `DuovaKit` (paquet Swift) : modèles, centimes, parts, soldes, virements, récap ; testé
  avec Swift Testing (`swift test --package-path ios/DuovaKit`).
- `Duova` (app SwiftUI) : liste des groupes, détail d'un groupe (total, barres de soldes,
  remboursements, dépenses, membres), formulaire de dépense avec la part de chacun en
  direct, partage du récap avec la feuille de partage d'iOS. Les données sont enregistrées
  en JSON sur l'appareil, protégées par le chiffrement d'iOS.

La CI construit l'app et lance les tests sur le runner `xcode-27` de GitHub Actions.
