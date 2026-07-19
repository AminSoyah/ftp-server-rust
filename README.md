# Serveur FTP multithreadé en Rust

Un serveur FTP développé en Rust, gérant plusieurs clients simultanément via 
des threads. Implémente les commandes principales du protocole FTP (LIST, 
RETR, STOR, CWD, PWD, DELE, RNFR/RNTO...), avec synchronisation des accès 
partagés via `Arc` et gestion du canal de données en mode passif.

Projet réalisé avec Lucas Sauvage.

## Lancer le projet
cd ftp-server
cargo run -- -d <ftp_server_dir>
Pour voir tous les paramètres disponibles (port, ip, nombre de threads...) :
cargo run -- -h

## Tests
cargo test

## Documentation détaillée
Le rapport complet du projet (architecture, gestion des commandes, canal de 
données, choix techniques) est disponible dans [`docs/RAPPORT.md`](docs/rapport.md).
