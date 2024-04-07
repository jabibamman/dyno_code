
# Déploiement sur Google Cloud

## Introduction

Ce guide explique comment déployer une application Rust sur Google Cloud Run. Cloud Run est un service de calcul sans serveur qui permet d'exécuter des conteneurs sur une plateforme gérée. Il prend en charge les conteneurs Docker et les déploie automatiquement sur une infrastructure sans serveur.

## Prérequis

1. Créez un projet Google Cloud Platform (GCP) en utilisant la console GCP.
2. Installez le SDK Google Cloud (gcloud) sur votre machine locale.
3. Installez Docker sur votre machine locale.
4. Installez Rust sur votre machine locale.

## Initialisation du projet

1. Initialisez le SDK: gcloud init et suivez les instructions pour vous connecter et sélectionnez le projet.
2. Activer les API nécessaires:

```bash
gcloud services enable run.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable containerregistry.googleapis.com
```

3. Configurer l'authentification Docker pour Container Registry:

```bash
gcloud auth configure-docker
```

## Déployer l'Application Rust sur Google Cloud Run

### Créer un fichier Dockerfile
   
Il est nécessaire de créer un fichier Dockerfile à la racine du projet pour définir l'image Docker.

### Construire l'image Docker et la pousser vers Google Container Registry (GCR):
   
Remplacez <IMAGE_NAME> par le nom de votre image (par exemple, dyno_code) et <PROJECT_ID> par votre ID de projet Google Cloud.

```bash
gcloud builds submit --tag gcr.io/<PROJECT_ID>/<IMAGE_NAME>
```

### Déployer l'image sur Cloud Run:

Remplacez <SERVICE_NAME> par le nom de votre service Cloud Run (par exemple, dyno-code-service) et <IMAGE_NAME> par le nom de votre image.

```bash
gcloud run deploy <SERVICE_NAME> --image gcr.io/<PROJECT_ID>/<IMAGE_NAME> --platform managed --allow-unauthenticated --region europe-west1
```

`--allow-unauthenticated` permet d'accéder au service sans authentification. Environnement de production, envisagez de gérer l'accès via IAM ou des mécanismes d'authentification.
`--region europe-west1` spécifie la région de déploiement. Choisissez la région la plus proche de vos utilisateurs.