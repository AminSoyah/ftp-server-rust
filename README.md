# SR1-Projet2_Soyah_Sauvage

## Binôme

Amin SOYAH
Lucas SAUVAGE 

## L'exécution 

La CLI définit la manière d'exécuter. 

Elle requiert un seul paramètre obligatoire : le dossier qui servira de serveur FTP. 
Tous les autres paramètres (username, password, port, ip, nombre de threads) ont des valeurs par défaut. 

Pour exécuter le projet il faut donc au minimum exécuter :
- `cd ftp-server`
- `cargo run -- -d <ftp_server_dir>`

Pour avoir plus de précisions sur la valeur des paramètres par défaut etc. veuillez exécuter :
- `cd ftp-server`
- `cargo run -- -h`

## Le serveur 

Le serveur ne fait, en réalité, qu'écouter sur l'ip et le port qui lui sont associés. 
La difficultée de cette connexion réside dans l'utilisation de thread pour gérer les connections. 
En effet, le serveur utilise des ressources qui doivent être partagées et utilisées en gardant l'assurance que la donnée n'est pas corrompue après le passage du client.
Pour ce faire, il faut utiliser les modules `Arc`. 

Ce module permet de donner une copie de l'objet `self` au thread, afin d'exécuter la fonction `handle_client_connect`.  

Le module `Arc` permet aussi l'utilisation d'un compteur thread-safe. 
En effet, ce module permet de générer des objets `i32` tout en gardant la certitude qu'un client n'a pas augmenté 2 fois, ou tout autre problème généré par le multi-threading.
Avant chaque thread le "générateur" est augmenté, la valeur précédente est retournée et le thread se lance avec la valeur précédente. 


## Les commandes:

Le module `commands.rs` contient toute la logisue de traitement des commandes FTP, on a commencé par créer une enum: `FtpCommand`, qui servira a gérer tous les cas, tous les msgs, grâce au pattern matching, on peut gérer tous les cas. 
Ensuite la fonction: `parse_ftp_command` qui nous permet de transformer une ligne de texte brute en commande typée par exemple:

"USER aminou" -> split sur espaces -> ["USER", "aminou"]
              -> match sur "USER" -> Some(FtpCommand::Usr("aminou"))

- `ftp_command_list` cette fonction nous permet de lister des fichiers, elle prend en paramètre un user_session et renvoie un result, cette fonction lit le contenu du repertoire actuel du client grâce a la fonction `read_dir` puis pour chaque entrée récupére le nom de chaque fichier avec `entry.file_name`, dans cette fonction on gére les caractéres spéciaux aussi grâce a la fonction `to_string_lossy` qui convertir OsString en String.

- `ftp_command_pwd` cette fonction prend aussi un user_session en paramètre et renvoie le chemin du dossier courant.
- Format de réponse:

```bash
    257 "/chemin/complet/du/dossier" is the current directory.
```

- `ftp_command_cwd` cette fonction nous permet de changer de dossier, elle prend donc en paramètre *new_dir* de type `str` et user_session; on construit le nouveau chemin avec `PathBuf::push()`, on verifiie que ce chemin existe et que ce chemin est un dossier. Pour cette fonction 3 types de réponses sont attendues: 
    - `250 Directory successfully changed` la commande s'est réalisée avec succès
    - `550 Directory does not exist` la commande a echoué; le chemin est introuvable
    - `550 Not a directory` un échec également; le chemin pointe vers un fichier

Et finalement on modifie le dossier courrant du client: `session.current_dir = new_path.to_string_lossy().to_string();`

- `ftp_command_retr_file` : 

```rust
pub fn ftp_command_retr_file(path: &PathBuf, data_stream: &mut TcpStream) -> io::Result
```

Le rôle de cette fonction est de lire un fichier depuis le disque et l'envoie sur le canal de donnée. Pour pouvoir gérer le cas des gros fichiers on a utilisé une boucle circulaire de 4096 bytes, c'est à dire pour chaque tour de boucle on lit 4096 bytes, ca nous permet de ne pas trop charger le buffer, d'améliorer la performance de cette fonction et de gérer différent types de fichier : `.txt`, `.md`, `.jpg`. 
Pour chaque transfert on vérifie que le fichier existe et que c'est bien un fichier.
Le module `commands.rs` contient toute la logique de traitement des commandes FTP, on a commencé par créé une enum: `FtpCommand`, qui servira a gérer tous les cas, tous les msgs, grâce au pattern matching, on peut gérer tous les cas. Ensuite la fonction: `parse_ftp_command` qui nous permet de transformer une ligne de texte brute en commande typée par exmeple:
"USER aminou" -> split sur espaces -> ["USER", "aminou"]
              -> match sur "USER" -> Some(FtpCommand::Usr("aminou"))

- `ftp_command_list` cette fonction nous permet de lister des fichiers, elle prend en paramètre un user_session et renvoie un result, cette fonction lit le contenu du repertoire actuel du client grâce a la fonction `read_dir` puis pour chaque entrée récupére le nom de chaque fichier avec `entry.file_name`, dans cette fonction on gére les caractéres spéciaux aussi grâce a la fonction `to_string_lossy` qui convertir OsString en String.

- `ftp_command_pwd` cette fonction prend aussi un user_session en paramètre et renvoie le chemin du dossier courant.
- Format de réponse:
```bash
    257 "/chemin/complet/du/dossier" is the current directory.
```
- `ftp_command_cwd` cette fonction nous permet de changer de dossier, elle prend donc en paramètre *new_dir* de type `str` et user_session; on construit le nouveau chemin avec `PathBuf::push()`, on verifiie que ce chemin existe et que ce chemin est un dossier. Pour cette fonction 3 types de réponses sont attendues: 
    - `250 Directory successfully changed` la commande s'est réalisée avec è
    - `550 Directory does not exist` la commande a echoué; le chemin est introuvable
    - `550 Not a directory` un échec également; le chemin pointe vers un fichier
Et finalement on modifie le dossier courrant du client: `session.current_dir = new_path.to_string_lossy().to_string();`

- `ftp_command_retr_file` 
```rust
pub fn ftp_command_retr_file(filename: &str, data_stream: &mut TcpStream) -> io::Result
```
Le rôle de cette fonction est de lire un fichier depuis le disque et l'envoie sur le canal de donnée. 
Pour pouvoir gérer le cas des gros fichiers on a utilisé une boucle circulaire de 4096 bytes, c'est à dire pour chaque tour de boucle on lit 4096 bytes, ca nous permet de ne pas trop charger le buffer, d'améliorer la performance de cette fonction et de gérer différent types de fichier : `.txt`, `.md`, `.jpg`. 
Pour chaque transfert on vérifie que le fichier existe et que c'est bien un fichier. 

Cette fonction utilise 2 autres fonctions auxiliaires car le traitement de la donnée est différent si c'est un répertoire ou si c'est un fichier. 

- `ftp_command_stor` 
```rust
pub fn ftp_command_stor(filename: &str, data_stream: &mut TcpStream) -> io::Result
```
Le rôle de cette fonction est de recevoir un fichier depuis le canal de données et le sauvegarder en local.
Du même principe que la fonction précédente `RETR` mais dans le sens inverse, cette fonction crée un fichier, recoit des morceaux de 4096 bytes et grâce a une boucle écrit au fur et à mesure .
Pareil que pour `RETR`, cette fonction utilise des fonctions auxiliaires en fonction du type (dossier ou fichier).

- Gestion du canal de données
    Le protocole Ftp utilise 2 cannaux différents: canal de contrôle et canal de données, on devait donc, pour que certaines fonctions comme `LIST`, `RETR` ou `STOR` fonctionnent correctement, implémentés la fonction : `setup_passive_mode` qui ouvre un port pour le canal de données et génère la réponse `PASV`, cette fonction renvoie un TcpListener pour accepter la connexion et la réponse FTP formatée.

- `connect_data_socket` cette fonction accepte la connexion sur le canal de données et exécute l'operation passée en paramètre (LIST/RETR/STOR). Cette fonction accepte la connexion, match sur cmd qui est de type enum `DataChannelFunction`, exécute l'opération correspondante et ferme la connexion.

- `handle_data_transfer` Cette fonction gére le protocole d'une opération nécessitant le canal de données: elle vérifie que PASV a été appelé pour ouvrir une connexion, envoie un msg sur le canal de contrôle, ensuite appelle `connect_data_socket` pour effectuer cette opération et fini par envoyer un msg de fin d'opération sur le canal de contrôle.

- `handle_command` est la fonction qui va orchestrer le bon fonctionnement des commandes

- `handle_client_session` c'est la fonction qui gére toute la session d'un client connecté; elle envoie un message de bienvenue, lance une boucle infinie puis lit chaque ligne depuis le canal de contrôle et fait appel a la fonction `parse_ftp_command` pour parser ces lignes, appelle la fonction `handle_command` pour exécuter les commandes recupérer suite au parsinget gére les erreurs grâce au `if let Err(e)` et recommence. Cette boucle ne prend fin que lorsque le client se déconnecte

De plus, il existe des commandes plus simples, directement définie dans la fonction de matching des commandes : 

- La logique de gestion : 
    - Renommer : 
        - RNFR qui stipule le nom de la donnée à modifier, et stocke le chemin vers cette donnée dans la session de l'utilisateur pour RNTO
        - RNTO qui stipule le nouveau nom de la donnée, et va chercher le chemin dans la session.

    - Supprimer : 
        - DELE est utilisée pour supprimer un fichier. 
        - RMD est utilisée pour supprimer un dossier. 

    - Créer : 
        - MKD est utilisée pour créer un répertoire vide. 
        - STOR est utilisée pour créer un fichier vide (i.e. elle "déplace" un fichier vide, c'est pourquoi STOR)


## Les autres 

- Les **erreurs** dérivent du module `thiserror` qui permet la création de nouvelles erreurs, qui peuvent ou non dériver d'autres erreurs du module io. 

- L'utilisation d'**env_logger** permet de définir plusieurs niveaux de logs, via des macros telles que info!, ou debug! (qui ont des noms auto-descriptifs). 



## Présentation des ajouts 

### Utilisateur restreint au répertoire de départ

Afin d'empêcher à l'utilisateur d'accéder à plus de données que le serveur lui en propose, il faut séparer la gestion du chemin en 2 : 

- Le chemin relatif : Celui donné à l'utilisateur, il dérive de "/" 
- Le chemin absolu : Celui recréé à chaque demande de l'utilisateur. Il retire le "/" de départ et rajoute le `starting_dir` (qui est uniquement stocké dans le serveur.) 

Cela permet à l'utilisateur d'être restreint tout en permettant au code de "comprendre" de quel chemin l'on parle. 


### Utilisation d'une archive temporaire pour le download 

La méthode qui gère la commande `RETR` envoie la donnée requise en passant par une archive. 
Cela permet de contourner l'utilisation de `RETR` qui ne va chercher qu'un seul fichier. 
Le fichier selectionné est donc une archive qui contient le contenu du répertoire. 

### Utilisation de buffers pour manipuler le stream 

Afin d'éviter de faire des appels au stream à chaque fois que quelque chose est lu ou envoyé, il faut créer des Buffers. 
Ces buffers permettent de stocker plus de texte. 
Cela permet d'éviter un appel système pour chaque octet lu/envoyé, car les buffers ne se vident que lors d'un appel à `flush` où lorsqu'ils sont pleins.

## LIVRABLE 2 

Avant de s'attaquer directement au code, nous avons décidé de s'attarder un petit peu sur la conception du projet. 
Le modèle que nous en avons déduit prévoit 5 "objets"(<small>Les classes n'existant pas en Rust, je fais ici référence à l'implémentation du structure</small>) : 

- Le client : Pas de code spécial car utilisation de FileZilla 
- Les cookies : Sous la forme d'une structure `UserSession`, l'objet créé garde en mémoire les informations de l'utilisateur ainsi que son exploration dans le serveur. Chaque client possède une structure. 
- Le serveur : Il écoute en boucle sur l'ip et le port spécifié, et gère les connexions des clients. Un thread est ouvert par client. 
- Les commandes : Le serveur doit envoyer des commandes en fonction des inputs reçus par le client. Il faut alors parser l'input, exécuter la commande, et construire la réponse en fonction de son échec ou non. 
- Les erreurs : Au nombre pharamineux de 2, ce sont des erreurs personnalisées. 


##  Tests

Pour notre projet, nous avons ajouté des tests pour accompagner notre code et vérifier le bon fonctionnement de certaines fonctionnalités.

### Structure des tests

Les tests sont organisés dans le dossier ftp-server/tests :

- client_test.rs : tests du client FTP, la résolution des chemins, la gestion du mode passif et le traitement des commandes simples (ex. TYPE).

- commands_test.rs : tests des commandes FTP (USER, PWD, LIST, CWD) et de la logique de leurs réponses.

Chaque test utilise des sessions utilisateurs simulées (UserSession) et des connexions réseau simulées via TcpListener/TcpStream. Cela permet d’exécuter les tests sans avoir besoin d’un serveur FTP réel.

Ce qui est testé
Client FTP (client_test.rs) :
* Résolution des chemins : `resolve_path`, `resolve_any_path`,`absolute_or_relative`.
* Mise en place du mode passif : `setup_passive_mode` et calcul des ports associés.
* Gestion de commandes simples : `handle_command` 

Commandes FTP (commands_test.rs) :

* Parsing des commandes entrantes : `parse_ftp_command`.
* Réponses aux commandes standards :`PWD`, `LIST`, `CWD`.
* Gestion des répertoires existants et inexistants

Pour faciliter les tests, certaines fonctions ont été rendues publiques et tempfile::TempDir est utilisé pour créer des répertoires temporaires isolés lors des tests.

### Exécution des tests

Pour lancer tous les tests, utilisez la commande :
cargo test
```bash
     Running tests/client_test.rs (target/debug/deps/client_test-4f7fe6aed2612add)

running 7 tests
test test_passive_mode_calculates_port ... ok
test test_absolute_or_relative_absolute ... ok
test test_absolute_or_relative_relative ... ok
test test_setup_passive_mode_opens_port ... ok
test test_resolve_any_path_absolute ... ok
test test_handle_command_type ... ok
test test_resolve_path_root ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/commands_test.rs (target/debug/deps/commands_test-31e4904779664c5e)

running 8 tests
test test_parse_list ... ok
test test_parse_pwd ... ok
test test_parse_user ... ok
test test_pwd ... ok
test test_cwd_not_exists ... ok
test test_cwd_success ... ok
test test_list_empty ... ok
test test_list_with_files ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```  
