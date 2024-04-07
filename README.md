# Dyno code

## Description

Dyno code est un service web qui permet de compiler et d'exécuter du code en ligne. Il est écrit en Rust et utilise le framework web actix-web.

## Fonctionnalités

- Compilation et exécution de code en ligne
- Support des langages de programmation suivants:
  - Python
  - Lua

## Déploiement sur Google Cloud

Voir [Déploiement sur Google Cloud](docs/google_cloud.md).

## Développement

### Prérequis

- Rust
- Docker

### Lancer le serveur

```bash
cargo run
```

### Lancer le serveur avec Docker

```bash
docker build -t dyno_code .
docker run -p 8080:8080 dyno_code
```	
