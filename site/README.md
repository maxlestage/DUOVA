# Le site de Duova

Un site [Paquets](https://github.com/maxlestage/Paquets) : le serveur est une
`paquets::App`, les pages sont des `paquets::Page` rendues côté serveur, et les parties
interactives sont des îlots compilés en WebAssembly. Voir le [README](../README.md) pour
le lancer et le déployer.

| Partie | Où |
|---|---|
| Calcul (centimes, parts, soldes, virements, récap) | `src/split.rs` (serveur et navigateur) |
| Îlots : scène des pièces, scène des virements, simulateur | `src/islands.rs` (serveur et navigateur) |
| Bundle du navigateur : 3D WebGL 2, animations de la page | `src/client/` (wasm32) |
| Serveur, pages, SEO | `src/server/` (natif) |
| Styles, icône, image Open Graph | `public/` (servi à `/`) |
| Tests de toutes les routes | `tests/site.rs` |
